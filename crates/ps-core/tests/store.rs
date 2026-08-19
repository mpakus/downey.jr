use std::fs;

use ps_core::store::{JsonStore, StoreWarning, VersionedDocument};
use ps_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct TestDocument {
    schema_version: u32,
    value: String,
}

impl VersionedDocument for TestDocument {
    const SCHEMA_VERSION: u32 = 1;

    fn migrate(mut value: Value, from: u32) -> Result<Value> {
        if from != 0 {
            return Err(Error::UnsupportedSchema {
                found: from,
                supported: Self::SCHEMA_VERSION,
            });
        }
        value["schema_version"] = Value::from(Self::SCHEMA_VERSION);
        Ok(value)
    }
}

#[test]
fn writes_only_after_the_debounce_or_a_forced_flush() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = temp.path().join("state.json");
    let mut store = JsonStore::<TestDocument>::open(&path).expect("store");

    store.replace(TestDocument {
        schema_version: 1,
        value: "saved".into(),
    });

    assert!(!store.flush_if_due().expect("debounced flush"));
    assert!(!path.exists());
    assert!(store.flush().expect("forced flush"));

    let reopened = JsonStore::<TestDocument>::open(&path).expect("reopened store");
    assert_eq!(reopened.value().value, "saved");
}

#[test]
fn backs_up_and_migrates_an_older_schema() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = temp.path().join("state.json");
    fs::write(&path, br#"{"schema_version":0,"value":"old"}"#).expect("old schema");

    let store = JsonStore::<TestDocument>::open(&path).expect("migrated store");

    assert_eq!(store.value().schema_version, 1);
    assert_eq!(store.value().value, "old");
    assert!(temp.path().join("state.json.bak.0").is_file());
}

#[test]
fn preserves_corrupt_json_and_starts_with_defaults() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = temp.path().join("state.json");
    fs::write(&path, b"not json").expect("corrupt state");

    let mut store = JsonStore::<TestDocument>::open(&path).expect("default store");

    assert_eq!(store.value(), &TestDocument::default());
    let StoreWarning::CorruptFileMoved { preserved_at } =
        store.take_warning().expect("corrupt-file warning");
    assert!(preserved_at.is_file());
    assert!(!path.exists());
}

#[test]
fn rejects_a_newer_schema_and_flushes_on_close() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = temp.path().join("state.json");
    fs::write(&path, br#"{"schema_version":99,"value":"future"}"#).expect("future schema");

    let error = match JsonStore::<TestDocument>::open(&path) {
        Ok(_) => panic!("unsupported schema should fail"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        Error::UnsupportedSchema {
            found: 99,
            supported: 1
        }
    ));

    let missing = temp.path().join("fresh.json");
    let mut store = JsonStore::<TestDocument>::open(&missing).expect("defaults");
    assert!(!store.flush().expect("nothing dirty"));
    assert_eq!(store.path(), missing.as_path());
    store.update(|document| document.value = "written".into());
    store.close().expect("flush on close");
    let reopened = JsonStore::<TestDocument>::open(&missing).expect("reopened");
    assert_eq!(reopened.value().value, "written");
}

#[test]
fn flush_if_due_writes_after_the_debounce_window() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = temp.path().join("state.json");
    let mut store = JsonStore::<TestDocument>::open(&path).expect("store");
    store.update(|document| document.value = "later".into());
    std::thread::sleep(std::time::Duration::from_millis(520));
    assert!(store.flush_if_due().expect("due"));
    let reopened = JsonStore::<TestDocument>::open(&path).expect("reopened");
    assert_eq!(reopened.value().value, "later");
}
