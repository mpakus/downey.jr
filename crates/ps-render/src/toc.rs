use std::collections::{HashSet, VecDeque};

use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};

use crate::blocks::SpannedEvent;

/// A heading shown in the document table of contents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TocItem {
    /// Heading depth from 1 through 6.
    pub level: u8,
    /// Plain-text heading label.
    pub title: String,
    /// Unique HTML identifier assigned to the heading.
    pub id: String,
}

pub(crate) struct HeadingIds<'input, 'toc, I> {
    events: I,
    pending: VecDeque<SpannedEvent<'input>>,
    used_ids: HashSet<String>,
    toc: &'toc mut Vec<TocItem>,
}

impl<'input, 'toc, I> HeadingIds<'input, 'toc, I>
where
    I: Iterator<Item = SpannedEvent<'input>>,
{
    pub(crate) fn new(events: I, toc: &'toc mut Vec<TocItem>) -> Self {
        Self {
            events,
            pending: VecDeque::new(),
            used_ids: HashSet::new(),
            toc,
        }
    }

    fn queue_heading(
        &mut self,
        level: HeadingLevel,
        explicit_id: Option<pulldown_cmark::CowStr<'input>>,
        classes: Vec<pulldown_cmark::CowStr<'input>>,
        attrs: Vec<(
            pulldown_cmark::CowStr<'input>,
            Option<pulldown_cmark::CowStr<'input>>,
        )>,
        source_range: std::ops::Range<usize>,
    ) {
        let mut body = Vec::new();
        let mut title = String::new();

        for event in self.events.by_ref() {
            if matches!(event.0, Event::End(TagEnd::Heading(_))) {
                body.push(event);
                break;
            }
            append_title(&mut title, &event.0);
            body.push(event);
        }

        let base_id = explicit_id
            .filter(|id| !id.is_empty())
            .map(String::from)
            .unwrap_or_else(|| slug(&title));
        let id = unique_id(base_id, &mut self.used_ids);

        self.toc.push(TocItem {
            level: heading_level(level),
            title,
            id: id.clone(),
        });
        self.pending.push_back((
            Event::Start(Tag::Heading {
                level,
                id: Some(id.into()),
                classes,
                attrs,
            }),
            source_range,
        ));
        self.pending.extend(body);
    }
}

impl<'input, I> Iterator for HeadingIds<'input, '_, I>
where
    I: Iterator<Item = SpannedEvent<'input>>,
{
    type Item = SpannedEvent<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.pending.pop_front() {
            return Some(event);
        }

        match self.events.next()? {
            (
                Event::Start(Tag::Heading {
                    level,
                    id,
                    classes,
                    attrs,
                }),
                source_range,
            ) => {
                self.queue_heading(level, id, classes, attrs, source_range);
                self.pending.pop_front()
            }
            event => Some(event),
        }
    }
}

fn append_title(title: &mut String, event: &Event<'_>) {
    match event {
        Event::Text(text)
        | Event::Code(text)
        | Event::InlineMath(text)
        | Event::DisplayMath(text) => title.push_str(text),
        Event::SoftBreak | Event::HardBreak if !title.ends_with(' ') => title.push(' '),
        _ => {}
    }
}

fn slug(title: &str) -> String {
    let mut slug = String::new();
    let mut needs_separator = false;

    for character in title.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() || character == '_' {
            if needs_separator && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(character);
            needs_separator = false;
        } else if !slug.is_empty() {
            needs_separator = true;
        }
    }

    if slug.is_empty() {
        "section".to_owned()
    } else {
        slug
    }
}

fn unique_id(base: String, used_ids: &mut HashSet<String>) -> String {
    if used_ids.insert(base.clone()) {
        return base;
    }

    for suffix in 1_u64.. {
        let candidate = format!("{base}-{suffix}");
        if used_ids.insert(candidate.clone()) {
            return candidate;
        }
    }

    base
}

const fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}
