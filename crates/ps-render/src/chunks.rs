use std::iter::Peekable;

use pulldown_cmark::Event;

pub(crate) const OPEN_SECTION: &str = "<section class=\"chunk\">";
pub(crate) const CLOSE_SECTION: &str = "</section>\n";
const SECTION_SEPARATOR: &str = "</section>\n<section class=\"chunk\">";
const TARGET_BYTES: usize = 64 * 1024;

pub(crate) struct Chunks<I: Iterator> {
    events: Peekable<I>,
    depth: usize,
    bytes: usize,
    separator_pending: bool,
}

impl<I: Iterator> Chunks<I> {
    pub(crate) fn new(events: I) -> Self {
        Self {
            events: events.peekable(),
            depth: 0,
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
    I: Iterator<Item = Event<'input>>,
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

        let event = self.events.next()?;
        let boundary = match &event {
            Event::Start(_) => {
                self.bytes += 8;
                self.depth += 1;
                false
            }
            Event::End(_) => {
                self.bytes += 8;
                self.depth -= 1;
                self.depth == 0
            }
            Event::Text(text)
            | Event::Code(text)
            | Event::Html(text)
            | Event::InlineHtml(text)
            | Event::FootnoteReference(text)
            | Event::InlineMath(text)
            | Event::DisplayMath(text) => {
                self.bytes += text.len();
                self.depth == 0
            }
            Event::SoftBreak | Event::HardBreak | Event::TaskListMarker(_) => {
                self.bytes += 1;
                self.depth == 0
            }
            Event::Rule => {
                self.bytes += 4;
                self.depth == 0
            }
        };

        if boundary && self.bytes >= TARGET_BYTES {
            self.separator_pending = true;
            self.bytes = 0;
        }
        Some(event)
    }
}
