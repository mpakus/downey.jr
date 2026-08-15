use std::collections::VecDeque;
use std::sync::OnceLock;

use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd, html};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

use crate::blocks::SpannedEvent;

static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();

pub(crate) struct Highlight<'input, I> {
    events: I,
    pending: VecDeque<SpannedEvent<'input>>,
    last_highlight: Option<(String, String, String)>,
}

impl<'input, I> Highlight<'input, I>
where
    I: Iterator<Item = SpannedEvent<'input>>,
{
    pub(crate) fn new(events: I) -> Self {
        Self {
            events,
            pending: VecDeque::new(),
            last_highlight: None,
        }
    }

    fn queue_code_block(
        &mut self,
        opening: SpannedEvent<'input>,
        language: &str,
        syntax: &SyntaxReference,
        syntax_set: &SyntaxSet,
    ) {
        let source_range = opening.1.clone();
        let mut original = VecDeque::from([opening]);
        let mut source = String::new();

        for event in self.events.by_ref() {
            match &event.0 {
                Event::Text(text) | Event::Code(text) => source.push_str(text),
                Event::SoftBreak | Event::HardBreak => source.push('\n'),
                _ => {}
            }
            let finished = matches!(&event.0, Event::End(TagEnd::CodeBlock));
            original.push_back(event);
            if finished {
                break;
            }
        }

        if let Some((cached_language, cached_source, html)) = &self.last_highlight
            && cached_language == language
            && cached_source == &source
        {
            self.pending
                .push_back((Event::Html(html.clone().into()), source_range));
            return;
        }

        match highlighted_html(&source, language, syntax, syntax_set) {
            Ok(html) => {
                self.last_highlight = Some((language.to_owned(), source, html.clone()));
                self.pending
                    .push_back((Event::Html(html.into()), source_range));
            }
            Err(_) => self.pending = original,
        }
    }
}

impl<'input, I> Iterator for Highlight<'input, I>
where
    I: Iterator<Item = SpannedEvent<'input>>,
{
    type Item = SpannedEvent<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.pending.pop_front() {
            return Some(event);
        }

        let event = self.events.next()?;
        let Some(language) = (match &event.0 {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                info.split_whitespace().next().map(str::to_owned)
            }
            _ => None,
        }) else {
            return Some(event);
        };
        let syntax_set = SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines);
        let Some(syntax) = syntax_set.find_syntax_by_token(&language) else {
            return Some(event);
        };

        self.queue_code_block(event, &language, syntax, syntax_set);
        self.pending.pop_front()
    }
}

fn highlighted_html(
    source: &str,
    language: &str,
    syntax: &SyntaxReference,
    syntax_set: &SyntaxSet,
) -> Result<String, syntect::Error> {
    let mut generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        syntax_set,
        ClassStyle::SpacedPrefixed { prefix: "syntax-" },
    );
    for line in LinesWithEndings::from(source) {
        generator.parse_html_for_line_which_includes_newline(line)?;
    }
    let mut escaped_language = String::new();
    html::push_html(
        &mut escaped_language,
        [Event::Text(language.into())].into_iter(),
    );

    Ok(format!(
        "<pre class=\"code\"><code class=\"language-{escaped_language}\">{}</code></pre>\n",
        generator.finalize()
    ))
}
