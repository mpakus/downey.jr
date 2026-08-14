use std::collections::VecDeque;

use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

pub(crate) struct Mermaid<'input, I> {
    events: I,
    pending: VecDeque<Event<'input>>,
}

impl<'input, I> Mermaid<'input, I>
where
    I: Iterator<Item = Event<'input>>,
{
    pub(crate) fn new(events: I) -> Self {
        Self {
            events,
            pending: VecDeque::new(),
        }
    }

    fn queue_figure(&mut self) {
        let mut source = String::new();

        for event in self.events.by_ref() {
            match event {
                Event::End(TagEnd::CodeBlock) => break,
                Event::Text(text) | Event::Code(text) => source.push_str(&text),
                Event::SoftBreak | Event::HardBreak => source.push('\n'),
                _ => {}
            }
        }

        let hash = blake3::hash(source.as_bytes()).to_hex();
        self.pending.push_back(Event::Html(
            format!("<figure class=\"mermaid\" data-hash=\"{hash}\"><template>").into(),
        ));
        self.pending.push_back(Event::Text(source.into()));
        self.pending
            .push_back(Event::Html("</template></figure>\n".into()));
    }
}

impl<'input, I> Iterator for Mermaid<'input, I>
where
    I: Iterator<Item = Event<'input>>,
{
    type Item = Event<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.pending.pop_front() {
            return Some(event);
        }

        match self.events.next()? {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info)))
                if info.split_whitespace().next() == Some("mermaid") =>
            {
                self.queue_figure();
                self.pending.pop_front()
            }
            event => Some(event),
        }
    }
}
