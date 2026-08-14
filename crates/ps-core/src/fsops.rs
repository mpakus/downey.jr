//! Safe filesystem path resolution within a project directory.

use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use unicode_normalization::UnicodeNormalization;

use crate::{Error, Result};

const MAX_FILE_NAME_BYTES: usize = 255;
static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

/// Resolves a project-relative path without permitting access outside `project_root`.
///
/// Each name is normalized to NFC. The path's parent must already exist so it can
/// be canonicalized and checked against the canonical project root.
pub fn resolve(project_root: &Path, rel_path: &Path) -> Result<PathBuf> {
    let canonical_root = project_root
        .canonicalize()
        .map_err(|source| Error::io("open the project directory", project_root, source))?;
    if !canonical_root.is_dir() {
        return Err(unsafe_path(
            project_root,
            "the project path is not a directory",
        ));
    }

    validate_raw_path(rel_path)?;
    let normalized = normalize_relative_path(rel_path)?;
    if normalized.as_os_str().is_empty() {
        return Ok(canonical_root);
    }

    let candidate = canonical_root.join(normalized);
    let parent = candidate.parent().ok_or_else(|| {
        unsafe_path(
            rel_path,
            "the path does not have a project directory parent",
        )
    })?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|source| Error::io("open the path's parent directory", parent, source))?;
    ensure_inside_project(&canonical_root, &canonical_parent)?;

    if fs::symlink_metadata(&candidate).is_ok() {
        let canonical_candidate = candidate
            .canonicalize()
            .map_err(|source| Error::io("resolve the path", &candidate, source))?;
        ensure_inside_project(&canonical_root, &canonical_candidate)?;
    }

    Ok(candidate)
}

/// Creates one directory inside a project without replacing an existing item.
pub fn mkdir(project_root: &Path, rel_path: &Path) -> Result<PathBuf> {
    let destination = resolve_child(project_root, rel_path)?;
    match fs::create_dir(&destination) {
        Ok(()) => {
            sync_parent(&destination)?;
            Ok(destination)
        }
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            Err(name_conflict(&destination))
        }
        Err(source) => Err(Error::io("create the folder", &destination, source)),
    }
}

/// Creates one empty file atomically inside a project without replacing an existing item.
pub fn create_file(project_root: &Path, rel_path: &Path) -> Result<PathBuf> {
    let destination = resolve_child(project_root, rel_path)?;
    if path_is_occupied(&destination) {
        return Err(name_conflict(&destination));
    }

    let parent = destination
        .parent()
        .ok_or_else(|| unsafe_path(rel_path, "the file does not have a parent directory"))?;
    let (temporary_path, temporary_file) = create_temporary_file(parent)?;
    if let Err(source) = temporary_file.sync_all() {
        drop(temporary_file);
        let _ = fs::remove_file(&temporary_path);
        return Err(Error::io(
            "sync the temporary file",
            &temporary_path,
            source,
        ));
    }
    drop(temporary_file);

    let reservation = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)
    {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            let _ = fs::remove_file(&temporary_path);
            return Err(name_conflict(&destination));
        }
        Err(source) => {
            let _ = fs::remove_file(&temporary_path);
            return Err(Error::io("create the file", &destination, source));
        }
    };
    if let Err(source) = reservation.sync_all() {
        drop(reservation);
        let _ = fs::remove_file(&temporary_path);
        return Err(Error::io("sync the new file", &destination, source));
    }
    drop(reservation);

    if let Err(source) = fs::rename(&temporary_path, &destination) {
        let _ = fs::remove_file(&temporary_path);
        return Err(Error::io("finish creating the file", &destination, source));
    }
    sync_parent(&destination)?;
    Ok(destination)
}

