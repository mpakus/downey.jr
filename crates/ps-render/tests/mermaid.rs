use ps_render::render;

#[test]
fn turns_mermaid_fences_into_hashed_safe_templates() {
    let source = "graph TD\n  A[<unsafe>] --> B\n  </template><script>alert(1)</script>\n";
    let markdown = format!("```mermaid\n{source}```\n\n```rust\nfn main() {{}}\n```\n");

    let html = render(&markdown);

    assert!(html.starts_with("<figure class=\"mermaid\" data-hash=\""));
    assert!(html.contains("<template>graph TD\n"));
    assert!(html.contains("A[&lt;unsafe&gt;] --&gt; B"));
    assert!(html.contains("&lt;/template&gt;&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("</template></figure>"));
    assert!(html.contains("<pre><code class=\"language-rust\">"));
    assert!(!html.contains("<script>alert(1)</script>"));

    let hash = html
        .split_once("data-hash=\"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(hash, _)| hash)
        .expect("Mermaid figure must contain a data hash");
    assert_eq!(hash.len(), 64);
    assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));

    let changed = render("```mermaid\ngraph TD\n  A --> C\n```\n");
    assert!(!changed.contains(&format!("data-hash=\"{hash}\"")));
}
