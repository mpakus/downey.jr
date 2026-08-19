use std::fs;

use ps_core::themes::ThemeCatalog;

#[test]
fn loads_twelve_builtin_themes_and_skips_invalid_user_files() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let user = temp.path().join("themes");
    fs::create_dir(&user).expect("user themes");
    fs::write(user.join("broken.json"), "{not json").expect("broken");
    fs::write(
        user.join("custom.json"),
        r##"{
          "id": "custom-sand",
          "name": "Custom Sand",
          "appearance": "light",
          "tokens": {
            "bg": "#FFF8F0",
            "bg-elev": "#FFFFFF",
            "sidebar": "#F3EDE4",
            "fg": "#1E1C1A",
            "fg-muted": "#6B6763",
            "border": "#E3DFD8",
            "accent": "#C1452F",
            "selection": "#F0DFD8",
            "code-bg": "#F4F2ED",
            "hl-kw": "#8B2F8B",
            "hl-str": "#2E7D32",
            "hl-num": "#B35C00",
            "hl-com": "#8C8880",
            "hl-fn": "#1565C0",
            "hl-type": "#00695C",
            "ed-cursor": "#C1452F",
            "ed-sel": "#DCE8F5",
            "ed-active-line": "#F5F3EE",
            "ed-syntax": "#A8A29A"
          }
        }"##,
    )
    .expect("custom");

    let catalog = ThemeCatalog::load(&user);
    let infos = catalog.infos();
    assert_eq!(infos.iter().filter(|theme| theme.builtin).count(), 12);
    assert!(
        infos
            .iter()
            .any(|theme| theme.id == "custom-sand" && !theme.builtin)
    );
    assert!(
        catalog
            .warnings
            .iter()
            .any(|warning| warning.contains("broken.json"))
    );
    let css = catalog.css();
    assert!(css.contains("color-scheme: light"));
    assert!(css.contains("[data-theme='paper-light']"));
    assert!(css.contains("[data-theme='custom-sand']"));
    assert!(css.contains("--bg: #FBFAF7;"));
}

const CUSTOM_TOKENS: &str = r##"
            "bg": "#FFF8F0",
            "bg-elev": "#FFFFFF",
            "sidebar": "#F3EDE4",
            "fg": "#1E1C1A",
            "fg-muted": "#6B6763",
            "border": "#E3DFD8",
            "accent": "#C1452F",
            "selection": "#F0DFD8",
            "code-bg": "#F4F2ED",
            "hl-kw": "#8B2F8B",
            "hl-str": "#2E7D32",
            "hl-num": "#B35C00",
            "hl-com": "#8C8880",
            "hl-fn": "#1565C0",
            "hl-type": "#00695C",
            "ed-cursor": "#C1452F",
            "ed-sel": "#DCE8F5",
            "ed-active-line": "#F5F3EE",
            "ed-syntax": "#A8A29A"
"##;

#[test]
fn skips_non_json_files_duplicate_ids_and_invalid_user_themes() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let user = temp.path().join("themes");
    fs::create_dir(&user).expect("user themes");
    fs::write(user.join("notes.txt"), "not a theme").expect("non-json");
    fs::write(
        user.join("paper-light.json"),
        format!(
            r##"{{
          "id": "paper-light",
          "name": "Duplicate",
          "appearance": "light",
          "tokens": {{ {CUSTOM_TOKENS} }}
        }}"##
        ),
    )
    .expect("duplicate id");
    fs::write(
        user.join("bad-id.json"),
        format!(
            r##"{{
          "id": "Not A Slug",
          "name": "Bad",
          "appearance": "dark",
          "tokens": {{ {CUSTOM_TOKENS} }}
        }}"##
        ),
    )
    .expect("bad id");
    fs::write(
        user.join("empty-name.json"),
        format!(
            r##"{{
          "id": "empty-name",
          "name": "   ",
          "appearance": "light",
          "tokens": {{ {CUSTOM_TOKENS} }}
        }}"##
        ),
    )
    .expect("empty name");
    fs::write(
        user.join("bad-color.json"),
        r##"{
          "id": "bad-color",
          "name": "Bad Color",
          "appearance": "light",
          "tokens": {
            "bg": "red",
            "bg-elev": "#FFFFFF",
            "sidebar": "#F3EDE4",
            "fg": "#1E1C1A",
            "fg-muted": "#6B6763",
            "border": "#E3DFD8",
            "accent": "#C1452F",
            "selection": "#F0DFD8",
            "code-bg": "#F4F2ED",
            "hl-kw": "#8B2F8B",
            "hl-str": "#2E7D32",
            "hl-num": "#B35C00",
            "hl-com": "#8C8880",
            "hl-fn": "#1565C0",
            "hl-type": "#00695C",
            "ed-cursor": "#C1452F",
            "ed-sel": "#DCE8F5",
            "ed-active-line": "#F5F3EE",
            "ed-syntax": "#A8A29A"
          }
        }"##,
    )
    .expect("bad color");

    let catalog = ThemeCatalog::load(&user);
    assert!(!catalog.infos().iter().any(|theme| theme.id == "empty-name"));
    assert!(
        catalog
            .warnings
            .iter()
            .any(|warning| warning.contains("already loaded"))
    );
    assert!(
        catalog
            .warnings
            .iter()
            .any(|warning| warning.contains("bad-id.json"))
    );
    assert!(
        catalog
            .warnings
            .iter()
            .any(|warning| warning.contains("empty-name.json"))
    );
    assert!(
        catalog
            .warnings
            .iter()
            .any(|warning| warning.contains("bad-color.json"))
    );
}

#[test]
fn missing_user_theme_directory_is_ignored() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let catalog = ThemeCatalog::load(&temp.path().join("missing"));
    assert_eq!(catalog.infos().len(), 12);
    assert!(catalog.warnings.is_empty());
}

#[cfg(unix)]
#[test]
fn unreadable_user_theme_directory_is_reported() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("temporary directory");
    let user = temp.path().join("themes");
    fs::create_dir(&user).expect("user themes");
    fs::set_permissions(&user, fs::Permissions::from_mode(0o000)).expect("lock");
    let catalog = ThemeCatalog::load(&user);
    fs::set_permissions(&user, fs::Permissions::from_mode(0o755)).expect("unlock");
    assert!(
        catalog
            .warnings
            .iter()
            .any(|warning| warning.contains("couldn't read"))
    );
}

#[cfg(unix)]
#[test]
fn unreadable_user_theme_file_is_skipped() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("temporary directory");
    let user = temp.path().join("themes");
    fs::create_dir(&user).expect("user themes");
    let locked = user.join("locked.json");
    fs::write(&locked, "{}").expect("theme file");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("lock");
    let catalog = ThemeCatalog::load(&user);
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o644)).expect("unlock");
    assert!(
        catalog
            .warnings
            .iter()
            .any(|warning| warning.contains("locked.json"))
    );
}