/// Renames one project item atomically without replacing an existing item.
pub fn rename(project_root: &Path, from: &Path, to: &Path) -> Result<PathBuf> {
    let source_path = resolve_child(project_root, from)?;
    let destination = resolve_child(project_root, to)?;
    if source_path == destination {
        return Ok(destination);
    }
    if path_is_occupied(&destination) {
        return Err(name_conflict(&destination));
    }

    fs::rename(&source_path, &destination)
        .map_err(|source| Error::io("rename the file or folder", &source_path, source))?;
    sync_parent(&source_path)?;
    if source_path.parent() != destination.parent() {
        sync_parent(&destination)?;
    }
    Ok(destination)
}

fn validate_raw_path(path: &Path) -> Result<()> {
    if path.is_absolute() {
        return Err(unsafe_path(path, "absolute paths are not allowed"));
    }
    if path.as_os_str().as_encoded_bytes().contains(&0) {
        return Err(unsafe_path(path, "file names cannot contain NUL bytes"));
    }
    Ok(())
}

fn normalize_relative_path(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        let Component::Normal(name) = component else {
            return Err(unsafe_path(
                path,
                "only relative file and directory names are allowed",
            ));
        };
        let name = name
            .to_str()
            .ok_or_else(|| unsafe_path(path, "file names must be valid Unicode"))?;
        let name = name.nfc().collect::<String>();
        validate_name(path, &name)?;
        normalized.push(name);
    }
    Ok(normalized)
}

fn validate_name(path: &Path, name: &str) -> Result<()> {
    if matches!(name, "." | "..") {
        return Err(unsafe_path(path, "the name is reserved by the filesystem"));
    }
    if name.len() > MAX_FILE_NAME_BYTES {
        return Err(unsafe_path(path, "a file name is longer than 255 bytes"));
    }
    Ok(())
}

fn resolve_child(project_root: &Path, rel_path: &Path) -> Result<PathBuf> {
    let resolved = resolve(project_root, rel_path)?;
    let canonical_root = project_root
        .canonicalize()
        .map_err(|source| Error::io("open the project directory", project_root, source))?;
    if resolved == canonical_root {
        Err(unsafe_path(
            rel_path,
            "the project root cannot be changed by this operation",
        ))
    } else {
        Ok(resolved)
    }
}

fn create_temporary_file(parent: &Path) -> Result<(PathBuf, File)> {
    for _ in 0..100 {
        let sequence = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".1537paperstreet.tmp-{}-{sequence}",
            std::process::id()
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => return Err(Error::io("create a temporary file", path, source)),
        }
    }
    Err(unsafe_path(
        parent,
        "a unique temporary file name could not be allocated",
    ))
}

fn sync_parent(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| unsafe_path(path, "the path does not have a parent directory"))?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| Error::io("sync the containing folder", parent, source))
}

fn name_conflict(path: &Path) -> Error {
    Error::NameConflict {
        path: path.to_path_buf(),
        suggested_name: next_available_name(path),
    }
}

fn next_available_name(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("item");
    let name_path = Path::new(file_name);
    let stem = name_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or(file_name);
    let extension = name_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{extension}"))
        .unwrap_or_default();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));

    let mut number = 2_u64;
    loop {
        let suffix = format!(" {number}{extension}");
        let stem = truncate_utf8(stem, MAX_FILE_NAME_BYTES.saturating_sub(suffix.len()));
        let candidate = format!("{stem}{suffix}");
        if !path_is_occupied(&parent.join(&candidate)) {
            return candidate;
        }
        number = number.saturating_add(1);
    }
}

fn truncate_utf8(value: &str, max_bytes: usize) -> &str {
    let mut end = value.len().min(max_bytes);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

fn path_is_occupied(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn ensure_inside_project(project_root: &Path, path: &Path) -> Result<()> {
    if path.starts_with(project_root) {
        Ok(())
    } else {
        Err(Error::PathOutsideProject {
            path: path.to_path_buf(),
        })
    }
}

fn unsafe_path(path: impl Into<PathBuf>, reason: &'static str) -> Error {
    Error::UnsafePath {
        path: path.into(),
        reason,
    }
}
