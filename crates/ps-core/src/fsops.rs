//! Safe filesystem path resolution within a project directory.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use unicode_normalization::UnicodeNormalization;

use crate::{Error, Result};

const MAX_FILE_NAME_BYTES: usize = 255;
const PROGRESS_THRESHOLD_BYTES: u64 = 50 * 1024 * 1024;
const COPY_BUFFER_BYTES: usize = 1024 * 1024;
static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

/// The action to take when a copy destination already exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictStrategy {
    /// Snapshot and replace the destination.
    Replace,
    /// Copy to the next available numbered name.
    KeepBoth,
    /// Leave the destination unchanged.
    Skip,
}

/// The result of one copy operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CopyOutcome {
    /// The source was copied to this path.
    Copied {
        /// The final destination path.
        path: PathBuf,
    },
    /// The source was not copied because the destination already existed.
    Skipped {
        /// The unchanged destination path.
        path: PathBuf,
    },
}

/// Byte progress for a copy larger than 50 MiB.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CopyProgress {
    /// Bytes copied so far.
    pub bytes_copied: u64,
    /// Total bytes in all regular files being copied.
    pub total_bytes: u64,
}

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

/// Recursively copies a project item with explicit conflict handling.
///
/// `before_replace` must persist the destination's pre-replacement snapshot.
/// Replacement stops without changing the destination if that hook fails.
/// `on_progress` is called only when the source contains more than 50 MiB.
pub fn copy<B, P>(
    project_root: &Path,
    from: &Path,
    to: &Path,
    conflict: ConflictStrategy,
    mut before_replace: B,
    mut on_progress: P,
) -> Result<CopyOutcome>
where
    B: FnMut(&Path) -> Result<()>,
    P: FnMut(CopyProgress),
{
    let source = resolve_child(project_root, from)?;
    let mut destination = resolve_child(project_root, to)?;
    let source_metadata = fs::symlink_metadata(&source)
        .map_err(|source_error| Error::io("open the copy source", &source, source_error))?;
    reject_symlink(&source, &source_metadata)?;

    let destination_exists = path_is_occupied(&destination);
    if destination_exists {
        match conflict {
            ConflictStrategy::Skip => return Ok(CopyOutcome::Skipped { path: destination }),
            ConflictStrategy::KeepBoth => {
                let suggested = next_available_name(&destination);
                let parent = destination.parent().ok_or_else(|| {
                    unsafe_path(
                        &destination,
                        "the destination does not have a parent folder",
                    )
                })?;
                destination = parent.join(suggested);
            }
            ConflictStrategy::Replace => {
                if source == destination {
                    return Err(unsafe_path(
                        from,
                        "an item cannot replace itself during a copy",
                    ));
                }
            }
        }
    }

    let canonical_source = source
        .canonicalize()
        .map_err(|source_error| Error::io("open the copy source", &source, source_error))?;
    if source_metadata.is_dir()
        && (destination.starts_with(&canonical_source)
            || (destination_exists
                && conflict == ConflictStrategy::Replace
                && canonical_source.starts_with(&destination)))
    {
        return Err(unsafe_path(
            to,
            "a folder cannot be copied into itself or one of its children",
        ));
    }

    let total_bytes = measure_item(&source)?;
    let report_progress = total_bytes > PROGRESS_THRESHOLD_BYTES;
    let mut progress = CopyProgress {
        bytes_copied: 0,
        total_bytes,
    };

    if destination_exists && conflict == ConflictStrategy::Replace {
        before_replace(&destination)?;
        replace_with_copy(
            &source,
            &destination,
            &mut progress,
            report_progress,
            &mut on_progress,
        )?;
    } else {
        copy_new_item(
            &source,
            &destination,
            &mut progress,
            report_progress,
            &mut on_progress,
        )?;
    }

    Ok(CopyOutcome::Copied { path: destination })
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

fn measure_item(path: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|source| Error::io("inspect the copy source", path, source))?;
    reject_symlink(path, &metadata)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if !metadata.is_dir() {
        return Err(unsafe_path(path, "only files and folders can be copied"));
    }

    let mut total = 0_u64;
    let entries = fs::read_dir(path)
        .map_err(|source| Error::io("read the copy source folder", path, source))?;
    for entry in entries {
        let entry = entry.map_err(|source| Error::io("read a folder entry", path, source))?;
        total = total.saturating_add(measure_item(&entry.path())?);
    }
    Ok(total)
}

fn copy_new_item<P>(
    source: &Path,
    destination: &Path,
    progress: &mut CopyProgress,
    report_progress: bool,
    on_progress: &mut P,
) -> Result<()>
where
    P: FnMut(CopyProgress),
{
    let metadata = fs::symlink_metadata(source)
        .map_err(|source_error| Error::io("inspect the copy source", source, source_error))?;
    reject_symlink(source, &metadata)?;
    if metadata.is_file() {
        return copy_new_file(
            source,
            destination,
            &metadata,
            progress,
            report_progress,
            on_progress,
        );
    }
    if !metadata.is_dir() {
        return Err(unsafe_path(source, "only files and folders can be copied"));
    }

    match fs::create_dir(destination) {
        Ok(()) => {}
        Err(source_error) if source_error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(name_conflict(destination));
        }
        Err(source_error) => {
            return Err(Error::io(
                "create the destination folder",
                destination,
                source_error,
            ));
        }
    }

    let copy_result =
        copy_directory_entries(source, destination, progress, report_progress, on_progress);
    if let Err(error) = copy_result {
        let _ = remove_owned_item(destination);
        return Err(error);
    }
    fs::set_permissions(destination, metadata.permissions()).map_err(|source_error| {
        Error::io(
            "preserve the destination folder permissions",
            destination,
            source_error,
        )
    })?;
    sync_parent(destination)
}

