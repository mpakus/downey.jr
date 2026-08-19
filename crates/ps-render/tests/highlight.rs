use ps_render::render;

#[test]
fn emits_prefixed_classes_without_inline_styles() {
    let html = render("```rust\nfn main() { println!(\"<hello>\"); }\n```\n");

    assert!(html.contains("<pre class=\"code\"><code class=\"language-rust\">"));
    assert!(html.contains("<span class=\"syntax-source syntax-rust\">"));
    assert!(html.contains("&lt;hello&gt;"));
    assert!(!html.contains("style="));
}

#[test]
fn leaves_unknown_fenced_languages_to_the_markdown_renderer() {
    let html = render("```unknown-language\n<plain>\n```\n");

    assert!(html.contains("<pre><code class=\"language-unknown-language\">"));
    assert!(html.contains("&lt;plain&gt;"));
    assert!(!html.contains("syntax-"));
}

#[test]
fn highlights_common_language_aliases() {
    for (fence, needle) in [
        ("```js\nconst n = 1;\n```\n", "syntax-source syntax-js"),
        (
            "```python\ndef hi():\n    return 1\n```\n",
            "syntax-source syntax-python",
        ),
        ("```rb\nputs 1\n```\n", "syntax-source syntax-ruby"),
        (
            "```elixir\ndefmodule M do\nend\n```\n",
            "syntax-source syntax-elixir",
        ),
        ("```yaml\nkey: value\n```\n", "syntax-source syntax-yaml"),
        ("```rust\nlet n = 1;\n```\n", "syntax-source syntax-rust"),
    ] {
        let html = render(fence);
        assert!(
            html.contains("<pre class=\"code\">"),
            "missing highlighted pre for {fence:?} in {html}"
        );
        assert!(
            html.contains(needle),
            "missing {needle:?} for {fence:?} in {html}"
        );
    }
}

#[test]
fn repeated_code_blocks_keep_identical_highlighted_output() {
    let html = render("```rust\nlet answer = 42;\n```\n\n```rust\nlet answer = 42;\n```\n");

    assert_eq!(html.matches("syntax-source syntax-rust").count(), 2);
    assert_eq!(
        html.matches("syntax-storage syntax-type syntax-rust")
            .count(),
        2
    );
}
