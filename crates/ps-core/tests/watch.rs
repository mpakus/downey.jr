use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use ps_core::watch::{ProjectWatcher, WatchUpdate};

#[test]
fn coalesces_rapid_changes_after_the_debounce_window() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let watcher = ProjectWatcher::start(&root).expect("project watcher");
    let started = Instant::now();

    fs::write(root.join("note.md"), b"One").expect("first write");
    fs::write(root.join("note.md"), b"Two").expect("second write");
    fs::write(root.join("note.md"), b"Three").expect("third write");

    let update = watcher
        .recv_timeout(Duration::from_secs(3))
        .expect("watch update");
    let WatchUpdate::PathsChanged { paths } = update else {
        panic!("expected changed paths");
    };
    let elapsed = started.elapsed();
    assert!(elapsed >= Duration::from_millis(140));
    assert!(elapsed < Duration::from_secs(1));
    assert_eq!(
        paths
            .iter()
            .filter(|path| path.as_path() == Path::new("note.md"))
            .count(),
        1
    );
}

#[test]
fn expanded_nodes_are_validated_through_the_project_boundary() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("open/nested")).expect("project directory");
    let watcher = ProjectWatcher::start(&root).expect("project watcher");

    watcher
        .set_expanded(&[PathBuf::from(""), PathBuf::from("open")])
        .expect("expanded folders");
    assert!(
        watcher
            .set_expanded(&[PathBuf::from("../outside")])
            .is_err()
    );
}
