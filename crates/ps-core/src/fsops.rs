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

/// The result of moving one project item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MoveOutcome {
    /// The item was moved to this path.
    Moved {
        /// The original project-relative path.
        from: PathBuf,
        /// The final absolute destination path.
        path: PathBuf,
    },
    /// The item remained at its original path because the destination existed.
    Skipped {
        /// The unchanged project-relative path.
        from: PathBuf,
        /// The conflicting absolute destination path.
        path: PathBuf,
    },
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

/// Moves project items into one destination directory.
///
/// Items on the same volume use an atomic rename. Cross-volume items are staged,
/// copied with the same safety guarantees as [`copy`], and restored to their
/// original paths if copying fails. `before_replace` must persist a `pre_replace`
/// snapshot before an existing destination can be changed.
pub fn move_items<B, P>(
    project_root: &Path,
    from: &[PathBuf],
    to_dir: &Path,
    conflict: ConflictStrategy,
    mut before_replace: B,
    mut on_progress: P,
) -> Result<Vec<MoveOutcome>>
where
    B: FnMut(&Path) -> Result<()>,
    P: FnMut(CopyProgress),
{
    let destination_dir = resolve(project_root, to_dir)?
        .canonicalize()
        .map_err(|source| Error::io("open the move destination folder", to_dir, source))?;
    if !destination_dir.is_dir() {
        return Err(unsafe_path(to_dir, "the move destination is not a folder"));
    }

    let mut outcomes = Vec::with_capacity(from.len());
    for rel_path in from {
        let source = resolve_child(project_root, rel_path)?;
        let file_name = source
            .file_name()
            .ok_or_else(|| unsafe_path(rel_path, "the move source does not have a file name"))?;
        let destination = destination_dir.join(file_name);
        if source == destination {
            outcomes.push(MoveOutcome::Skipped {
                from: rel_path.clone(),
                path: destination,
            });
            continue;
        }

        let metadata = fs::symlink_metadata(&source)
            .map_err(|source_error| Error::io("open the move source", &source, source_error))?;
        let canonical_source = source
            .canonicalize()
            .map_err(|source_error| Error::io("open the move source", &source, source_error))?;
        if metadata.is_dir() && destination_dir.starts_with(&canonical_source) {
            return Err(unsafe_path(
                to_dir,
                "a folder cannot be moved into itself or one of its children",
            ));
        }

        let outcome = if same_volume(&source, &destination_dir)? {
            move_one_on_same_volume(
                rel_path,
                &source,
                &destination,
                conflict,
                &mut before_replace,
            )?
        } else {
            reject_symlink(&source, &metadata)?;
            let total_bytes = measure_item(&source)?;
            let report_progress = total_bytes > PROGRESS_THRESHOLD_BYTES;
            let mut progress = CopyProgress {
                bytes_copied: 0,
                total_bytes,
            };
            move_one_across_volumes(
                &source,
                &destination,
                conflict,
                &mut before_replace,
                &mut progress,
                report_progress,
                &mut on_progress,
            )?
            .with_from(rel_path.clone())
        };
        outcomes.push(outcome);
    }
    Ok(outcomes)
}

impl MoveOutcome {
    fn with_from(self, from: PathBuf) -> Self {
        match self {
            Self::Moved { path, .. } => Self::Moved { from, path },
            Self::Skipped { path, .. } => Self::Skipped { from, path },
        }
    }
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

