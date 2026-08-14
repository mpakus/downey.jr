use std::collections::VecDeque;
use std::ops::Range;

use pulldown_cmark::Event;

pub(crate) type SpannedEvent<'input> = (Event<'input>, Range<usize>);

pub(crate) enum BlockEvent<'input> {
    Start(Event<'input>),
    Event(Event<'input>),
    End(Event<'input>),
}

/// Source metadata for one rendered top-level Markdown block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedBlock {
    /// Zero-based block position in source order.
    pub index: usize,
    /// Exact byte range occupied by the block in the Markdown source.
    pub source_range: Range<usize>,
    /// One-based source line on which the block starts.
    pub source_line: usize,
    /// Lowercase hexadecimal BLAKE3 hash of the source range.
    pub hash: String,
}

pub(crate) struct Blocks<'events, 'source, 'blocks, I> {
    events: I,
    markdown: &'source str,
    blocks: &'blocks mut Vec<RenderedBlock>,
    pending: VecDeque<BlockEvent<'events>>,
    scanned_to: usize,
    source_line: usize,
}

impl<'events, 'source, 'blocks, I> Blocks<'events, 'source, 'blocks, I>
where
    I: Iterator<Item = SpannedEvent<'events>>,
{
    pub(crate) fn new(
        events: I,
        markdown: &'source str,
        blocks: &'blocks mut Vec<RenderedBlock>,
    ) -> Self {
        Self {
            events,
            markdown,
            blocks,
            pending: VecDeque::new(),
            scanned_to: 0,
            source_line: 1,
        }
    }

    fn queue_block(&mut self) -> Option<()> {
        let (first, first_range) = self.events.next()?;
        let mut depth = usize::from(matches!(first, Event::Start(_)));
        let source_range = first_range.start..first_range.end;

        self.source_line += self.markdown.as_bytes()[self.scanned_to..source_range.start]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count();
        self.scanned_to = source_range.start;

        let index = self.blocks.len();
        let source = &self.markdown.as_bytes()[source_range.clone()];
        let hash = blake3::hash(source).to_hex().to_string();
        let opening = format!(
            "<section data-block=\"{index}\" data-src-line=\"{}\" data-hash=\"{hash}\">",
            self.source_line
        );
        self.blocks.push(RenderedBlock {
            index,
            source_range,
            source_line: self.source_line,
            hash,
        });

        self.pending
            .push_back(BlockEvent::Start(Event::Html(opening.into())));
        self.pending.push_back(BlockEvent::Event(first));
        while depth > 0 {
            let Some((event, _)) = self.events.next() else {
                break;
            };
            match event {
                Event::Start(_) => depth += 1,
                Event::End(_) => depth -= 1,
                _ => {}
            }
            self.pending.push_back(BlockEvent::Event(event));
        }
        self.pending
            .push_back(BlockEvent::End(Event::Html("</section>\n".into())));
        Some(())
    }
}

impl<'events, I> Iterator for Blocks<'events, '_, '_, I>
where
    I: Iterator<Item = SpannedEvent<'events>>,
{
    type Item = BlockEvent<'events>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.pending.pop_front() {
            return Some(event);
        }
        self.queue_block()?;
        self.pending.pop_front()
    }
}
