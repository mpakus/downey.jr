use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ps_core::fsops;
use pulldown_cmark::{Event, Tag};

pub(crate) struct Links<'input, 'context, I> {
    events: I,
    project_root: &'context Path,
    canonical_root: Option<PathBuf>,
    document_dir: PathBuf,
    project_scope: &'context str,
    destinations: HashMap<String, String>,
    marker: std::marker::PhantomData<&'input str>,
}

impl<'input, 'context, I> Links<'input, 'context, I>
where
    I: Iterator<Item = Event<'input>>,
{
    pub(crate) fn new(
        events: I,
        project_root: &'context Path,
        document_path: &Path,
        project_scope: &'context str,
    ) -> Self {
        Self {
            events,
            project_root,
            canonical_root: fsops::resolve(project_root, Path::new("")).ok(),
            document_dir: document_path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .to_owned(),
            project_scope,
            destinations: HashMap::new(),
            marker: std::marker::PhantomData,
        }
    }

    fn rewrite(
        &mut self,
        destination: pulldown_cmark::CowStr<'input>,
    ) -> pulldown_cmark::CowStr<'input> {
        if let Some(rewritten) = self.destinations.get(destination.as_ref()) {
            return rewritten.clone().into();
        }

        match self.asset_url(&destination) {
            Ok(Some(url)) => {
                self.destinations
                    .insert(destination.to_string(), url.clone());
                url.into()
            }
            Ok(None) => destination,
            Err(()) => {
                self.destinations
                    .insert(destination.to_string(), "#invalid-path".to_owned());
                "#invalid-path".into()
            }
        }
    }

    fn asset_url(&self, destination: &str) -> Result<Option<String>, ()> {
        if destination.starts_with('#') || has_scheme(destination) {
            return Ok(None);
        }

        let (path, suffix) = split_suffix(destination);
        if path.is_empty() {
            return Ok(None);
        }
        if self.project_scope.is_empty()
            || !self
                .project_scope
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(());
        }

        let canonical_root = self.canonical_root.as_ref().ok_or(())?;
        let relative = self.document_dir.join(path);
        let resolved = fsops::resolve(self.project_root, &relative).map_err(|_| ())?;
        let relative = resolved.strip_prefix(canonical_root).map_err(|_| ())?;
        let relative = relative
            .iter()
            .map(|part| part.to_str().ok_or(()))
            .collect::<Result<Vec<_>, _>>()?
            .join("/");

        Ok(Some(format!(
            "asset://localhost/{}/{relative}{suffix}",
            self.project_scope
        )))
    }
}

impl<'input, I> Iterator for Links<'input, '_, I>
where
    I: Iterator<Item = Event<'input>>,
{
    type Item = Event<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.events.next()? {
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => Some(Event::Start(Tag::Link {
                link_type,
                dest_url: self.rewrite(dest_url),
                title,
                id,
            })),
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => Some(Event::Start(Tag::Image {
                link_type,
                dest_url: self.rewrite(dest_url),
                title,
                id,
            })),
            event => Some(event),
        }
    }
}

fn has_scheme(destination: &str) -> bool {
    destination
        .char_indices()
        .take_while(|(_, character)| !matches!(character, '/' | '?' | '#'))
        .any(|(_, character)| character == ':')
}

fn split_suffix(destination: &str) -> (&str, &str) {
    destination
        .char_indices()
        .find(|(_, character)| matches!(character, '?' | '#'))
        .map_or((destination, ""), |(index, _)| destination.split_at(index))
}
