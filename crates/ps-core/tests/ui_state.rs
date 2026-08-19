use std::path::PathBuf;

use ps_core::store::{JsonStore, VersionedDocument};
use ps_core::ui_state::UiState;

#[test]
fn expanded_directories_round_trip_and_reject_escapes() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let path = temp.path().join("ui-state.json");
    let mut store = JsonStore::<UiState>::open(&path).expect("open");
    let mut state = store.value().clone();
    state
        .set_expanded(
            "proj".into(),
            vec![PathBuf::from("chapters"), PathBuf::from("chapters/drafts")],
        )
        .expect("set");
    store.replace(state);
    store.flush().expect("flush");

    let reopened = JsonStore::<UiState>::open(&path).expect("reopen");
    assert_eq!(
        reopened.value().expanded_for("proj"),
        vec!["chapters".to_owned(), "chapters/drafts".to_owned()]
    );
    assert!(reopened.value().expanded_for("missing").is_empty());

    let mut state = UiState::default();
    assert!(
        state
            .set_expanded("proj".into(), vec![PathBuf::from("../secret")])
            .is_err()
    );
    assert!(
        state
            .set_expanded("proj".into(), vec![PathBuf::from("/tmp/notes")])
            .is_err()
    );

    state
        .set_expanded("proj".into(), vec![PathBuf::from("inbox")])
        .expect("inbox");
    state.remove_project("proj");
    assert!(state.expanded_for("proj").is_empty());

    let mut cleared = UiState::default();
    cleared
        .set_expanded(
            "proj".into(),
            vec![PathBuf::from(""), PathBuf::from("inbox")],
        )
        .expect("skip empty");
    assert_eq!(cleared.expanded_for("proj"), vec!["inbox".to_owned()]);
    cleared
        .set_expanded("proj".into(), Vec::new())
        .expect("clear");
    assert!(cleared.expanded_for("proj").is_empty());
    assert!(UiState::migrate(serde_json::json!({}), 0).is_err());

    let invalid = UiState {
        schema_version: 0,
        expanded: Default::default(),
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn missing_file_starts_with_a_valid_default() {
    let opened = JsonStore::<UiState>::open(
        tempfile::tempdir()
            .expect("temporary directory")
            .path()
            .join("missing.json"),
    )
    .expect("missing file uses defaults");
    assert_eq!(opened.value().schema_version, 1);
    assert!(opened.value().expanded_for("any").is_empty());
}
