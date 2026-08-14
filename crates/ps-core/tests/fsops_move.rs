use std::fs;
use std::path::{Path, PathBuf};

use ps_core::fsops::{self, ConflictStrategy, MoveOutcome};

#[test]
fn moves_multiple_items_into_one_directory() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("archive")).expect("archive directory");
    fs::write(root.join("one.md"), b"One").expect("first source");
    fs::write(root.join("two.md"), b"Two").expect("second source");

    let outcomes = fsops::move_items(
        &root,
        &[PathBuf::from("one.md"), PathBuf::from("two.md")],
        Path::new("archive"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect("moved files");

    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes
            .iter()
            .all(|outcome| matches!(outcome, MoveOutcome::Moved { .. }))
    );
    assert!(!root.join("one.md").exists());
    assert!(!root.join("two.md").exists());
    assert_eq!(
        fs::read(root.join("archive/one.md")).expect("first"),
        b"One"
    );
    assert_eq!(
        fs::read(root.join("archive/two.md")).expect("second"),
        b"Two"
    );
}

#[test]
fn replacement_is_snapshotted_before_same_volume_move() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("archive")).expect("archive directory");
    fs::write(root.join("draft.md"), b"New text").expect("source");
    fs::write(root.join("archive/draft.md"), b"Old text").expect("destination");
    let mut snapshot = Vec::new();

    fsops::move_items(
        &root,
        &[PathBuf::from("draft.md")],
        Path::new("archive"),
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
    .expect("moved replacement");

    assert_eq!(snapshot, b"Old text");
    assert_eq!(
        fs::read(root.join("archive/draft.md")).expect("moved file"),
        b"New text"
    );
}

#[test]
fn keep_both_and_skip_do_not_destroy_existing_files() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("archive")).expect("archive directory");
    fs::write(root.join("draft.md"), b"New text").expect("source");
    fs::write(root.join("archive/draft.md"), b"Old text").expect("destination");

    let kept = fsops::move_items(
        &root,
        &[PathBuf::from("draft.md")],
        Path::new("archive"),
        ConflictStrategy::KeepBoth,
        |_| Ok(()),
        |_| {},
    )
    .expect("kept both");
    assert!(matches!(
        &kept[0],
        MoveOutcome::Moved { path, .. } if path.ends_with("draft 2.md")
    ));
    assert_eq!(
        fs::read(root.join("archive/draft.md")).expect("old"),
        b"Old text"
    );

    fs::write(root.join("draft.md"), b"Another text").expect("second source");
    let skipped = fsops::move_items(
        &root,
        &[PathBuf::from("draft.md")],
        Path::new("archive"),
        ConflictStrategy::Skip,
        |_| Ok(()),
        |_| {},
    )
    .expect("skipped move");
    assert!(matches!(&skipped[0], MoveOutcome::Skipped { .. }));
    assert_eq!(
        fs::read(root.join("draft.md")).expect("source"),
        b"Another text"
    );
    assert_eq!(
        fs::read(root.join("archive/draft.md")).expect("old"),
        b"Old text"
    );
}
