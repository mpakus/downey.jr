use std::ops::Range;

use pulldown_cmark::{Event, Tag, TagEnd, html};

use crate::blocks::SpannedEvent;

pub(crate) struct FrontMatter<I> {
    events: I,
}

impl<I> FrontMatter<I> {
    pub(crate) fn new(events: I) -> Self {
        Self { events }
    }
}

impl<'input, I> Iterator for FrontMatter<I>
where
    I: Iterator<Item = SpannedEvent<'input>>,
{
    type Item = SpannedEvent<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        let (event, mut source_range) = self.events.next()?;
        if !matches!(event, Event::Start(Tag::MetadataBlock(_))) {
            return Some((event, source_range));
        }

        let mut yaml = String::new();
        for (inner, inner_range) in self.events.by_ref() {
            extend_range(&mut source_range, inner_range.end);
            match inner {
                Event::Text(text) => yaml.push_str(&text),
                Event::End(TagEnd::MetadataBlock(_)) => break,
                _ => {}
            }
        }

        Some((Event::Html(metadata_html(&yaml).into()), source_range))
    }
}

fn extend_range(range: &mut Range<usize>, end: usize) {
    if end > range.end {
        range.end = end;
    }
}

fn metadata_html(yaml: &str) -> String {
    let mut html = String::from("<div class=\"front-matter\"><dl>");
    let mut wrote = false;
    for line in yaml.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let value = value
            .trim()
            .trim_matches(|character| matches!(character, '"' | '\''));
        html.push_str("<dt>");
        push_escaped(&mut html, key);
        html.push_str("</dt><dd>");
        push_escaped(&mut html, value);
        html.push_str("</dd>");
        wrote = true;
    }
    if !wrote {
        html.push_str("<dt>front matter</dt><dd><pre>");
        push_escaped(&mut html, yaml.trim());
        html.push_str("</pre></dd>");
    }
    html.push_str("</dl></div>\n");
    html
}

fn push_escaped(output: &mut String, text: &str) {
    html::push_html(output, [Event::Text(text.into())].into_iter());
}