    copy_directory_entries(source, destination, progress, report_progress, on_progress)?;
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
        restore_destination_backup(Some(&(staging, backup)), destination)?;
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

#[cfg(unix)]
fn same_volume(source: &Path, destination_dir: &Path) -> Result<bool> {
    use std::os::unix::fs::MetadataExt;

    let source_device = fs::metadata(source)
        .map_err(|error| Error::io("inspect the move source volume", source, error))?
        .dev();
    let destination_device = fs::metadata(destination_dir)
        .map_err(|error| {
            Error::io(
                "inspect the move destination volume",
                destination_dir,
                error,
            )
        })?
        .dev();
    Ok(source_device == destination_device)
}

#[cfg(not(unix))]
fn same_volume(_source: &Path, _destination_dir: &Path) -> Result<bool> {
    Ok(true)
}

fn move_one_on_same_volume<B>(
    from: &Path,
    source: &Path,
    destination: &Path,
    conflict: ConflictStrategy,
    before_replace: &mut B,
) -> Result<MoveOutcome>
where
    B: FnMut(&Path) -> Result<()>,
{
    let mut destination = destination.to_path_buf();
    let destination_exists = path_is_occupied(&destination);
    if destination_exists {
        match conflict {
            ConflictStrategy::Skip => {
                return Ok(MoveOutcome::Skipped {
                    from: from.to_path_buf(),
                    path: destination,
                });
            }
            ConflictStrategy::KeepBoth => {
                let suggested = next_available_name(&destination);
                let parent = destination.parent().ok_or_else(|| {
                    unsafe_path(
                        &destination,
                        "the move destination does not have a parent folder",
                    )
                })?;
                destination = parent.join(suggested);
            }
            ConflictStrategy::Replace => {
                before_replace(&destination)?;
                replace_with_rename(source, &destination)?;
                return Ok(MoveOutcome::Moved {
                    from: from.to_path_buf(),
                    path: destination,
                });
            }
        }
    }

    if path_is_occupied(&destination) {
        return Err(name_conflict(&destination));
    }
    fs::rename(source, &destination)
        .map_err(|error| Error::io("move the file or folder", source, error))?;
    sync_parent(source)?;
    if source.parent() != destination.parent() {
        sync_parent(&destination)?;
    }
    Ok(MoveOutcome::Moved {
        from: from.to_path_buf(),
        path: destination,
    })
}

fn replace_with_rename(source: &Path, destination: &Path) -> Result<()> {
    let parent = destination.parent().ok_or_else(|| {
        unsafe_path(
            destination,
            "the move destination does not have a parent folder",
        )
    })?;
    let staging = create_staging_directory(parent, "move-replaced")?;
    let backup = staging.join("original");
    fs::rename(destination, &backup)
        .map_err(|error| Error::io("stage the replaced move destination", destination, error))?;
    if let Err(error) = fs::rename(source, destination) {
        fs::rename(&backup, destination).map_err(|restore_error| {
            Error::io(
                "restore the move destination after a failed rename",
                destination,
                restore_error,
            )
        })?;
        let _ = fs::remove_dir(&staging);
        return Err(Error::io("move the file or folder", source, error));
    }

    remove_owned_item(&backup)?;
    fs::remove_dir(&staging)
        .map_err(|error| Error::io("remove move staging data", &staging, error))?;
    sync_parent(source)?;
    if source.parent() != destination.parent() {
        sync_parent(destination)?;
    }
    Ok(())
}

fn move_one_across_volumes<B, P>(
    source: &Path,
    destination: &Path,
    conflict: ConflictStrategy,
    before_replace: &mut B,
    progress: &mut CopyProgress,
    report_progress: bool,
    on_progress: &mut P,
) -> Result<MoveOutcome>
where
    B: FnMut(&Path) -> Result<()>,
    P: FnMut(CopyProgress),
{
    let mut destination = destination.to_path_buf();
    let destination_exists = path_is_occupied(&destination);
    if destination_exists {
        match conflict {
            ConflictStrategy::Skip => {
                return Ok(MoveOutcome::Skipped {
                    from: source.to_path_buf(),
                    path: destination,
                });
            }
            ConflictStrategy::KeepBoth => {
                let suggested = next_available_name(&destination);
                let parent = destination.parent().ok_or_else(|| {
                    unsafe_path(
                        &destination,
                        "the move destination does not have a parent folder",
                    )
                })?;
                destination = parent.join(suggested);
            }
            ConflictStrategy::Replace => before_replace(&destination)?,
        }
    }

    let source_parent = source
        .parent()
        .ok_or_else(|| unsafe_path(source, "the move source does not have a parent folder"))?;
    let source_staging = create_staging_directory(source_parent, "move-source")?;
    let staged_source = source_staging.join("original");
    fs::rename(source, &staged_source)
        .map_err(|error| Error::io("stage the cross-volume move source", source, error))?;

    let destination_backup = if destination_exists && conflict == ConflictStrategy::Replace {
        match stage_existing_item(&destination, "move-destination") {
            Ok(staged) => Some(staged),
            Err(error) => {
                restore_staged_source(&staged_source, source, &source_staging)?;
                return Err(error);
            }
        }
    } else {
        None
    };

    if let Err(copy_error) = copy_new_item(
        &staged_source,
        &destination,
        progress,
        report_progress,
        on_progress,
    ) {
        restore_destination_backup(destination_backup.as_ref(), &destination)?;
        restore_staged_source(&staged_source, source, &source_staging)?;
        return Err(copy_error);
    }

    if let Err(delete_error) = remove_owned_item(&staged_source) {
        restore_staged_source(&staged_source, source, &source_staging)?;
        return Err(delete_error);
    }
    fs::remove_dir(&source_staging)
        .map_err(|error| Error::io("remove move source staging data", &source_staging, error))?;
    remove_destination_backup(destination_backup.as_ref())?;
    sync_parent(&destination)?;

    Ok(MoveOutcome::Moved {
        from: source.to_path_buf(),
        path: destination,
    })
}

fn stage_existing_item(path: &Path, purpose: &str) -> Result<(PathBuf, PathBuf)> {
    let parent = path
        .parent()
        .ok_or_else(|| unsafe_path(path, "the staged item does not have a parent folder"))?;
    let staging = create_staging_directory(parent, purpose)?;
    let backup = staging.join("original");
    if let Err(error) = fs::rename(path, &backup) {
        let _ = fs::remove_dir(&staging);
        return Err(Error::io("stage the existing item", path, error));
    }
    Ok((staging, backup))
}

fn restore_destination_backup(
    staged: Option<&(PathBuf, PathBuf)>,
    destination: &Path,
) -> Result<()> {
    let Some((staging, backup)) = staged else {
        return Ok(());
    };
    let failed_copy = staging.join("failed-copy");
    if path_is_occupied(destination) {
        fs::rename(destination, &failed_copy).map_err(|error| {
            Error::io(
                "preserve a failed or externally created destination",
                destination,
                error,
            )
        })?;
    }
    fs::rename(backup, destination)
        .map_err(|error| Error::io("restore the original move destination", destination, error))?;
    if path_is_occupied(&failed_copy) {
        Ok(())
    } else {
        fs::remove_dir(staging)
            .map_err(|error| Error::io("remove move destination staging data", staging, error))
    }
}

fn remove_destination_backup(staged: Option<&(PathBuf, PathBuf)>) -> Result<()> {
    let Some((staging, backup)) = staged else {
        return Ok(());
    };
    remove_owned_item(backup)?;
    fs::remove_dir(staging)
        .map_err(|error| Error::io("remove move destination staging data", staging, error))
}

fn restore_staged_source(staged: &Path, source: &Path, staging: &Path) -> Result<()> {
    fs::rename(staged, source)
        .map_err(|error| Error::io("restore the cross-volume move source", source, error))?;
    fs::remove_dir(staging)
        .map_err(|error| Error::io("remove move source staging data", staging, error))
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

#[cfg(all(test, unix))]
mod move_tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn cross_volume_move_copies_then_removes_the_staged_source() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let root = temp.path();
        let source_dir = root.join("source");
        let destination_dir = root.join("destination");
        fs::create_dir(&source_dir).expect("source directory");
        fs::create_dir(&destination_dir).expect("destination directory");
        let source = source_dir.join("draft.md");
        let destination = destination_dir.join("draft.md");
        fs::write(&source, b"Move me").expect("source file");
        let mut progress = CopyProgress {
            bytes_copied: 0,
            total_bytes: 7,
        };

        let outcome = move_one_across_volumes(
            &source,
            &destination,
            ConflictStrategy::Replace,
            &mut |_| Ok(()),
            &mut progress,
            false,
            &mut |_| {},
        )
        .expect("cross-volume move");

        assert!(matches!(outcome, MoveOutcome::Moved { .. }));
        assert!(!source.exists());
        assert_eq!(fs::read(destination).expect("destination"), b"Move me");
    }

