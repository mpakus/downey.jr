//! Persisted file-tree expansion state.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::store::VersionedDocument;
use crate::{Error, Result};

/// On-disk map of expanded directories keyed by project identifier.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiState {
    /// Storage schema version.
    pub schema_version: u32,
    /// Project-relative directory paths that were expanded in the tree.
    #[serde(default)]
    pub expanded: BTreeMap<String, Vec<String>>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            schema_version: <Self as VersionedDocument>::SCHEMA_VERSION,
            expanded: BTreeMap::new(),
        }
    }
}

impl VersionedDocument for UiState {
    const SCHEMA_VERSION: u32 = 1;

    fn migrate(value: Value, from: u32) -> Result<Value> {
        let _ = value;
        Err(Error::UnsupportedSchema {
            found: from,
            supported: Self::SCHEMA_VERSION,
        })
    }

    fn validate(&self) -> Result<()> {
        if self.schema_version != Self::SCHEMA_VERSION {
            return Err(Error::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::SCHEMA_VERSION,
            });
        }
        for paths in self.expanded.values() {
            for path in paths {
                validate_expanded_rel(Path::new(path))?;
            }
        }
        Ok(())
    }
}

impl UiState {
    /// Returns the expanded directories for `project_id`.
    pub fn expanded_for(&self, project_id: &str) -> Vec<String> {
        self.expanded.get(project_id).cloned().unwrap_or_default()
    }

    /// Replaces the expanded directories for `project_id`.
    pub fn set_expanded(&mut self, project_id: String, paths: Vec<PathBuf>) -> Result<()> {
        let mut cleaned = Vec::new();
        for path in paths {
            validate_expanded_rel(&path)?;
            if path.as_os_str().is_empty() {
                continue;
            }
            cleaned.push(path.to_string_lossy().replace('\\', "/"));
        }
        cleaned.sort();
        cleaned.dedup();
        if cleaned.is_empty() {
            self.expanded.remove(&project_id);
        } else {
            self.expanded.insert(project_id, cleaned);
        }
        Ok(())
    }

    /// Drops expansion state for a project that was removed from the registry.
    pub fn remove_project(&mut self, project_id: &str) {
        self.expanded.remove(project_id);
    }
}

fn validate_expanded_rel(path: &Path) -> Result<()> {
    if path.is_absolute() {
        return Err(Error::UnsafePath {
            path: path.to_path_buf(),
            reason: "expanded tree paths must be project-relative",
        });
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(Error::UnsafePath {
                path: path.to_path_buf(),
                reason: "expanded tree paths cannot contain '..'",
            });
        }
    }
    Ok(())
}
