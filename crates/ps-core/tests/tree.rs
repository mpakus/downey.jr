use std::fs;
use std::path::Path;

use ps_core::tree::{self, TreeNodeKind};

#[test]
fn reads_only_one_level_with_directories_first_and_natural_sort() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("folder10/nested")).expect("folder ten");
    fs::create_dir(root.join("folder2")).expect("folder two");
    for name in ["file10.md", "file2.md", "file1.md", "file02.md"] {
        fs::write(root.join(name), b"").expect("tree file");
    }
    fs::write(root.join("folder10/nested/deep.md"), b"deep").expect("nested file");

    let nodes = tree::read_dir(&root, Path::new(""), false).expect("tree nodes");
    let names = nodes
        .iter()
        .map(|node| node.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        [
            "folder2",
            "folder10",
            "file1.md",
            "file2.md",
            "file02.md",
            "file10.md"
        ]
    );
    assert!(
        nodes[..2]
            .iter()
            .all(|node| node.kind == TreeNodeKind::Directory)
    );
    assert!(
        nodes[2..]
            .iter()
            .all(|node| node.kind == TreeNodeKind::File)
    );
    assert!(nodes.iter().all(|node| !node.rel_path.ends_with("deep.md")));
}

#[test]
fn hidden_filter_can_be_disabled_explicitly() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("visible.md"), b"").expect("visible file");
    fs::write(root.join(".hidden.md"), b"").expect("hidden file");

    let filtered = tree::read_dir(&root, Path::new(""), false).expect("filtered tree");
    let visible = tree::read_dir(&root, Path::new(""), true).expect("complete tree");

    assert_eq!(filtered.len(), 1);
    assert_eq!(visible.len(), 2);
    assert!(visible.iter().any(|node| node.name == ".hidden.md"));
}

#[cfg(unix)]
#[test]
fn identifies_symlinks_without_following_them() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("target.md"), b"Target").expect("target file");
    symlink(root.join("target.md"), root.join("alias.md")).expect("symlink");

    let nodes = tree::read_dir(&root, Path::new(""), false).expect("tree nodes");
    let alias = nodes
        .iter()
        .find(|node| node.name == "alias.md")
        .expect("alias node");

    assert_eq!(alias.kind, TreeNodeKind::Symlink);
}

#[test]
fn node_at_matches_read_dir_for_the_same_path() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir_all(root.join("inbox")).expect("project directory");
    fs::write(root.join("inbox/note.md"), b"Note").expect("tree file");

    let listed = tree::read_dir(&root, Path::new("inbox"), false).expect("tree nodes");
    let note = listed
        .iter()
        .find(|node| node.name == "note.md")
        .expect("listed note");
    let from_absolute = tree::node_at(&root, &root.join("inbox/note.md")).expect("absolute node");

    assert_eq!(from_absolute, *note);
}

#[test]
fn node_at_rejects_paths_outside_the_project() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(temp.path().join("outside.md"), b"no").expect("outside file");

    assert!(tree::node_at(&root, &temp.path().join("outside.md")).is_err());
    assert!(tree::node_at(&root, &root).is_err());
}

#[cfg(unix)]
#[test]
fn node_at_identifies_symlinks_without_following_them() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    fs::write(root.join("target.md"), b"Target").expect("target file");
    symlink(root.join("target.md"), root.join("alias.md")).expect("symlink");

    let alias = tree::node_at(&root, &root.join("alias.md")).expect("alias node");
    assert_eq!(alias.kind, TreeNodeKind::Symlink);
    assert_eq!(alias.name, "alias.md");
}

#[test]
fn rejects_directories_outside_the_project() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");

    assert!(tree::read_dir(&root, Path::new(".."), false).is_err());
}
