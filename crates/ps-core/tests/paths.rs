use ps_core::paths::AppPaths;

#[test]
fn ensure_creates_the_application_directory_tree() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let paths = AppPaths::from_root(temp.path().join("paperstreet"));

    paths.ensure().expect("application directories");

    assert!(paths.root().is_dir());
    assert!(paths.themes().is_dir());
    assert!(paths.cache().is_dir());
    assert!(paths.logs().is_dir());
    assert_eq!(paths.config_file(), paths.root().join("config.json"));
    assert_eq!(paths.projects_file(), paths.root().join("projects.json"));
}
