use std::fs;
use std::path::Path;

use ps_core::{Error, fsops};

#[test]
fn creates_normalized_directories_and_empty_files() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");

    let directory = fsops::mkdir(&root, Path::new("и\u{306}")).expect("created directory");
    let file =
        fsops::create_file(&root, Path::new("и\u{306}/draft.md")).expect("created empty file");

    assert_eq!(directory.file_name(), Some(Path::new("й").as_os_str()));
    assert_eq!(file, directory.join("draft.md"));
    assert_eq!(fs::read(file).expect("empty file"), b"");
}

#[test]
fn conflicts_preserve_existing_content_and_suggest_the_next_name() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let canonical_root = root.canonicalize().expect("canonical project directory");
    fs::write(root.join("draft.md"), b"Never overwrite this").expect("existing file");
    fs::write(root.join("draft 2.md"), b"Keep this too").expect("second existing file");

    let error = fsops::create_file(&root, Path::new("draft.md")).expect_err("name conflict");

    match error {
        Error::NameConflict {
            path,
            suggested_name,
        } => {
            assert_eq!(path, canonical_root.join("draft.md"));
            assert_eq!(suggested_name, "draft 3.md");
        }
        other => panic!("unexpected error: {other}"),
    }
    assert_eq!(
        fs::read(root.join("draft.md")).expect("preserved file"),
        b"Never overwrite this"
    );
}

#[test]
fn directory_conflicts_suggest_a_free_directory_name() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fsops::mkdir(&root, Path::new("notes")).expect("notes directory");

    let error = fsops::mkdir(&root, Path::new("notes")).expect_err("name conflict");

    assert!(matches!(
        error,
        Error::NameConflict {
            suggested_name,
            ..
        } if suggested_name == "notes 2"
    ));
}

#[test]
fn conflict_suggestions_remain_valid_at_the_name_length_limit() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let name = format!("{}abc.md", "😀".repeat(62));
    fs::write(root.join(&name), b"existing").expect("maximum-length file name");

    let error = fsops::create_file(&root, Path::new(&name)).expect_err("name conflict");
    let Error::NameConflict { suggested_name, .. } = error else {
        panic!("expected a name conflict");
    };

    assert!(suggested_name.len() <= 255);
    assert!(fsops::resolve(&root, Path::new(&suggested_name)).is_ok());
}

#[test]
fn rename_is_atomic_within_the_project() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let canonical_root = root.canonicalize().expect("canonical project directory");
    fs::write(root.join("draft.md"), b"Important text").expect("draft file");

    let renamed =
        fsops::rename(&root, Path::new("draft.md"), Path::new("final.md")).expect("renamed file");

    assert_eq!(renamed, canonical_root.join("final.md"));
    assert!(!root.join("draft.md").exists());
    assert_eq!(
        fs::read(root.join("final.md")).expect("renamed content"),
        b"Important text"
    );
}

#[test]
fn rename_conflict_never_replaces_either_file() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("draft.md"), b"Draft").expect("draft file");
    fs::write(root.join("final.md"), b"Final").expect("final file");

    let error = fsops::rename(&root, Path::new("draft.md"), Path::new("final.md"))
        .expect_err("name conflict");

    assert!(matches!(
        error,
        Error::NameConflict {
            suggested_name,
            ..
        } if suggested_name == "final 2.md"
    ));
    assert_eq!(fs::read(root.join("draft.md")).expect("draft"), b"Draft");
    assert_eq!(fs::read(root.join("final.md")).expect("final"), b"Final");
}
