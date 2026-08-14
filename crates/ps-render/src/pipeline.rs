use std::cell::Cell;
use std::path::Path;

use pulldown_cmark::{Event, Options, Parser, html};

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
    let raw_html_seen = Cell::new(false);
    let events = sanitize::Events::new(
        Parser::new_ext(markdown, markdown_options()),
        &raw_html_seen,
    );
    finish_render(markdown, events, render_options, &raw_html_seen)
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
    let raw_html_seen = Cell::new(false);
    let events = sanitize::Events::new(
        Parser::new_ext(markdown, markdown_options()),
        &raw_html_seen,
    );
    finish_render(
        markdown,
        Links::new(events, project_root, document_path, project_scope),
        render_options,
        &raw_html_seen,
    )
}

fn finish_render<'input>(
    markdown: &str,
    events: impl Iterator<Item = Event<'input>>,
    render_options: RenderOptions,
    raw_html_seen: &Cell<bool>,
) -> RenderedDocument {
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
        );
        Some(prefix)
    } else {
        render_events(events, has_headings, &mut output, &mut toc);
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

    RenderedDocument { html: output, toc }
}

fn render_events<'input>(
    events: impl Iterator<Item = Event<'input>>,
    has_headings: bool,
    output: &mut String,
    toc: &mut Vec<TocItem>,
) {
    if has_headings {
        let mut events = HeadingIds::new(events, toc);
        render_chunks(&mut events, output);
    } else {
        render_chunks(events, output);
    }
}

fn render_chunks<'input>(events: impl Iterator<Item = Event<'input>>, output: &mut String) {
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

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH
}
