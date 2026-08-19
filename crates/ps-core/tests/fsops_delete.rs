use std::fs;
use std::path::PathBuf;

use ps_core::fsops;

#[test]
fn failed_trash_snapshot_keeps_every_item() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("one.md"), b"One").expect("first file");
    fs::write(root.join("two.md"), b"Two").expect("second file");

    let result = fsops::trash(
        &root,
        &[PathBuf::from("one.md"), PathBuf::from("two.md")],
        |_| {
            Err(ps_core::Error::InvalidProject {
                reason: "snapshot failed",
            })
        },
    );

    assert!(result.is_err());
    assert_eq!(
        fs::read(root.join("one.md")).expect("first preserved"),
        b"One"
    );
    assert_eq!(
        fs::read(root.join("two.md")).expect("second preserved"),
        b"Two"
    );
}

#[test]
fn permanent_delete_snapshots_all_items_before_deleting_any() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("folder")).expect("project directory");
    fs::write(root.join("one.md"), b"One").expect("first file");
    fs::write(root.join("folder/two.md"), b"Two").expect("nested file");
    let mut snapshots = Vec::new();

    fsops::permanently_delete(
        &root,
        &[PathBuf::from("one.md"), PathBuf::from("folder")],
        |path| {
            snapshots.push(path.to_path_buf());
            Ok(())
        },
    )
    .expect("permanent delete");

    assert_eq!(snapshots.len(), 2);
    assert!(!root.join("one.md").exists());
    assert!(!root.join("folder").exists());
}

#[test]
fn failed_permanent_snapshot_keeps_the_whole_batch() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("one.md"), b"One").expect("first file");
    fs::write(root.join("two.md"), b"Two").expect("second file");
    let mut snapshots = 0;

    let result = fsops::permanently_delete(
        &root,
        &[PathBuf::from("one.md"), PathBuf::from("two.md")],
        |_| {
            snapshots += 1;
            if snapshots == 2 {
                Err(ps_core::Error::InvalidProject {
                    reason: "snapshot failed",
                })
            } else {
                Ok(())
            }
        },
    );

    assert!(result.is_err());
    assert_eq!(
        fs::read(root.join("one.md")).expect("first preserved"),
        b"One"
    );
    assert_eq!(
        fs::read(root.join("two.md")).expect("second preserved"),
        b"Two"
    );
}

#[cfg(unix)]
#[test]
fn permanent_delete_removes_a_symlink_not_its_target() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("target.md"), b"Keep me").expect("target file");
    symlink(root.join("target.md"), root.join("alias.md")).expect("inside symlink");

    fsops::permanently_delete(&root, &[PathBuf::from("alias.md")], |_| Ok(()))
        .expect("delete symlink");

    assert!(!root.join("alias.md").exists());
    assert_eq!(
        fs::read(root.join("target.md")).expect("target"),
        b"Keep me"
    );
}

#[test]
fn missing_items_are_rejected_before_any_delete() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("one.md"), b"One").expect("file");

    assert!(
        fsops::permanently_delete(
            &root,
            &[PathBuf::from("one.md"), PathBuf::from("missing.md")],
            |_| Ok(()),
        )
        .is_err()
    );
    assert_eq!(fs::read(root.join("one.md")).expect("kept"), b"One");
    assert!(fsops::trash(&root, &[PathBuf::from("missing.md")], |_| Ok(())).is_err());
}
