use ps_render::render;

#[test]
fn renders_the_required_markdown_extensions() {
    let markdown = r#"# Title {#custom}

| Name | Done |
| --- | --- |
| Reader | ~~no~~ yes |

- [x] shipped

Footnote[^1] -- "quoted"

Inline math: $x + y$.

$$z = 1$$

[^1]: note
"#;

    let html = render(markdown);

    for expected in [
        "<h1 id=\"custom\">Title</h1>",
        "<table>",
        "<del>no</del>",
        "type=\"checkbox\"",
        "class=\"footnote-reference\"",
        "– “quoted”",
        "class=\"math math-inline\"",
        "class=\"math math-display\"",
    ] {
        assert!(html.contains(expected), "missing {expected:?} in {html}");
    }
}
