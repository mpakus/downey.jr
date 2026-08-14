use ps_render::{render, render_document};

const FIVE_MIB: usize = 5 * 1024 * 1024;

macro_rules! snapshot_case {
    ($name:ident, $file:literal) => {
        #[test]
        fn $name() {
            insta::assert_snapshot!(stringify!($name), render(include_str!($file)));
        }
    };
}

snapshot_case!(commonmark, "corpus/commonmark.md");
snapshot_case!(tables, "corpus/tables.md");
snapshot_case!(footnotes, "corpus/footnotes.md");
snapshot_case!(task_lists, "corpus/task-lists.md");
snapshot_case!(mermaid, "corpus/mermaid.md");
snapshot_case!(math, "corpus/math.md");
snapshot_case!(nested_lists, "corpus/nested-lists.md");
snapshot_case!(raw_html, "corpus/raw-html.md");
snapshot_case!(broken_markdown, "corpus/broken-markdown.md");
snapshot_case!(unicode_rtl, "corpus/unicode-rtl.md");

#[test]
fn large_document() {
    let seed = include_str!("corpus/large-document.md");
    let mut markdown = seed.repeat(FIVE_MIB.div_ceil(seed.len()));
    markdown.truncate(FIVE_MIB);

    let rendered = render_document(&markdown);
    let summary = format!(
        "input_bytes: {}\noutput_bytes: {}\nblocks: {}\nchunks: {}\noutput_hash: {}",
        markdown.len(),
        rendered.html.len(),
        rendered.blocks.len(),
        rendered.html.matches("<section class=\"chunk\">").count(),
        blake3::hash(rendered.html.as_bytes()).to_hex()
    );

    insta::assert_snapshot!(summary);
}
