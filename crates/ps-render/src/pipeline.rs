use std::borrow::Cow;
use std::cell::Cell;
use std::path::Path;

use pulldown_cmark::{Options, Parser, html};

use crate::blocks::{BlockEvent, Blocks, RenderedBlock, SpannedEvent};
use crate::chunks::{CLOSE_SECTION, Chunks, OPEN_SECTION};
use crate::links::Links;
use crate::mermaid::Mermaid;
use crate::sanitize;
use crate::toc::{HeadingIds, TocItem};

/// Options that affect Markdown rendering.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderOptions {
    /// Whether raw HTML bypasses sanitization.
    pub allow_raw_html: bool,
}

/// Rendered HTML and the headings needed to build its table of contents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedDocument {
    /// The rendered Markdown.
    pub html: String,
    /// Headings in source order.
    pub toc: Vec<TocItem>,
    /// Top-level blocks with source mapping and content hashes.
    pub blocks: Vec<RenderedBlock>,
}

/// Renders Markdown as HTML with the extensions supported by 1537paperstreet.
#[must_use]
pub fn render(markdown: &str) -> String {
    render_with_options(markdown, RenderOptions::default())
}

/// Renders Markdown with explicit rendering options.
#[must_use]
pub fn render_with_options(markdown: &str, options: RenderOptions) -> String {
    render_document_with_options(markdown, options).html
}

/// Renders Markdown and collects its table of contents.
#[must_use]
pub fn render_document(markdown: &str) -> RenderedDocument {
    render_document_with_options(markdown, RenderOptions::default())
}

/// Renders Markdown and collects its table of contents with explicit options.
#[must_use]
pub fn render_document_with_options(
    markdown: &str,
    render_options: RenderOptions,
) -> RenderedDocument {
    let parser_input = normalize_parser_input(markdown);
    let raw_html_seen = Cell::new(false);
    let events = sanitize::Events::new(
        Parser::new_ext(&parser_input, markdown_options()).into_offset_iter(),
        &raw_html_seen,
    );
    let mut blocks = Vec::new();
    let (html, toc) = finish_render(
        markdown,
        events,
        render_options,
        &raw_html_seen,
        &mut blocks,
    );
    RenderedDocument { html, toc, blocks }
}

/// Renders Markdown with project-relative links restricted to one project root.
#[must_use]
pub fn render_project(
    markdown: &str,
    project_root: &Path,
    document_path: &Path,
    project_scope: &str,
) -> RenderedDocument {
    render_project_with_options(
        markdown,
        project_root,
        document_path,
        project_scope,
        RenderOptions::default(),
    )
}

/// Renders project-relative Markdown with explicit rendering options.
#[must_use]
pub fn render_project_with_options(
    markdown: &str,
    project_root: &Path,
    document_path: &Path,
    project_scope: &str,
    render_options: RenderOptions,
) -> RenderedDocument {
    let parser_input = normalize_parser_input(markdown);
    let raw_html_seen = Cell::new(false);
    let events = sanitize::Events::new(
        Parser::new_ext(&parser_input, markdown_options()).into_offset_iter(),
        &raw_html_seen,
    );
    let mut blocks = Vec::new();
    let (html, toc) = finish_render(
        markdown,
        Links::new(events, project_root, document_path, project_scope),
        render_options,
        &raw_html_seen,
        &mut blocks,
    );
    RenderedDocument { html, toc, blocks }
}

fn finish_render<'events>(
    markdown: &str,
    events: impl Iterator<Item = SpannedEvent<'events>>,
    render_options: RenderOptions,
    raw_html_seen: &Cell<bool>,
    blocks: &mut Vec<RenderedBlock>,
) -> (String, Vec<TocItem>) {
    let mut output = String::new();
    let mut toc = Vec::new();
    let mut mermaid_figures = Vec::new();
    let has_headings = may_have_heading(markdown);

    let mermaid_prefix = if markdown.contains("mermaid") {
        let prefix = format!("PSMERMAID{}__", blake3::hash(markdown.as_bytes()).to_hex());
        render_events(
            Mermaid::new(events, &prefix, &mut mermaid_figures),
            has_headings,
            &mut output,
            &mut toc,
            markdown,
            blocks,
        );
        Some(prefix)
    } else {
        render_events(
            events,
            has_headings,
            &mut output,
            &mut toc,
            markdown,
            blocks,
        );
        None
    };

    if !render_options.allow_raw_html && raw_html_seen.get() {
        output = sanitize::clean(&output);
    }
    if let Some(prefix) = mermaid_prefix {
        for (index, figure) in mermaid_figures.iter().enumerate() {
            output = output.replacen(&format!("{prefix}{index}__"), figure, 1);
        }
    }

    (output, toc)
}

fn render_events<'events>(
    events: impl Iterator<Item = SpannedEvent<'events>>,
    has_headings: bool,
    output: &mut String,
    toc: &mut Vec<TocItem>,
    markdown: &str,
    blocks: &mut Vec<RenderedBlock>,
) {
    if has_headings {
        let events = HeadingIds::new(events, toc);
        render_chunks(Blocks::new(events, markdown, blocks), output);
    } else {
        render_chunks(Blocks::new(events, markdown, blocks), output);
    }
}

fn render_chunks<'input>(events: impl Iterator<Item = BlockEvent<'input>>, output: &mut String) {
    let mut chunks = Chunks::new(events);
    if chunks.has_events() {
        output.push_str(OPEN_SECTION);
        html::push_html(output, chunks);
        output.push_str(CLOSE_SECTION);
    }
}

fn may_have_heading(markdown: &str) -> bool {
    markdown
        .bytes()
        .any(|byte| matches!(byte, b'#' | b'=' | b'-'))
}

fn normalize_parser_input(markdown: &str) -> Cow<'_, str> {
    if !markdown.chars().any(is_unsupported_control) {
        return Cow::Borrowed(markdown);
    }

    Cow::Owned(
        markdown
            .chars()
            .map(|character| {
                if is_unsupported_control(character) {
                    ' '
                } else {
                    character
                }
            })
            .collect(),
    )
}

fn is_unsupported_control(character: char) -> bool {
    matches!(
        character,
        '\0'..='\u{0008}' | '\u{000B}'..='\u{000C}' | '\u{000E}'..='\u{001F}' | '\u{007F}'
    )
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH
}
