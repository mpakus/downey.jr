use ps_render::{render, render_document};

#[test]
fn assigns_unique_heading_ids_and_collects_a_toc() {
    let rendered = render_document(
        "# Hello, *world*!\n\n## Hello, world!\n\n### Привет, мир! 🚀\n\n# Custom {#kept}\n\n# Another {#kept}\n",
    );

    let entries = rendered
        .toc
        .iter()
        .map(|item| (item.level, item.title.as_str(), item.id.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(
        entries,
        [
            (1, "Hello, world!", "hello-world"),
            (2, "Hello, world!", "hello-world-1"),
            (3, "Привет, мир! 🚀", "привет-мир"),
            (1, "Custom", "kept"),
            (1, "Another", "kept-1"),
        ]
    );

    for id in [
        "hello-world",
        "hello-world-1",
        "привет-мир",
        "kept",
        "kept-1",
    ] {
        assert!(
            rendered.html.contains(&format!("id=\"{id}\"")),
            "missing heading id {id:?} in {}",
            rendered.html
        );
    }

    assert!(render("# Reader").contains("<h1 id=\"reader\">Reader</h1>"));
}
