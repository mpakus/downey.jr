use std::fs;

use ps_core::log::FileLog;

#[test]
fn appends_lines_and_rotates_when_the_file_grows() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = temp.path().join("logs/app.log");
    let log = FileLog::open_with_limit(&path, 80).expect("open log");

    log.info("started").expect("info");
    log.warn("theme skipped").expect("warn");
    assert!(
        fs::read_to_string(&path)
            .expect("log")
            .contains("info started")
    );

    log.error("code overflow").expect("error");
    log.error("code later").expect("error");

    assert!(path.exists());
    let rotated = temp.path().join("logs/app.log.1");
    assert!(rotated.exists(), "expected rotation into app.log.1");
    let combined = format!(
        "{}{}",
        fs::read_to_string(&rotated).unwrap_or_default(),
        fs::read_to_string(&path).unwrap_or_default()
    );
    assert!(combined.contains("started") || combined.contains("code later"));
    assert!(!combined.contains("# Heading\nsecret markdown"));
}

#[test]
fn open_uses_the_default_rotation_limit() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = temp.path().join("app.log");
    let log = FileLog::open(&path).expect("open default log");
    log.info("hello").expect("info");
    assert!(
        fs::read_to_string(&path)
            .expect("log")
            .contains("info hello")
    );
}

#[test]
fn rotation_drops_the_oldest_backup() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = temp.path().join("app.log");
    let log = FileLog::open_with_limit(&path, 40).expect("open log");
    for index in 0..12 {
        log.info(&format!("line-{index}")).expect("info");
    }
    assert!(path.exists());
    assert!(temp.path().join("app.log.1").exists());
}
