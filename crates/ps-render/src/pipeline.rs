use pulldown_cmark::{Options, Parser, html};

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

    if !may_have_heading(markdown) {
        html::push_html(&mut output, Parser::new_ext(markdown, options()));
        return RenderedDocument { html: output, toc };
    }

    let mut events = HeadingIds::new(Parser::new_ext(markdown, options()), &mut toc);
    html::push_html(&mut output, &mut events);
    drop(events);

    RenderedDocument { html: output, toc }
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
