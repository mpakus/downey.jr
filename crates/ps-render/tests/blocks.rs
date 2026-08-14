use ps_render::render_document;

#[test]
fn maps_top_level_blocks_back_to_exact_source_ranges() {
    let markdown = "# Заголовок 🚀\r\n\r\nParagraph *one*.\r\n\r\n- parent\r\n  - child\r\n\r\n<div>RTL مرحبا</div>\r\n";
    let rendered = render_document(markdown);
    let expected = [
        (0, 1, "# Заголовок 🚀\r\n"),
        (1, 3, "Paragraph *one*.\r\n"),
        (2, 5, "- parent\r\n  - child\r\n\r\n"),
        (3, 8, "<div>RTL مرحبا</div>\r\n"),
    ];

    assert_eq!(rendered.blocks.len(), expected.len());
    for (block, (index, source_line, source)) in rendered.blocks.iter().zip(expected) {
        assert_eq!(block.index, index);
        assert_eq!(block.source_line, source_line);
        assert_eq!(&markdown[block.source_range.clone()], source);
        assert_eq!(
            block.hash,
            blake3::hash(source.as_bytes()).to_hex().to_string()
        );

        let attributes = format!(
            "data-block=\"{index}\" data-src-line=\"{source_line}\" data-hash=\"{}\"",
            block.hash
        );
        assert!(
            rendered.html.contains(&attributes),
            "missing {attributes:?} in {}",
            rendered.html
        );
    }
}

#[test]
fn keeps_nested_and_unclosed_markdown_inside_one_top_level_block() {
    let markdown = "> quote\n>\n> - nested\n\n```rust\nunclosed\n";
    let rendered = render_document(markdown);

    assert_eq!(rendered.blocks.len(), 2);
    assert_eq!(
        &markdown[rendered.blocks[0].source_range.clone()],
        "> quote\n>\n> - nested\n"
    );
    assert_eq!(
        &markdown[rendered.blocks[1].source_range.clone()],
        "```rust\nunclosed\n"
    );
    assert_eq!(rendered.blocks[0].source_line, 1);
    assert_eq!(rendered.blocks[1].source_line, 5);
}

#[test]
fn source_maps_cover_the_renderer_corpus() {
    let corpus = [
        "# CommonMark\n\nParagraph with *emphasis* and [link](https://example.com).\n",
        "| A | B |\n| - | - |\n| 1 | 2 |\n",
        "Footnote[^1].\n\n[^1]: note\n",
        "- [x] done\n- [ ] open\n",
        "```mermaid\ngraph TD\nA --> B\n```\n",
        "Inline $x + y$.\n\n$$z = 1$$\n",
        "- outer\n  - nested\n    - deep\n",
        "<div>raw <strong>HTML</strong></div>\n",
        "Broken **strong\n\n```rust\nunclosed\n",
        "Кириллица 😀\n\nمرحبا بالعالم\n",
    ];

    for markdown in corpus {
        let rendered = render_document(markdown);
        let mut previous_end = 0;

        assert!(!rendered.blocks.is_empty(), "no blocks for {markdown:?}");
        for block in &rendered.blocks {
            assert!(block.source_range.start >= previous_end);
            assert!(block.source_range.end <= markdown.len());
            assert!(markdown.is_char_boundary(block.source_range.start));
            assert!(markdown.is_char_boundary(block.source_range.end));

            let source = &markdown[block.source_range.clone()];
            let source_line = markdown.as_bytes()[..block.source_range.start]
                .iter()
                .filter(|byte| **byte == b'\n')
                .count()
                + 1;
            assert_eq!(block.source_line, source_line);
            assert_eq!(
                block.hash,
                blake3::hash(source.as_bytes()).to_hex().to_string()
            );
            assert!(
                rendered
                    .html
                    .contains(&format!("data-block=\"{}\"", block.index)),
                "missing block {} for {markdown:?}",
                block.index
            );
            previous_end = block.source_range.end;
        }
    }
}
