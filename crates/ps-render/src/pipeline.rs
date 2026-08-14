use pulldown_cmark::{Options, Parser, html};

/// Renders Markdown as HTML with the extensions supported by 1537paperstreet.
#[must_use]
pub fn render(markdown: &str) -> String {
    let mut output = String::new();
    html::push_html(&mut output, Parser::new_ext(markdown, options()));
    output
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
