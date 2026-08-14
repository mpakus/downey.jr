use std::collections::VecDeque;

use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd, html};

use crate::blocks::SpannedEvent;

pub(crate) struct Mermaid<'input, 'context, I> {
    events: I,
    pending: VecDeque<SpannedEvent<'input>>,
    placeholder_prefix: &'context str,
    figures: &'context mut Vec<String>,
}

impl<'input, 'context, I> Mermaid<'input, 'context, I>
where
    I: Iterator<Item = SpannedEvent<'input>>,
{
    pub(crate) fn new(
        events: I,
        placeholder_prefix: &'context str,
        figures: &'context mut Vec<String>,
    ) -> Self {
        Self {
            events,
            pending: VecDeque::new(),
            placeholder_prefix,
            figures,
        }
    }

    fn queue_figure(&mut self, source_range: std::ops::Range<usize>) {
        let mut source = String::new();

        for (event, _) in self.events.by_ref() {
            match event {
                Event::End(TagEnd::CodeBlock) => break,
                Event::Text(text) | Event::Code(text) => source.push_str(&text),
                Event::SoftBreak | Event::HardBreak => source.push('\n'),
                _ => {}
            }
        }

        let hash = blake3::hash(source.as_bytes()).to_hex();
        let figure_events = [
            Event::Html(
                format!("<figure class=\"mermaid\" data-hash=\"{hash}\"><template>").into(),
            ),
            Event::Text(source.into()),
            Event::Html("</template></figure>\n".into()),
        ];
        let mut figure = String::new();
        html::push_html(&mut figure, figure_events.into_iter());

        let index = self.figures.len();
        self.figures.push(figure);
        self.pending.push_back((
            Event::Text(format!("{}{index}__", self.placeholder_prefix).into()),
            source_range,
        ));
    }
}

impl<'input, I> Iterator for Mermaid<'input, '_, I>
where
    I: Iterator<Item = SpannedEvent<'input>>,
{
    type Item = SpannedEvent<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.pending.pop_front() {
            return Some(event);
        }

        match self.events.next()? {
            (Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))), source_range)
                if info.split_whitespace().next() == Some("mermaid") =>
            {
                self.queue_figure(source_range);
                self.pending.pop_front()
            }
            event => Some(event),
        }
    }
}
