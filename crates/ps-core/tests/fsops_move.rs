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

#[test]
fn skips_an_item_that_is_already_in_the_destination_folder() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("draft.md"), b"Stay").expect("source");

    let outcomes = fsops::move_items(
        &root,
        &[PathBuf::from("draft.md")],
        Path::new(""),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect("skipped in place");

    assert!(matches!(&outcomes[0], MoveOutcome::Skipped { .. }));
    assert_eq!(fs::read(root.join("draft.md")).expect("unchanged"), b"Stay");
}

#[test]
fn rejects_a_file_as_the_move_destination() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("draft.md"), b"Move me").expect("source");
    fs::write(root.join("target.md"), b"Not a folder").expect("file destination");

    let error = fsops::move_items(
        &root,
        &[PathBuf::from("draft.md")],
        Path::new("target.md"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect_err("destination must be a folder");

    assert!(error.to_string().contains("not a folder"));
    assert_eq!(fs::read(root.join("draft.md")).expect("source"), b"Move me");
}

#[test]
fn rejects_moving_a_folder_into_itself() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("chapters/nested")).expect("folder");
    fs::write(root.join("chapters/nested/note.md"), b"Keep").expect("note");

    let error = fsops::move_items(
        &root,
        &[PathBuf::from("chapters")],
        Path::new("chapters"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect_err("cannot move a folder into itself");

    assert!(error.to_string().contains("into itself"));
    assert_eq!(
        fs::read(root.join("chapters/nested/note.md")).expect("kept"),
        b"Keep"
    );
}

#[test]
fn failed_move_snapshot_leaves_both_files() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("archive")).expect("archive");
    fs::write(root.join("draft.md"), b"New").expect("source");
    fs::write(root.join("archive/draft.md"), b"Old").expect("destination");

    let result = fsops::move_items(
        &root,
        &[PathBuf::from("draft.md")],
        Path::new("archive"),
        ConflictStrategy::Replace,
        |_| {
            Err(ps_core::Error::InvalidProject {
                reason: "snapshot failed",
            })
        },
        |_| {},
    );

    assert!(result.is_err());
    assert_eq!(fs::read(root.join("draft.md")).expect("source"), b"New");
    assert_eq!(
        fs::read(root.join("archive/draft.md")).expect("destination"),
        b"Old"
    );
}
