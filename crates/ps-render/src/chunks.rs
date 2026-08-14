use std::iter::Peekable;

use pulldown_cmark::Event;

use crate::blocks::BlockEvent;

pub(crate) const OPEN_SECTION: &str = "<section class=\"chunk\">";
pub(crate) const CLOSE_SECTION: &str = "</section>\n";
const SECTION_SEPARATOR: &str = "</section>\n<section class=\"chunk\">";
const TARGET_BYTES: usize = 64 * 1024;

pub(crate) struct Chunks<I: Iterator> {
    events: Peekable<I>,
    bytes: usize,
    separator_pending: bool,
}

impl<I: Iterator> Chunks<I> {
    pub(crate) fn new(events: I) -> Self {
        Self {
            events: events.peekable(),
            bytes: 0,
            separator_pending: false,
        }
    }

    pub(crate) fn has_events(&mut self) -> bool {
        self.events.peek().is_some()
    }
}

impl<'input, I> Iterator for Chunks<I>
where
    I: Iterator<Item = BlockEvent<'input>>,
{
    type Item = Event<'input>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.separator_pending {
            self.separator_pending = false;
            if self.events.peek().is_some() {
                return Some(Event::Html(SECTION_SEPARATOR.into()));
            }
        }

        let (event, boundary) = match self.events.next()? {
            BlockEvent::Start(event) | BlockEvent::Event(event) => (event, false),
            BlockEvent::End(event) => (event, true),
        };
        self.bytes += event_bytes(&event);

        if boundary && self.bytes >= TARGET_BYTES {
            self.separator_pending = true;
            self.bytes = 0;
        }
        Some(event)
    }
}

fn event_bytes(event: &Event<'_>) -> usize {
    match event {
        Event::Start(_) | Event::End(_) => 8,
        Event::Text(text)
        | Event::Code(text)
        | Event::Html(text)
        | Event::InlineHtml(text)
        | Event::FootnoteReference(text)
        | Event::InlineMath(text)
        | Event::DisplayMath(text) => text.len(),
        Event::SoftBreak | Event::HardBreak | Event::TaskListMarker(_) => 1,
        Event::Rule => 4,
    }
}
