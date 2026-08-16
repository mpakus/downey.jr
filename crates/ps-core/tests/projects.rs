use std::fs;

use ps_core::projects::ProjectStore;
use ulid::Ulid;

#[test]
fn project_crud_never_modifies_the_project_directory() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let store_path = temp.path().join("projects.json");
    let project_root = temp.path().join("notes");
    fs::create_dir(&project_root).expect("project directory");
    let document = project_root.join("important.md");
    fs::write(&document, b"Never delete this text").expect("project document");

    let mut store = ProjectStore::open(&store_path).expect("project store");
    let project = store
        .add("Notes", project_root.clone())
        .expect("added project");
    Ulid::from_string(&project.id).expect("ULID project identifier");
    assert_eq!(project.available, None);
    assert!(
        store
            .refresh_availability(&project.id)
            .expect("availability")
    );

    store
        .rename(&project.id, "Writing")
        .expect("renamed project");
    assert_eq!(store.get(&project.id).expect("found").name, "Writing");
    assert!(store.get("missing").is_err());
    store.remove(&project.id).expect("removed project record");
    store.flush().expect("saved project list");

    assert_eq!(
        fs::read(&document).expect("preserved document"),
        b"Never delete this text"
    );
    assert!(
        ProjectStore::open(&store_path)
            .expect("reopened project store")
            .list()
            .is_empty()
    );
}

#[test]
fn availability_is_checked_only_when_requested() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let missing = temp.path().join("not-mounted");
    let mut store = ProjectStore::open(temp.path().join("projects.json")).expect("project store");
    let project = store.add("Archive", missing).expect("added project");

    assert_eq!(project.available, None);
    assert!(
        !store
            .refresh_availability(&project.id)
            .expect("availability")
    );
    assert_eq!(store.list()[0].available, Some(false));
}
