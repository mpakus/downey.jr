use std::fs;
use std::path::{Path, PathBuf};

use ps_core::Error;
use ps_core::fsops::{self, ConflictStrategy, CopyOutcome};

#[test]
fn conflicts_never_overwrite_existing_text() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("draft.md"), b"Important original").expect("existing file");

    let error = fsops::create_file(&root, Path::new("draft.md")).expect_err("name conflict");

    assert!(matches!(error, Error::NameConflict { .. }));
    assert_eq!(
        fs::read(root.join("draft.md")).expect("original file"),
        b"Important original"
    );
}

#[test]
fn nfd_cyrillic_and_nfc_paths_refer_to_the_same_normalized_name() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");

    let directory = fsops::mkdir(&root, Path::new("и\u{306}")).expect("NFD directory");
    let file = fsops::create_file(&root, Path::new("й/заметка.md")).expect("NFC file");

    assert_eq!(directory.file_name(), Some(Path::new("й").as_os_str()));
    assert_eq!(file, directory.join("заметка.md"));
}

#[test]
fn emoji_names_survive_copy_without_byte_changes() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let bytes = [0_u8, 1, 2, 127, 128, 254, 255];
    fs::write(root.join("📝.md"), bytes).expect("emoji source");

    let outcome = fsops::copy(
        &root,
        Path::new("📝.md"),
        Path::new("📚.md"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect("emoji copy");

    assert!(matches!(outcome, CopyOutcome::Copied { .. }));
    assert_eq!(fs::read(root.join("📚.md")).expect("emoji copy"), bytes);
}

#[test]
fn creates_and_resolves_twenty_directory_levels() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let mut relative = PathBuf::new();
    for level in 1..=20 {
        relative.push(format!("level{level}"));
        fsops::mkdir(&root, &relative).expect("nested directory");
    }

    let file = fsops::create_file(&root, &relative.join("deep.md")).expect("deep file");

    assert_eq!(
        file.strip_prefix(root.canonicalize().expect("root"))
            .expect("relative")
            .components()
            .count(),
        21
    );
}

#[test]
fn zero_byte_files_copy_as_zero_byte_files() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fsops::create_file(&root, Path::new("empty.md")).expect("empty file");

    fsops::copy(
        &root,
        Path::new("empty.md"),
        Path::new("empty-copy.md"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect("empty copy");

    assert_eq!(
        fs::metadata(root.join("empty.md"))
            .expect("source metadata")
            .len(),
        0
    );
    assert_eq!(
        fs::metadata(root.join("empty-copy.md"))
            .expect("copy metadata")
            .len(),
        0
    );
}

#[cfg(unix)]
#[test]
fn read_only_permissions_are_preserved_and_block_new_writes() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    let locked = root.join("locked");
    fs::create_dir_all(&locked).expect("locked directory");
    fs::write(root.join("readonly.md"), b"Read only").expect("read-only source");
    fs::set_permissions(root.join("readonly.md"), fs::Permissions::from_mode(0o444))
        .expect("read-only file permissions");

    fsops::copy(
        &root,
        Path::new("readonly.md"),
        Path::new("readonly-copy.md"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect("read-only copy");
    assert_eq!(
        fs::metadata(root.join("readonly-copy.md"))
            .expect("copy metadata")
            .permissions()
            .mode()
            & 0o777,
        0o444
    );

    fs::set_permissions(&locked, fs::Permissions::from_mode(0o555))
        .expect("locked directory permissions");
    let result = fsops::create_file(&root, Path::new("locked/new.md"));
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755))
        .expect("restore directory permissions");
    assert!(result.is_err());
}
