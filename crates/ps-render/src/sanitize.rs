use std::cell::Cell;

use pulldown_cmark::{Event, Tag};

pub(crate) struct Events<'input, 'context, I> {
    events: I,
    raw_html_seen: &'context Cell<bool>,
    marker: std::marker::PhantomData<&'input str>,
}

impl<'input, 'context, I> Events<'input, 'context, I>
where
    I: Iterator<Item = Event<'input>>,
{
    pub(crate) fn new(events: I, raw_html_seen: &'context Cell<bool>) -> Self {
        Self {
            events,
            raw_html_seen,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'input, I> Iterator for Events<'input, '_, I>
where
    I: Iterator<Item = Event<'input>>,
{
    type Item = Event<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.events.next()?;
        if matches!(event, Event::Html(_) | Event::InlineHtml(_)) {
            self.raw_html_seen.set(true);
        }

        match event {
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) if forbidden_scheme(&dest_url) => Some(Event::Start(Tag::Link {
                link_type,
                dest_url: "#invalid-url".into(),
                title,
                id,
            })),
            event => Some(event),
        }
    }
}

pub(crate) fn clean(html: &str) -> String {
    let mut sanitizer = ammonia::Builder::default();
    sanitizer
        .add_tags(["input"])
        .add_generic_attributes(["class", "id", "data-hash"])
        .add_tag_attributes("input", ["checked", "disabled", "type"])
        .add_url_schemes(["asset"]);
    sanitizer.clean(html).to_string()
}

fn forbidden_scheme(destination: &str) -> bool {
    let Some((scheme, _)) = destination.split_once(':') else {
        return false;
    };
    let normalized: String = scheme
        .chars()
        .filter(|character| !matches!(character, '\t' | '\n' | '\r'))
        .collect();
    let normalized = normalized.trim_matches(|character: char| character <= ' ');
    normalized.eq_ignore_ascii_case("javascript") || normalized.eq_ignore_ascii_case("data")
}
