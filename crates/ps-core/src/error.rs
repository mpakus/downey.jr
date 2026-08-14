//! Errors returned by core application services.

use std::io;
use std::path::PathBuf;

/// The error type shared by `ps-core` modules.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The application data directory could not be determined.
    #[error(
        "The application data directory could not be found. Set PAPERSTREET_HOME and try again."
    )]
    HomeDirectoryUnavailable,

    /// A filesystem operation on application data failed.
    #[error("Couldn't {action} at '{}'.", path.display())]
    Io {
        /// A user-readable description of the attempted operation.
        action: &'static str,
        /// The path involved in the operation.
        path: PathBuf,
        /// The underlying operating-system error.
        #[source]
        source: io::Error,
    },

    /// JSON could not be encoded or decoded.
    #[error("Couldn't {action} JSON at '{}'.", path.display())]
    Json {
        /// A user-readable description of the attempted operation.
        action: &'static str,
        /// The path involved in the operation.
        path: PathBuf,
        /// The underlying JSON error.
        #[source]
        source: serde_json::Error,
    },

    /// A stored schema is newer than the application supports or has no migration path.
    #[error(
        "This data uses schema version {found}, but this version of the app supports version {supported}."
    )]
    UnsupportedSchema {
        /// The schema version found on disk.
        found: u32,
        /// The newest supported schema version.
        supported: u32,
    },

    /// A configuration value is outside its supported range.
    #[error("The setting '{field}' must be between {min} and {max}; found {actual}.")]
    InvalidConfig {
        /// The invalid configuration field.
        field: &'static str,
        /// The smallest accepted value.
        min: i64,
        /// The largest accepted value.
        max: i64,
        /// The value found in the configuration.
        actual: i64,
    },

    /// A project record contains a value that cannot be stored safely.
    #[error("The project record is invalid: {reason}.")]
    InvalidProject {
        /// A user-readable explanation of the invalid value.
        reason: &'static str,
    },

    /// A project-relative path cannot be used safely.
    #[error("The path '{}' cannot be used: {reason}.", path.display())]
    UnsafePath {
        /// The rejected path.
        path: PathBuf,
        /// A user-readable explanation of why the path was rejected.
        reason: &'static str,
    },

    /// A resolved path points outside its project directory.
    #[error("The path '{}' points outside the project directory.", path.display())]
    PathOutsideProject {
        /// The rejected path.
        path: PathBuf,
    },

    /// A file or directory already uses the requested name.
    #[error(
        "A file or folder named '{}' already exists. Try '{suggested_name}'.",
        path.display()
    )]
    NameConflict {
        /// The occupied destination path.
        path: PathBuf,
        /// The next available file name in the destination directory.
        suggested_name: String,
    },

    /// Moving selected items to the operating system Trash failed.
    #[error("Couldn't move the selected items to Trash.")]
    Trash {
        /// The underlying Trash operation error.
        #[source]
        source: trash::Error,
    },

    /// Starting or configuring filesystem observation failed.
    #[error("Couldn't {action} for '{}'.", path.display())]
    Notify {
        /// A user-readable description of the attempted watcher operation.
        action: &'static str,
        /// The watched project path.
        path: PathBuf,
        /// The underlying notification backend error.
        #[source]
        source: notify::Error,
    },

    /// A requested project does not exist in the registry.
    #[error("The project '{id}' is no longer in the project list.")]
    ProjectNotFound {
        /// The missing project identifier.
        id: String,
    },

    /// A project timestamp could not be represented in the storage format.
    #[error("Couldn't create a timestamp for the project record.")]
    TimeFormat {
        /// The underlying timestamp formatting error.
        #[source]
        source: time::error::Format,
    },
}

impl Error {
    pub(crate) fn io(action: &'static str, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }

    pub(crate) fn json(
        action: &'static str,
        path: impl Into<PathBuf>,
        source: serde_json::Error,
    ) -> Self {
        Self::Json {
            action,
            path: path.into(),
            source,
        }
    }
}

/// A result returned by `ps-core`.
pub type Result<T> = std::result::Result<T, Error>;
