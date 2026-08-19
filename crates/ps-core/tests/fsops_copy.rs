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
fn replacement_race_preserves_external_content_and_restores_the_original() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    File::create(root.join("large.bin"))
        .expect("large source")
        .set_len(51 * 1024 * 1024)
        .expect("sparse source size");
    fs::write(root.join("target.bin"), b"Original text").expect("original destination");
    let mut inserted_external_file = false;

    let result = fsops::copy(
        &root,
        Path::new("large.bin"),
        Path::new("target.bin"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {
            if !inserted_external_file {
                fs::write(root.join("target.bin"), b"External text").expect("external destination");
                inserted_external_file = true;
            }
        },
    );

    assert!(result.is_err());
    assert_eq!(
        fs::read(root.join("target.bin")).expect("original restored"),
        b"Original text"
    );
    let recovery = fs::read_dir(&root)
        .expect("project entries")
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(".1537paperstreet.replaced-"))
        })
        .expect("failed copy recovery folder");
    assert_eq!(
        fs::read(recovery.join("failed-copy")).expect("external content preserved"),
        b"External text"
    );
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

#[test]
fn imports_an_outside_file_into_the_project() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    let outside = temp.path().join("outside");
    fs::create_dir_all(root.join("inbox")).expect("project directory");
    fs::create_dir(&outside).expect("outside directory");
    fs::write(outside.join("photo.png"), b"png-bytes").expect("outside file");

    let outcomes = fsops::import_into(
        &root,
        Path::new("inbox"),
        &[outside.join("photo.png")],
        ConflictStrategy::KeepBoth,
        |_| Ok(()),
        |_| {},
    )
    .expect("imported");

    assert_eq!(outcomes.len(), 1);
    assert_eq!(
        fs::read(root.join("inbox/photo.png")).expect("imported file"),
        b"png-bytes"
    );
    assert_eq!(
        fs::read(outside.join("photo.png")).expect("source left in place"),
        b"png-bytes"
    );
}

#[test]
fn import_keep_both_does_not_replace_an_existing_name() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    let outside = temp.path().join("outside");
    fs::create_dir(&root).expect("project directory");
    fs::create_dir(&outside).expect("outside directory");
    fs::write(root.join("note.md"), b"inside").expect("existing");
    fs::write(outside.join("note.md"), b"imported").expect("outside file");

    fsops::import_into(
        &root,
        Path::new(""),
        &[outside.join("note.md")],
        ConflictStrategy::KeepBoth,
        |_| Ok(()),
        |_| {},
    )
    .expect("imported beside existing");

    assert_eq!(fs::read(root.join("note.md")).expect("original"), b"inside");
    assert_eq!(
        fs::read(root.join("note 2.md")).expect("kept both"),
        b"imported"
    );
}

#[test]
fn import_rejects_a_path_outside_the_chosen_folder_via_parent_escape() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let secret = temp.path().join("secret.txt");
    fs::write(&secret, b"nope").expect("secret");

    let result = fsops::import_into(
        &root,
        Path::new(".."),
        &[secret],
        ConflictStrategy::KeepBoth,
        |_| Ok(()),
        |_| {},
    );

    assert!(result.is_err());
    assert!(!root.join("secret.txt").exists());
}

#[test]
fn copy_cannot_replace_the_source_with_itself() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("note.md"), b"Same").expect("source");

    let error = fsops::copy(
        &root,
        Path::new("note.md"),
        Path::new("note.md"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect_err("self-replace");

    assert!(error.to_string().contains("cannot replace itself"));
    assert_eq!(fs::read(root.join("note.md")).expect("unchanged"), b"Same");
}

#[test]
fn copy_rejects_replacing_a_folder_with_one_of_its_children() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("source/nested")).expect("nested");
    fs::write(root.join("source/nested/note.md"), b"Keep").expect("note");

    let error = fsops::copy(
        &root,
        Path::new("source/nested"),
        Path::new("source"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect_err("child cannot replace parent folder");

    assert!(error.to_string().contains("into itself"));
    assert_eq!(
        fs::read(root.join("source/nested/note.md")).expect("kept"),
        b"Keep"
    );
}

#[test]
fn import_skips_an_existing_name() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    let outside = temp.path().join("outside");
    fs::create_dir(&root).expect("project directory");
    fs::create_dir(&outside).expect("outside directory");
    fs::write(root.join("note.md"), b"inside").expect("existing");
    fs::write(outside.join("note.md"), b"imported").expect("outside file");

    let outcomes = fsops::import_into(
        &root,
        Path::new(""),
        &[outside.join("note.md")],
        ConflictStrategy::Skip,
        |_| Ok(()),
        |_| {},
    )
    .expect("skipped");

    assert!(matches!(outcomes[0], CopyOutcome::Skipped { .. }));
    assert_eq!(fs::read(root.join("note.md")).expect("original"), b"inside");
    assert!(!root.join("note 2.md").exists());
}

