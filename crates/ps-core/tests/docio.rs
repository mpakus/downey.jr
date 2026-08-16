use std::fs;
use std::path::Path;

use ps_core::docio::{self, DocumentEncoding, LineEnding, SOURCE_ONLY_BYTES};

fn project_file(name: &str, bytes: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempfile::tempdir().expect("temporary directory");
    let root = temp.path().join("project");
    fs::create_dir(&root).expect("project directory");
    let path = root.join(name);
    fs::write(&path, bytes).expect("document");
    (temp, root)
}

#[test]
fn reads_lf_and_crlf_with_and_without_a_bom() {
    let cases: &[(&[u8], &str, LineEnding, bool)] = &[
        (b"hello\n", "hello\n", LineEnding::Lf, false),
        (b"hello\r\n", "hello\r\n", LineEnding::CrLf, false),
        (b"\xEF\xBB\xBFhello\n", "hello\n", LineEnding::Lf, true),
        (
            b"\xEF\xBB\xBFhello\r\n",
            "hello\r\n",
            LineEnding::CrLf,
            true,
        ),
        (b"hello", "hello", LineEnding::Lf, false),
        (b"hello\r\nworld", "hello\r\nworld", LineEnding::CrLf, false),
        (b"\xEF\xBB\xBFhello", "hello", LineEnding::Lf, true),
        (
            b"\xEF\xBB\xBFhello\r\nworld",
            "hello\r\nworld",
            LineEnding::CrLf,
            true,
        ),
    ];

    for (bytes, text, eol, bom) in cases {
        let (_temp, root) = project_file("note.md", bytes);
        let loaded = docio::read_doc(&root, Path::new("note.md")).expect("read");
        assert_eq!(loaded.source.text, *text);
        assert_eq!(loaded.source.eol, *eol);
        assert_eq!(loaded.source.bom, *bom);
        assert_eq!(loaded.source.encoding, DocumentEncoding::Utf8);
        assert!(loaded.source.writable);
        assert!(!loaded.source_only);
    }
}

#[test]
fn binary_files_are_read_only_with_empty_text() {
    let (_temp, root) = project_file("note.md", &[0xff, 0xfe, 0x00]);
    let loaded = docio::read_doc(&root, Path::new("note.md")).expect("read");
    assert_eq!(loaded.source.text, "");
    assert_eq!(loaded.source.encoding, DocumentEncoding::Binary);
    assert!(!loaded.source.writable);
    assert!(loaded.source_only);
    assert!(loaded.source.readonly_reason.is_some());
}

#[test]
fn files_over_eight_megabytes_open_as_source_only() {
    let mut bytes = vec![b'a'; SOURCE_ONLY_BYTES as usize + 1];
    bytes.push(b'\n');
    let (_temp, root) = project_file("huge.md", &bytes);
    let loaded = docio::read_doc(&root, Path::new("huge.md")).expect("read");
    assert_eq!(loaded.source.encoding, DocumentEncoding::Utf8);
    assert!(!loaded.source.writable);
    assert!(loaded.source_only);
    assert_eq!(loaded.size, SOURCE_ONLY_BYTES + 2);
}

#[test]
fn rejects_paths_outside_the_project() {
    let (_temp, root) = project_file("note.md", b"hello\n");
    assert!(docio::read_doc(&root, Path::new("..")).is_err());
}

#[cfg(unix)]
#[test]
fn unwritable_utf8_files_are_marked_read_only() {
    use std::os::unix::fs::PermissionsExt;

    let (_temp, root) = project_file("note.md", b"hello\n");
    let path = root.join("note.md");
    let mut permissions = fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o444);
    fs::set_permissions(&path, permissions).expect("chmod");

    let loaded = docio::read_doc(&root, Path::new("note.md")).expect("read");
    assert!(!loaded.source.writable);
    assert!(!loaded.source_only);
    assert!(loaded.source.readonly_reason.is_some());
}
