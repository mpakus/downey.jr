//! Persistent project registry.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use ts_rs::TS;
use ulid::Ulid;

use crate::store::{JsonStore, StoreWarning, VersionedDocument};
use crate::{Error, Result};

/// A registered folder containing Markdown documents.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
pub struct Project {
    /// Stable, lexicographically sortable ULID.
    pub id: String,
    /// User-visible project name.
    pub name: String,
    /// Absolute path to the project folder.
    #[ts(type = "string")]
    pub path: PathBuf,
    /// RFC 3339 timestamp at which the project was registered.
    pub added_at: String,
    /// RFC 3339 timestamp of the most recent open operation.
    pub last_opened_at: Option<String>,
    /// Whether the project stays above unpinned projects.
    pub pinned: bool,
    /// Optional theme accent override.
    pub accent: Option<String>,
    /// Last opened project-relative file path.
    pub last_file: Option<String>,
    /// Cached path availability, omitted from persistent storage.
    #[serde(skip, default)]
    pub available: Option<bool>,
}

/// Arguments for `projects_list`.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsListQuery {
    /// Optional fuzzy query over project names and paths.
    pub query: Option<String>,
    /// Maximum number of projects to return.
    pub limit: u32,
    /// Number of ranked matches to skip.
    pub offset: u32,
}

/// A page of registered projects.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsListResult {
    /// Projects in the requested page.
    pub items: Vec<Project>,
    /// Number of projects matching the query before paging.
    pub total: u32,
}

impl Project {
    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::InvalidProject {
                reason: "the name is empty",
            });
        }
        if !self.path.is_absolute() {
            return Err(Error::InvalidProject {
                reason: "the folder path is not absolute",
            });
        }
        if Ulid::from_string(&self.id).is_err() {
            return Err(Error::InvalidProject {
                reason: "the identifier is not a ULID",
            });
        }
        Ok(())
    }
}

/// The versioned document stored in `projects.json`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectsDocument {
    /// Storage schema version.
    pub schema_version: u32,
    /// Registered projects in display order.
    pub projects: Vec<Project>,
}

impl Default for ProjectsDocument {
    fn default() -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            projects: Vec::new(),
        }
    }
}

impl VersionedDocument for ProjectsDocument {
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
        for project in &self.projects {
            project.validate()?;
        }
        Ok(())
    }
}

/// CRUD service for the persistent project registry.
pub struct ProjectStore {
    store: JsonStore<ProjectsDocument>,
}

impl ProjectStore {
    /// Opens the project registry at `path`.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        JsonStore::open(path).map(|store| Self { store })
    }

    /// Returns projects in their stored display order.
    pub fn list(&self) -> &[Project] {
        &self.store.value().projects
    }

    /// Returns a registered project by identifier.
    pub fn get(&self, id: &str) -> Result<&Project> {
        self.store
            .value()
            .projects
            .iter()
            .find(|project| project.id == id)
            .ok_or_else(|| Error::ProjectNotFound { id: id.to_owned() })
    }

    /// Registers an absolute project folder path.
    pub fn add(&mut self, name: impl Into<String>, path: PathBuf) -> Result<Project> {
        let project = Project {
            id: Ulid::generate().to_string(),
            name: name.into(),
            path,
            added_at: now_rfc3339()?,
            last_opened_at: None,
            pinned: false,
            accent: None,
            last_file: None,
            available: None,
        };
        project.validate()?;
        let returned = project.clone();
        self.store
            .update(|document| document.projects.push(project));
        Ok(returned)
    }

    /// Changes the user-visible name of a registered project.
    pub fn rename(&mut self, id: &str, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(Error::InvalidProject {
                reason: "the name is empty",
            });
        }
        let project = self.find_mut(id)?;
        project.name = name;
        self.mark_dirty();
        Ok(())
    }

    /// Removes only a registry record and never touches its project folder.
    pub fn remove(&mut self, id: &str) -> Result<()> {
        let Some(index) = self
            .store
            .value()
            .projects
            .iter()
            .position(|project| project.id == id)
        else {
            return Err(Error::ProjectNotFound { id: id.to_owned() });
        };
        self.store.update(|document| {
            document.projects.remove(index);
        });
        Ok(())
    }

    /// Checks a project folder on demand and caches the result in memory.
    pub fn refresh_availability(&mut self, id: &str) -> Result<bool> {
        let project = self.find_mut(id)?;
        let available = project.path.is_dir();
        project.available = Some(available);
        self.mark_dirty();
        Ok(available)
    }

    /// Immediately persists pending registry changes.
    pub fn flush(&mut self) -> Result<bool> {
        self.store.flush()
    }

    /// Persists pending changes and closes the registry.
    pub fn close(self) -> Result<()> {
        self.store.close()
    }

    /// Returns and clears a recoverable warning from opening the registry.
    pub fn take_warning(&mut self) -> Option<StoreWarning> {
        self.store.take_warning()
    }

    fn find_mut(&mut self, id: &str) -> Result<&mut Project> {
        self.store
            .value_mut()
            .projects
            .iter_mut()
            .find(|project| project.id == id)
            .ok_or_else(|| Error::ProjectNotFound { id: id.to_owned() })
    }

    fn mark_dirty(&mut self) {
        self.store.mark_dirty();
    }
}

fn now_rfc3339() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|source| Error::TimeFormat { source })
}
