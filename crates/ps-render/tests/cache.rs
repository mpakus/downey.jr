use std::fs::{self, FileTimes, OpenOptions};
use std::time::UNIX_EPOCH;

use ps_render::{RenderCache, RenderOptions};
use tempfile::tempdir;

#[test]
fn keys_cached_html_by_markdown_and_render_options() {
    let temporary = tempdir().expect("temporary cache root");
    let mut cache = RenderCache::new(temporary.path()).expect("render cache");
    let markdown = "<mark data-reader=\"yes\">raw</mark>\n";

    let sanitized = cache
        .render(markdown, RenderOptions::default())
        .expect("sanitized render");
    let raw = cache
        .render(
            markdown,
            RenderOptions {
                allow_raw_html: true,
            },
        )
        .expect("raw render");

    assert_ne!(sanitized, raw);
    assert_eq!(html_files(temporary.path()).len(), 2);

    let sanitized_path = html_files(temporary.path())
        .into_iter()
        .find(|path| fs::read_to_string(path).is_ok_and(|html| html == sanitized))
        .expect("sanitized disk entry");
    OpenOptions::new()
        .write(true)
        .open(&sanitized_path)
        .expect("open cached render")
        .set_times(FileTimes::new().set_modified(UNIX_EPOCH))
        .expect("age cached render");

    drop(cache);
    let mut reopened = RenderCache::new(temporary.path()).expect("reopened cache");
    assert_eq!(
        reopened
            .render(markdown, RenderOptions::default())
            .expect("disk cache hit"),
        sanitized
    );
    assert!(
        fs::metadata(sanitized_path)
            .expect("cache metadata")
            .modified()
            .expect("cache modification time")
            > UNIX_EPOCH
    );
}

#[test]
fn memory_lru_keeps_only_the_sixteen_most_recent_documents() {
    let temporary = tempdir().expect("temporary cache root");
    let mut cache = RenderCache::new(temporary.path()).expect("render cache");
    let first = cache
        .render("document 0\n", RenderOptions::default())
        .expect("first render");

    for index in 1..17 {
        cache
            .render(&format!("document {index}\n"), RenderOptions::default())
            .expect("render");
    }

    let first_path = html_files(temporary.path())
        .into_iter()
        .find(|path| fs::read_to_string(path).is_ok_and(|html| html == first))
        .expect("first disk entry");
    fs::write(&first_path, "disk sentinel").expect("replace disposable cache entry");

    assert_eq!(
        cache
            .render("document 0\n", RenderOptions::default())
            .expect("evicted entry reload"),
        "disk sentinel"
    );
}

#[test]
fn startup_prunes_disk_cache_to_two_hundred_mebibytes() {
    let temporary = tempdir().expect("temporary cache root");
    for name in ["a.html", "b.html", "c.html"] {
        let path = temporary.path().join(name);
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .expect("sparse cache entry");
        file.set_len(80 * 1024 * 1024).expect("sparse length");
    }

    RenderCache::new(temporary.path()).expect("pruned render cache");

    let bytes = html_files(temporary.path())
        .iter()
        .map(|path| fs::metadata(path).expect("cache metadata").len())
        .sum::<u64>();
    assert!(bytes <= 200 * 1024 * 1024, "disk cache uses {bytes} bytes");
}

fn html_files(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(directory)
        .expect("cache directory")
        .map(|entry| entry.expect("cache entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "html")
        })
        .collect()
}