    #[test]
    fn cross_volume_copy_failure_restores_source_and_destination() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let root = temp.path();
        let source_dir = root.join("source");
        let destination_dir = root.join("destination");
        fs::create_dir(&source_dir).expect("source directory");
        fs::create_dir(&destination_dir).expect("destination directory");
        let source = source_dir.join("draft.md");
        let destination = destination_dir.join("draft.md");
        fs::write(&source, b"New text").expect("source file");
        fs::write(&destination, b"Important old text").expect("destination file");
        let mut progress = CopyProgress {
            bytes_copied: 0,
            total_bytes: 8,
        };

        let result = move_one_across_volumes(
            &source,
            &destination,
            ConflictStrategy::Replace,
            &mut |_| {
                fs::set_permissions(&source, fs::Permissions::from_mode(0o000)).map_err(
                    |source_error| Error::io("make the source unreadable", &source, source_error),
                )
            },
            &mut progress,
            false,
            &mut |_| {},
        );

        fs::set_permissions(&source, fs::Permissions::from_mode(0o644))
            .expect("restore source permissions");
        assert!(result.is_err());
        assert_eq!(fs::read(&source).expect("source remains"), b"New text");
        assert_eq!(
            fs::read(&destination).expect("destination restored"),
            b"Important old text"
        );
    }

    #[test]
    fn cross_volume_race_never_deletes_an_external_destination() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let root = temp.path();
        let source_dir = root.join("source");
        let destination_dir = root.join("destination");
        fs::create_dir(&source_dir).expect("source directory");
        fs::create_dir(&destination_dir).expect("destination directory");
        let source = source_dir.join("large.bin");
        let destination = destination_dir.join("large.bin");
        File::create(&source)
            .expect("large source")
            .set_len(51 * 1024 * 1024)
            .expect("sparse source size");
        let mut progress = CopyProgress {
            bytes_copied: 0,
            total_bytes: 51 * 1024 * 1024,
        };
        let mut inserted_external_file = false;

        let result = move_one_across_volumes(
            &source,
            &destination,
            ConflictStrategy::Replace,
            &mut |_| Ok(()),
            &mut progress,
            true,
            &mut |_| {
                if !inserted_external_file {
                    fs::write(&destination, b"External text").expect("external destination");
                    inserted_external_file = true;
                }
            },
        );

        assert!(result.is_err());
        assert!(source.exists());
        assert_eq!(
            fs::read(&destination).expect("external destination survives"),
            b"External text"
        );
    }
}
