//! Versioned JSON storage with atomic writes and corruption recovery.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{Error, Result};

const DEBOUNCE: Duration = Duration::from_millis(500);
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A warning produced while opening a recoverable store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoreWarning {
    /// Invalid JSON was moved aside and defaults were loaded.
    CorruptFileMoved {
        /// The path containing the preserved invalid bytes.
        preserved_at: PathBuf,
    },
}

/// A serializable document with an explicit schema version.
pub trait VersionedDocument: Default + DeserializeOwned + Serialize {
    /// The newest schema version understood by the application.
    const SCHEMA_VERSION: u32;

    /// Migrates a raw JSON document from `from` to [`Self::SCHEMA_VERSION`].
    fn migrate(value: Value, from: u32) -> Result<Value> {
        let _ = value;
        Err(Error::UnsupportedSchema {
            found: from,
            supported: Self::SCHEMA_VERSION,
        })
    }

    /// Validates a decoded document before it is exposed to callers.
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// A debounced, versioned JSON document stored on disk.
pub struct JsonStore<T> {
    path: PathBuf,
    value: T,
    dirty_since: Option<Instant>,
    warning: Option<StoreWarning>,
}

impl<T: VersionedDocument> JsonStore<T> {
    /// Opens a document, migrating old schemas and preserving invalid JSON.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                let value = T::default();
                value.validate()?;
                return Ok(Self {
                    path,
                    value,
                    dirty_since: None,
                    warning: None,
                });
            }
            Err(source) => return Err(Error::io("read application data", &path, source)),
        };

        let mut raw: Value = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(_) => {
                let preserved_at = corrupt_path(&path);
                fs::rename(&path, &preserved_at).map_err(|source| {
                    Error::io("preserve invalid application data", &path, source)
                })?;
                let value = T::default();
                value.validate()?;
                return Ok(Self {
                    path,
                    value,
                    dirty_since: None,
                    warning: Some(StoreWarning::CorruptFileMoved { preserved_at }),
                });
            }
        };

        let version = raw
            .get("schema_version")
            .and_then(Value::as_u64)
            .and_then(|version| u32::try_from(version).ok())
            .unwrap_or(0);

        if version > T::SCHEMA_VERSION {
            return Err(Error::UnsupportedSchema {
                found: version,
                supported: T::SCHEMA_VERSION,
            });
        }

        if version < T::SCHEMA_VERSION {
            let backup = appended_path(&path, &format!(".bak.{version}"));
            fs::copy(&path, &backup)
                .map_err(|source| Error::io("back up application data", &backup, source))?;
            raw = T::migrate(raw, version)?;
            let migrated = serde_json::to_vec_pretty(&raw)
                .map_err(|source| Error::json("encode migrated", &path, source))?;
            atomic_write(&path, &migrated)?;
        }

        let value: T =
            serde_json::from_value(raw).map_err(|source| Error::json("decode", &path, source))?;
        value.validate()?;

        Ok(Self {
            path,
            value,
            dirty_since: None,
            warning: None,
        })
    }

    /// Returns the current in-memory document.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the current in-memory document for controlled service mutations.
    pub(crate) fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Restarts the debounce window after a controlled in-memory mutation.
    pub(crate) fn mark_dirty(&mut self) {
        self.dirty_since = Some(Instant::now());
    }

    /// Replaces the in-memory document and restarts the debounce window.
    pub fn replace(&mut self, value: T) {
        self.value = value;
        self.dirty_since = Some(Instant::now());
    }

    /// Mutates the in-memory document and restarts the debounce window.
    pub fn update(&mut self, update: impl FnOnce(&mut T)) {
        update(&mut self.value);
        self.dirty_since = Some(Instant::now());
    }

    /// Writes pending changes if they have been idle for 500 milliseconds.
    pub fn flush_if_due(&mut self) -> Result<bool> {
        match self.dirty_since {
            Some(changed_at) if changed_at.elapsed() >= DEBOUNCE => self.flush(),
            _ => Ok(false),
        }
    }

    /// Immediately writes pending changes through a same-directory temporary file.
    pub fn flush(&mut self) -> Result<bool> {
        if self.dirty_since.is_none() {
            return Ok(false);
        }
        self.value.validate()?;
        let bytes = serde_json::to_vec_pretty(&self.value)
            .map_err(|source| Error::json("encode", &self.path, source))?;
        atomic_write(&self.path, &bytes)?;
        self.dirty_since = None;
        Ok(true)
    }

    /// Flushes pending changes before the store is closed.
    pub fn close(mut self) -> Result<()> {
        self.flush().map(|_| ())
    }

    /// Returns and clears the warning produced while opening the store.
    pub fn take_warning(&mut self) -> Option<StoreWarning> {
        self.warning.take()
    }

    /// Returns the backing file path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    atomic_write_with_hook(path, bytes, || {})
}

fn atomic_write_with_hook(path: &Path, bytes: &[u8], before_rename: impl FnOnce()) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        counter
    ));
    let mut cleanup = TemporaryFile::new(temporary.clone());

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|source| {
            Error::io(
                "create a temporary application-data file",
                &temporary,
                source,
            )
        })?;
    file.write_all(bytes)
        .map_err(|source| Error::io("write application data", &temporary, source))?;
    file.sync_all()
        .map_err(|source| Error::io("sync application data", &temporary, source))?;

    before_rename();
    fs::rename(&temporary, path)
        .map_err(|source| Error::io("replace application data", path, source))?;
    cleanup.disarm();
    Ok(())
}

fn appended_path(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

fn corrupt_path(path: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    appended_path(path, &format!(".corrupt.{timestamp}"))
}

struct TemporaryFile {
    path: PathBuf,
    armed: bool,
}

impl TemporaryFile {
    fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::thread;

    use super::*;

    const CHILD_PATH: &str = "PAPERSTREET_ATOMIC_CHILD_PATH";
    const CHILD_MARKER: &str = "PAPERSTREET_ATOMIC_CHILD_MARKER";

    #[test]
    fn atomic_write_survives_process_kill() {
        if let (Some(path), Some(marker)) =
            (std::env::var_os(CHILD_PATH), std::env::var_os(CHILD_MARKER))
        {
            atomic_write_with_hook(Path::new(&path), br#"{"value":"new"}"#, || {
                fs::write(marker, b"ready").expect("write child marker");
                loop {
                    thread::park();
                }
            })
            .expect("child atomic write");
            return;
        }

        let temp = tempfile::tempdir().expect("temporary directory");
        let path = temp.path().join("state.json");
        let marker = temp.path().join("ready");
        fs::write(&path, br#"{"value":"original"}"#).expect("original state");

        let mut child = Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "store::tests::atomic_write_survives_process_kill",
                "--nocapture",
            ])
            .env(CHILD_PATH, &path)
            .env(CHILD_MARKER, &marker)
            .spawn()
            .expect("atomic-write child");

        for _ in 0..500 {
            if marker.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(marker.exists(), "child did not reach the rename boundary");
        child.kill().expect("kill atomic-write child");
        child.wait().expect("wait for atomic-write child");

        let bytes = fs::read(&path).expect("preserved state");
        assert_eq!(bytes, br#"{"value":"original"}"#);
        serde_json::from_slice::<Value>(&bytes).expect("valid preserved JSON");
    }
}
