use std::fs;
use std::path::Path;

use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};
use ps_core::fsops;

#[test]
fn rejects_paths_that_can_escape_or_cannot_be_file_names() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");

    for invalid in [
        Path::new("../outside.md"),
        Path::new("notes/../../outside.md"),
        Path::new("/tmp/outside.md"),
        Path::new("bad\0name.md"),
        Path::new("."),
    ] {
        assert!(
            fsops::resolve(&root, invalid).is_err(),
            "accepted unsafe path: {}",
            invalid.display()
        );
    }

    let oversized = "a".repeat(256);
    assert!(fsops::resolve(&root, Path::new(&oversized)).is_err());
}

#[test]
fn normalizes_each_name_to_nfc() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");

    let resolved = fsops::resolve(&root, Path::new("и\u{306}.md")).expect("safe normalized path");

    assert_eq!(resolved.file_name(), Some(Path::new("й.md").as_os_str()));
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_that_leave_the_project() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    let outside = temp.path().join("outside");
    fs::create_dir(&root).expect("project directory");
    fs::create_dir(&outside).expect("outside directory");
    symlink(&outside, root.join("escape")).expect("outside symlink");

    assert!(fsops::resolve(&root, Path::new("escape/secret.md")).is_err());
}

#[cfg(unix)]
#[test]
fn permits_symlinks_whose_targets_stay_inside_the_project() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    let notes = root.join("notes");
    fs::create_dir_all(&notes).expect("notes directory");
    symlink(&notes, root.join("alias")).expect("inside symlink");

    let resolved = fsops::resolve(&root, Path::new("alias/draft.md")).expect("inside path");

    assert!(resolved.starts_with(root.canonicalize().expect("canonical project root")));
}

#[test]
fn arbitrary_input_never_resolves_outside_the_project() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let canonical_root = root.canonicalize().expect("canonical project root");
    let mut runner = TestRunner::new(Config {
        cases: 10_000,
        ..Config::default()
    });

    runner
        .run(&any::<String>(), |input| {
            if let Ok(resolved) = fsops::resolve(&root, Path::new(&input)) {
                prop_assert!(resolved.starts_with(&canonical_root));
            }
            Ok(())
        })
        .expect("10,000 path-safety cases");
}
