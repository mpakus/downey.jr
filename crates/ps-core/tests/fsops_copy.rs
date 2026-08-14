use std::fs::{self, File};
use std::path::Path;

use ps_core::fsops::{self, ConflictStrategy, CopyOutcome};

#[test]
fn recursively_copies_a_directory() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("source/nested")).expect("source directory");
    fs::write(root.join("source/readme.md"), b"Read me").expect("readme");
    fs::write(root.join("source/nested/empty.md"), b"").expect("empty file");

    let outcome = fsops::copy(
        &root,
        Path::new("source"),
        Path::new("backup"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect("copied directory");

    assert!(matches!(outcome, CopyOutcome::Copied { .. }));
    assert_eq!(
        fs::read(root.join("backup/readme.md")).expect("copied readme"),
        b"Read me"
    );
    assert_eq!(
        fs::read(root.join("backup/nested/empty.md")).expect("copied empty file"),
        b""
    );
}

#[test]
fn replace_calls_the_snapshot_hook_before_changing_content() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("source.md"), b"New text").expect("source file");
    fs::write(root.join("target.md"), b"Important old text").expect("target file");
    let mut snapshot = Vec::new();

    fsops::copy(
        &root,
        Path::new("source.md"),
        Path::new("target.md"),
        ConflictStrategy::Replace,
        |path| {
            snapshot = fs::read(path).map_err(|source| ps_core::Error::Io {
                action: "read the pre-replace snapshot",
                path: path.to_path_buf(),
                source,
            })?;
            Ok(())
        },
        |_| {},
    )
    .expect("replaced file");

    assert_eq!(snapshot, b"Important old text");
    assert_eq!(
        fs::read(root.join("target.md")).expect("replacement"),
        b"New text"
    );
}

#[test]
fn failed_snapshot_leaves_the_destination_unchanged() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("source.md"), b"New text").expect("source file");
    fs::write(root.join("target.md"), b"Important old text").expect("target file");

    let result = fsops::copy(
        &root,
        Path::new("source.md"),
        Path::new("target.md"),
        ConflictStrategy::Replace,
        |_| {
            Err(ps_core::Error::InvalidProject {
                reason: "snapshot failed",
            })
        },
        |_| {},
    );

    assert!(result.is_err());
    assert_eq!(
        fs::read(root.join("target.md")).expect("preserved destination"),
        b"Important old text"
    );
}

#[test]
fn failed_copy_rolls_the_original_destination_back() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let source = root.join("source.md");
    fs::write(&source, b"New text").expect("source file");
    fs::write(root.join("target.md"), b"Important old text").expect("target file");

    let result = fsops::copy(
        &root,
        Path::new("source.md"),
        Path::new("target.md"),
        ConflictStrategy::Replace,
        |_| {
            fs::remove_file(&source).map_err(|source_error| ps_core::Error::Io {
                action: "simulate a vanished copy source",
                path: source.clone(),
                source: source_error,
            })
        },
        |_| {},
    );

    assert!(result.is_err());
    assert_eq!(
        fs::read(root.join("target.md")).expect("restored destination"),
        b"Important old text"
    );
}

#[test]
fn keep_both_chooses_the_next_available_name() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("source.md"), b"Copy me").expect("source file");
    fs::write(root.join("target.md"), b"Existing").expect("target file");
    fs::write(root.join("target 2.md"), b"Existing too").expect("second target");

    let outcome = fsops::copy(
        &root,
        Path::new("source.md"),
        Path::new("target.md"),
        ConflictStrategy::KeepBoth,
        |_| Ok(()),
        |_| {},
    )
    .expect("kept both files");

    assert!(matches!(
        outcome,
        CopyOutcome::Copied { path } if path.ends_with("target 3.md")
    ));
    assert_eq!(
        fs::read(root.join("target.md")).expect("target"),
        b"Existing"
    );
    assert_eq!(
        fs::read(root.join("target 3.md")).expect("copy"),
        b"Copy me"
    );
}

#[test]
fn skip_preserves_the_existing_destination() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("source.md"), b"Copy me").expect("source file");
    fs::write(root.join("target.md"), b"Existing").expect("target file");

    let outcome = fsops::copy(
        &root,
        Path::new("source.md"),
        Path::new("target.md"),
        ConflictStrategy::Skip,
        |_| Ok(()),
        |_| {},
    )
    .expect("skipped conflict");

    assert!(matches!(outcome, CopyOutcome::Skipped { .. }));
    assert_eq!(
        fs::read(root.join("target.md")).expect("target"),
        b"Existing"
    );
}

#[test]
fn files_over_fifty_megabytes_report_progress() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let source = File::create(root.join("large.bin")).expect("large source");
    source
        .set_len(51 * 1024 * 1024)
        .expect("sparse source size");
    let mut updates = Vec::new();

    fsops::copy(
        &root,
        Path::new("large.bin"),
        Path::new("large-copy.bin"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |progress| updates.push(progress),
    )
    .expect("copied large file");

    let final_update = updates.last().expect("progress update");
    assert_eq!(final_update.bytes_copied, 51 * 1024 * 1024);
    assert_eq!(final_update.total_bytes, 51 * 1024 * 1024);
}

#[test]
fn rejects_copying_a_directory_into_itself() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("source/nested")).expect("source directory");

    let result = fsops::copy(
        &root,
        Path::new("source"),
        Path::new("source/nested/copy"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    );

    assert!(result.is_err());
    assert!(!root.join("source/nested/copy").exists());
}