#[test]
fn import_replace_snapshots_then_overwrites() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    let outside = temp.path().join("outside");
    fs::create_dir(&root).expect("project directory");
    fs::create_dir(&outside).expect("outside directory");
    fs::write(root.join("note.md"), b"inside").expect("existing");
    fs::write(outside.join("note.md"), b"imported").expect("outside file");
    let mut snapshot = Vec::new();

    fsops::import_into(
        &root,
        Path::new(""),
        &[outside.join("note.md")],
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
    .expect("replaced");

    assert_eq!(snapshot, b"inside");
    assert_eq!(
        fs::read(root.join("note.md")).expect("replaced file"),
        b"imported"
    );
}

#[test]
fn import_replace_skips_when_the_source_is_already_the_destination() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("note.md"), b"same").expect("existing");

    let outcomes = fsops::import_into(
        &root,
        Path::new(""),
        &[root.join("note.md")],
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect("same item");

    assert!(matches!(outcomes[0], CopyOutcome::Skipped { .. }));
    assert_eq!(fs::read(root.join("note.md")).expect("unchanged"), b"same");
}

#[test]
fn import_rejects_a_file_as_the_destination_folder() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    let outside = temp.path().join("photo.png");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("note.md"), b"not a folder").expect("file destination");
    fs::write(&outside, b"png").expect("outside file");

    let error = fsops::import_into(
        &root,
        Path::new("note.md"),
        &[outside],
        ConflictStrategy::KeepBoth,
        |_| Ok(()),
        |_| {},
    )
    .expect_err("destination must be a folder");

    assert!(error.to_string().contains("not a folder"));
}

#[test]
fn import_rejects_a_folder_copied_into_itself() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("inbox/nested")).expect("inbox");
    fs::write(root.join("inbox/nested/note.md"), b"Keep").expect("note");

    let error = fsops::import_into(
        &root,
        Path::new("inbox"),
        &[root.join("inbox")],
        ConflictStrategy::KeepBoth,
        |_| Ok(()),
        |_| {},
    )
    .expect_err("cannot import a folder into itself");

    assert!(error.to_string().contains("into itself"));
    assert_eq!(
        fs::read(root.join("inbox/nested/note.md")).expect("kept"),
        b"Keep"
    );
}

#[cfg(unix)]
#[test]
fn copy_and_import_reject_symbolic_links() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("target.md"), b"Target").expect("target");
    symlink(root.join("target.md"), root.join("alias.md")).expect("symlink");

    assert!(
        fsops::copy(
            &root,
            Path::new("alias.md"),
            Path::new("copy.md"),
            ConflictStrategy::Replace,
            |_| Ok(()),
            |_| {},
        )
        .is_err()
    );
    assert!(
        fsops::import_into(
            &root,
            Path::new(""),
            &[root.join("alias.md")],
            ConflictStrategy::KeepBoth,
            |_| Ok(()),
            |_| {},
        )
        .is_err()
    );
    assert!(!root.join("copy.md").exists());
}

#[cfg(unix)]
#[test]
fn copy_rejects_a_socket_inside_a_folder() {
    use std::os::unix::net::UnixListener;

    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("source")).expect("source");
    let _listener = UnixListener::bind(root.join("source/sock")).expect("socket");

    let error = fsops::copy(
        &root,
        Path::new("source"),
        Path::new("backup"),
        ConflictStrategy::Replace,
        |_| Ok(()),
        |_| {},
    )
    .expect_err("socket is not a copyable item");

    assert!(error.to_string().contains("only files and folders"));
    assert!(!root.join("backup").exists());
}

#[cfg(unix)]
#[test]
fn import_rejects_a_nul_byte_in_the_source_path() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;

    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let source = PathBuf::from(OsString::from_vec(b"bad\0name.md".to_vec()));

    let error = fsops::import_into(
        &root,
        Path::new(""),
        &[source],
        ConflictStrategy::KeepBoth,
        |_| Ok(()),
        |_| {},
    )
    .expect_err("NUL is rejected");

    assert!(error.to_string().contains("NUL"));
}
