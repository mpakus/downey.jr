use std::path::PathBuf;

use ps_core::projects::Project;
use ps_core::search::ProjectSearch;

fn project(id: &str, name: &str, path: &str) -> Project {
    Project {
        id: id.to_owned(),
        name: name.to_owned(),
        path: PathBuf::from(path),
        added_at: "2026-08-13T12:00:00Z".to_owned(),
        last_opened_at: None,
        pinned: false,
        accent: None,
        last_file: None,
        available: None,
    }
}

#[test]
fn fuzzy_search_matches_project_names_and_paths() {
    let mut search = ProjectSearch::new();
    search.upsert(project(
        "one",
        "Fight Club Notes",
        "/Users/tyler/Documents/notes",
    ));
    search.upsert(project("two", "Recipes", "/Users/tyler/Kitchen"));

    assert_eq!(search.search("fght clb", 10)[0].id, "one");
    assert_eq!(search.search("kitch", 10)[0].id, "two");
}

#[test]
fn index_updates_and_removes_projects_incrementally() {
    let mut search = ProjectSearch::new();
    search.upsert(project("one", "Old Name", "/Users/tyler/Notes"));
    search.upsert(project("one", "New Name", "/Users/tyler/Notes"));

    assert!(search.search("old", 10).is_empty());
    assert_eq!(search.search("new", 10)[0].id, "one");

    assert!(search.remove("one"));
    assert!(search.search("new", 10).is_empty());
    assert!(!search.remove("one"));
}