fn copy_directory_entries<P>(
    source: &Path,
    destination: &Path,
    progress: &mut CopyProgress,
    report_progress: bool,
    on_progress: &mut P,
) -> Result<()>
where
    P: FnMut(CopyProgress),
{
    let entries = fs::read_dir(source)
        .map_err(|source_error| Error::io("read the copy source folder", source, source_error))?;
    for entry in entries {
        let entry =
            entry.map_err(|source_error| Error::io("read a folder entry", source, source_error))?;
        let name = entry
            .file_name()
            .to_str()
            .ok_or_else(|| unsafe_path(entry.path(), "file names must be valid Unicode"))?
            .nfc()
            .collect::<String>();
        validate_name(&entry.path(), &name)?;
        copy_new_item(
            &entry.path(),
            &destination.join(name),
            progress,
            report_progress,
            on_progress,
        )?;
    }
    Ok(())
}

fn copy_new_file<P>(
    source: &Path,
    destination: &Path,
    metadata: &fs::Metadata,
    progress: &mut CopyProgress,
    report_progress: bool,
    on_progress: &mut P,
) -> Result<()>
where
    P: FnMut(CopyProgress),
{
    if path_is_occupied(destination) {
        return Err(name_conflict(destination));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| unsafe_path(destination, "the destination does not have a parent folder"))?;
    let (temporary_path, mut temporary_file) = create_temporary_file(parent)?;
    let copy_result = (|| {
        let mut source_file = File::open(source)
            .map_err(|source_error| Error::io("open the copy source file", source, source_error))?;
        let mut buffer = vec![0_u8; COPY_BUFFER_BYTES];
        loop {
            let count = source_file.read(&mut buffer).map_err(|source_error| {
                Error::io("read the copy source file", source, source_error)
            })?;
            if count == 0 {
                break;
            }
            temporary_file
                .write_all(&buffer[..count])
                .map_err(|source_error| {
                    Error::io("write the temporary copy", &temporary_path, source_error)
                })?;
            progress.bytes_copied = progress.bytes_copied.saturating_add(count as u64);
            if report_progress {
                on_progress(*progress);
            }
        }
        temporary_file.flush().map_err(|source_error| {
            Error::io("flush the temporary copy", &temporary_path, source_error)
        })?;
        fs::set_permissions(&temporary_path, metadata.permissions()).map_err(|source_error| {
            Error::io(
                "preserve the copied file permissions",
                &temporary_path,
                source_error,
            )
        })?;
        temporary_file.sync_all().map_err(|source_error| {
            Error::io("sync the temporary copy", &temporary_path, source_error)
        })
    })();
    drop(temporary_file);
    if let Err(error) = copy_result {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }

    let reservation = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
    {
        Ok(file) => file,
        Err(source_error) if source_error.kind() == io::ErrorKind::AlreadyExists => {
            let _ = fs::remove_file(&temporary_path);
            return Err(name_conflict(destination));
        }
        Err(source_error) => {
            let _ = fs::remove_file(&temporary_path);
            return Err(Error::io(
                "reserve the copy destination",
                destination,
                source_error,
            ));
        }
    };
    drop(reservation);
    if let Err(source_error) = fs::rename(&temporary_path, destination) {
        let _ = fs::remove_file(&temporary_path);
        return Err(Error::io(
            "finish the copied file",
            destination,
            source_error,
        ));
    }
    sync_parent(destination)
}

fn replace_with_copy<P>(
    source: &Path,
    destination: &Path,
    progress: &mut CopyProgress,
    report_progress: bool,
    on_progress: &mut P,
) -> Result<()>
where
    P: FnMut(CopyProgress),
{
    let parent = destination
        .parent()
        .ok_or_else(|| unsafe_path(destination, "the destination does not have a parent folder"))?;
    let staging = create_staging_directory(parent, "replaced")?;
    let backup = staging.join("original");
    fs::rename(destination, &backup)
        .map_err(|source_error| Error::io("stage the replaced item", destination, source_error))?;

    if let Err(copy_error) =
        copy_new_item(source, destination, progress, report_progress, on_progress)
    {
        let _ = remove_owned_item(destination);
        fs::rename(&backup, destination).map_err(|source_error| {
            Error::io(
                "restore the original item after a failed copy",
                destination,
                source_error,
            )
        })?;
        let _ = fs::remove_dir(&staging);
        sync_parent(destination)?;
        return Err(copy_error);
    }

    remove_owned_item(&backup)?;
    fs::remove_dir(&staging)
        .map_err(|source_error| Error::io("remove copy staging data", &staging, source_error))?;
    sync_parent(destination)
}

fn create_staging_directory(parent: &Path, purpose: &str) -> Result<PathBuf> {
    for _ in 0..100 {
        let sequence = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".1537paperstreet.{purpose}-{}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => return Err(Error::io("create a staging folder", path, source)),
        }
    }
    Err(unsafe_path(
        parent,
        "a unique staging folder name could not be allocated",
    ))
}

fn remove_owned_item(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(Error::io("inspect temporary copy data", path, source)),
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)
            .map_err(|source| Error::io("remove temporary copy data", path, source))
    } else {
        fs::remove_file(path)
            .map_err(|source| Error::io("remove temporary copy data", path, source))
    }
}

fn reject_symlink(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    if metadata.file_type().is_symlink() {
        Err(unsafe_path(
            path,
            "symbolic links cannot be copied recursively",
        ))
    } else {
        Ok(())
    }
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
