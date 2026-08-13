//! Safe filesystem path resolution within a project directory.

use std::fs;
use std::path::{Component, Path, PathBuf};

use unicode_normalization::UnicodeNormalization;

use crate::{Error, Result};

const MAX_FILE_NAME_BYTES: usize = 255;

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
