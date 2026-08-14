use pulldown_cmark::{Event, Options, Parser, html};

use crate::mermaid::Mermaid;
use crate::toc::{HeadingIds, TocItem};

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
    render_document(markdown).html
}

/// Renders Markdown and collects its table of contents.
#[must_use]
pub fn render_document(markdown: &str) -> RenderedDocument {
    let mut output = String::new();
    let mut toc = Vec::new();
    let has_headings = may_have_heading(markdown);
    let parser = Parser::new_ext(markdown, options());

    if markdown.contains("mermaid") {
        render_events(Mermaid::new(parser), has_headings, &mut output, &mut toc);
    } else {
        render_events(parser, has_headings, &mut output, &mut toc);
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
        html::push_html(output, &mut events);
    } else {
        html::push_html(output, events);
    }
}

fn may_have_heading(markdown: &str) -> bool {
    markdown
        .bytes()
        .any(|byte| matches!(byte, b'#' | b'=' | b'-'))
}

fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH
}
