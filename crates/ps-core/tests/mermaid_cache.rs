use std::fs;

use ps_core::mermaid_cache::MermaidSvgCache;

const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn stores_and_reads_svg_keyed_by_source_and_theme() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let cache = MermaidSvgCache::open(temp.path()).expect("open cache");
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";

    assert!(cache.get(HASH, "paper-light").expect("miss").is_none());
    cache.put(HASH, "paper-light", svg).expect("put");
    assert_eq!(
        cache.get(HASH, "paper-light").expect("hit").as_deref(),
        Some(svg)
    );
    assert!(
        cache
            .get(HASH, "paper-dark")
            .expect("other theme")
            .is_none()
    );
}

#[test]
fn rejects_path_escape_in_the_cache_key() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let cache = MermaidSvgCache::open(temp.path()).expect("open cache");
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";

    assert!(cache.get("../escape", "paper-light").is_err());
    assert!(cache.get(HASH, "../theme").is_err());
    assert!(cache.put(HASH, "paper-light", "not svg").is_err());
    assert!(cache.put(HASH, "paper-light", svg).is_ok());
    assert!(!temp.path().join("escape.svg").exists());
}

#[test]
fn replaces_an_existing_entry_and_rejects_oversized_svg() {
    let temp = tempfile::tempdir().expect("temporary directory");
    let cache = MermaidSvgCache::open(temp.path()).expect("open cache");
    cache
        .put(HASH, "paper-light", "<svg>one</svg>")
        .expect("first");
    cache
        .put(HASH, "paper-light", "<svg>two</svg>")
        .expect("replace");
    assert_eq!(
        cache.get(HASH, "paper-light").expect("hit").as_deref(),
        Some("<svg>two</svg>")
    );

    let oversized = format!("<svg>{}</svg>", "a".repeat(2 * 1024 * 1024));
    assert!(cache.put(HASH, "paper-dark", &oversized).is_err());
    assert!(cache.get(HASH, "paper-dark").expect("miss").is_none());
    assert!(cache.get("short", "paper-light").is_err());
    assert!(cache.put(HASH, "not_a_slug", "<svg></svg>").is_err());
}

#[cfg(unix)]
#[test]
fn get_reports_unreadable_cache_files() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("temporary directory");
    let cache = MermaidSvgCache::open(temp.path()).expect("open cache");
    cache.put(HASH, "paper-light", "<svg></svg>").expect("put");
    let svg = fs::read_dir(temp.path())
        .expect("cache entries")
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("svg"))
        .expect("cached svg");
    fs::set_permissions(&svg, fs::Permissions::from_mode(0o000)).expect("lock cache file");
    let result = cache.get(HASH, "paper-light");
    fs::set_permissions(&svg, fs::Permissions::from_mode(0o644)).expect("unlock cache file");
    assert!(result.is_err());
}
