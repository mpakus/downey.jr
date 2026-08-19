//! On-disk cache of rendered Mermaid SVG, keyed by source hash and theme.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{Error, Result};

const MAX_SVG_BYTES: usize = 2 * 1024 * 1024;
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Content-addressed SVG cache under `~/.1537paperstreet/cache/mermaid/`.
#[derive(Clone, Debug)]
pub struct MermaidSvgCache {
    directory: PathBuf,
}

impl MermaidSvgCache {
    /// Opens (or creates) the cache directory.
    ///
    /// ```
    /// use ps_core::mermaid_cache::MermaidSvgCache;
    ///
    /// let dir = tempfile::tempdir().unwrap();
    /// let cache = MermaidSvgCache::open(dir.path()).unwrap();
    /// let hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    /// assert!(cache.get(hash, "paper-light").unwrap().is_none());
    /// ```
    pub fn open(directory: impl Into<PathBuf>) -> Result<Self> {
        let directory = directory.into();
        fs::create_dir_all(&directory)
            .map_err(|source| Error::io("create the Mermaid cache", &directory, source))?;
        Ok(Self { directory })
    }

    /// Returns a cached SVG when both the source hash and theme match.
    pub fn get(&self, source_hash: &str, theme_id: &str) -> Result<Option<String>> {
        let path = self.path_for(source_hash, theme_id)?;
        match fs::read_to_string(&path) {
            Ok(svg) => Ok(Some(svg)),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(Error::io("read the Mermaid cache", path, source)),
        }
    }

    /// Stores a rendered SVG. Existing entries for the same key are replaced.
    pub fn put(&self, source_hash: &str, theme_id: &str, svg: &str) -> Result<()> {
        if svg.len() > MAX_SVG_BYTES {
            return Err(Error::UnsafePath {
                path: self.directory.clone(),
                reason: "the diagram SVG is larger than 2 MB",
            });
        }
        if !svg.trim_start().starts_with("<svg") {
            return Err(Error::UnsafePath {
                path: self.directory.clone(),
                reason: "the cache only stores SVG diagrams",
            });
        }
        let path = self.path_for(source_hash, theme_id)?;
        atomic_write(&path, svg.as_bytes())
    }

    fn path_for(&self, source_hash: &str, theme_id: &str) -> Result<PathBuf> {
        let key = cache_key(source_hash, theme_id)?;
        Ok(self.directory.join(format!("{key}.svg")))
    }
}

fn cache_key(source_hash: &str, theme_id: &str) -> Result<String> {
    if source_hash.len() != 64 || !source_hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(Error::UnsafePath {
            path: PathBuf::from(source_hash),
            reason: "the diagram hash must be 64 hexadecimal characters",
        });
    }
    if theme_id.is_empty()
        || !theme_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err(Error::UnsafePath {
            path: PathBuf::from(theme_id),
            reason: "the theme id must be a slug of ASCII letters, digits, and hyphens",
        });
    }
    Ok(
        blake3::hash(format!("{source_hash}\0{theme_id}").as_bytes())
            .to_hex()
            .to_string(),
    )
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Err(Error::UnsafePath {
            path: path.to_path_buf(),
            reason: "the cache path has no parent directory",
        });
    };
    let file_name = path.file_name().map_or_else(
        || std::ffi::OsString::from("diagram.svg"),
        std::ffi::OsStr::to_os_string,
    );
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(
        ".{}.{}.{}.tmp",
        file_name.to_string_lossy(),
        std::process::id(),
        counter
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|source| Error::io("create a temporary Mermaid cache file", &temporary, source))?;
    let write = file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|source| Error::io("write the Mermaid cache", &temporary, source));
    if let Err(error) = write {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    if let Err(source) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(Error::io("replace the Mermaid cache", path, source));
    }
    Ok(())
}
