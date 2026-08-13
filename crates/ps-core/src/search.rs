//! Incremental fuzzy search over registered projects.

use std::collections::HashMap;

use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::{Config, Matcher, Utf32String};

use crate::projects::Project;

struct SearchEntry {
    project: Project,
    haystack: Utf32String,
}

/// An in-memory project index updated one record at a time.
#[derive(Default)]
pub struct ProjectSearch {
    entries: HashMap<String, SearchEntry>,
}

impl ProjectSearch {
    /// Creates an empty search index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a project or replaces the existing project with the same identifier.
    pub fn upsert(&mut self, project: Project) {
        let text = format!("{} {}", project.name, project.path.display());
        self.entries.insert(
            project.id.clone(),
            SearchEntry {
                project,
                haystack: Utf32String::from(text),
            },
        );
    }

    /// Removes a project from the index and reports whether it existed.
    pub fn remove(&mut self, id: &str) -> bool {
        self.entries.remove(id).is_some()
    }

    /// Returns at most `limit` fuzzy matches ranked by relevance.
    pub fn search(&self, query: &str, limit: usize) -> Vec<Project> {
        let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
        let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
        let mut matches: Vec<_> = self
            .entries
            .values()
            .filter_map(|entry| {
                pattern
                    .score(entry.haystack.slice(..), &mut matcher)
                    .map(|score| (score, &entry.project))
            })
            .collect();
        matches.sort_unstable_by(|(left_score, left), (right_score, right)| {
            right_score
                .cmp(left_score)
                .then_with(|| left.name.cmp(&right.name))
                .then_with(|| left.id.cmp(&right.id))
        });
        matches
            .into_iter()
            .take(limit)
            .map(|(_, project)| project.clone())
            .collect()
    }
}
