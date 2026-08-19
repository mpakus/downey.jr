//! Rotating application logs that never store document text.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::{Error, Result};

const DEFAULT_MAX_BYTES: u64 = 1024 * 1024;
const BACKUPS: u8 = 2;

/// A mutex-protected log file with size-based rotation.
pub struct FileLog {
    inner: Mutex<Inner>,
}

struct Inner {
    path: PathBuf,
    file: File,
    max_bytes: u64,
}

impl FileLog {
    /// Opens (or creates) `path` for appending.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Self::open_with_limit(path, DEFAULT_MAX_BYTES)
    }

    /// Opens a log with an explicit rotation threshold. Used by tests.
    pub fn open_with_limit(path: impl Into<PathBuf>, max_bytes: u64) -> Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|source| Error::io("create the log directory", parent, source))?;
        }
        let file = open_append(&path)?;
        Ok(Self {
            inner: Mutex::new(Inner {
                path,
                file,
                max_bytes,
            }),
        })
    }

    /// Appends an informational line.
    ///
    /// Log lines record actions, not document text.
    ///
    /// ```
    /// use std::fs;
    /// use ps_core::log::FileLog;
    ///
    /// let dir = tempfile::tempdir().unwrap();
    /// let path = dir.path().join("app.log");
    /// let log = FileLog::open(&path).unwrap();
    /// log.info("opened project").unwrap();
    /// let text = fs::read_to_string(&path).unwrap();
    /// assert!(text.contains("opened project"));
    /// assert!(!text.contains("# Heading"));
    /// ```
    pub fn info(&self, message: &str) -> Result<()> {
        self.write("info", message)
    }

    /// Appends a warning line.
    pub fn warn(&self, message: &str) -> Result<()> {
        self.write("warn", message)
    }

    /// Appends an error line.
    pub fn error(&self, message: &str) -> Result<()> {
        self.write("error", message)
    }

    fn write(&self, level: &str, message: &str) -> Result<()> {
        let timestamp = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|source| Error::TimeFormat { source })?;
        let line = format!("{timestamp} {level} {message}\n");
        let mut inner = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        inner.rotate_if_needed()?;
        inner
            .file
            .write_all(line.as_bytes())
            .and_then(|()| inner.file.flush())
            .map_err(|source| Error::io("write the log", &inner.path, source))
    }
}

impl Inner {
    fn rotate_if_needed(&mut self) -> Result<()> {
        let len = self
            .file
            .metadata()
            .map_err(|source| Error::io("inspect the log", &self.path, source))?
            .len();
        if len < self.max_bytes {
            return Ok(());
        }
        let path = self.path.clone();
        let mut closed = open_append(&path)?;
        std::mem::swap(&mut self.file, &mut closed);
        drop(closed);
        rotate_backups(&path)?;
        self.file = open_append(&path)?;
        Ok(())
    }
}

fn open_append(path: &Path) -> Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| Error::io("open the log", path, source))
}

fn rotate_backups(path: &Path) -> Result<()> {
    for index in (1..=BACKUPS).rev() {
        let from = backup_path(path, index);
        let to = backup_path(path, index + 1);
        if index == BACKUPS {
            let _ = fs::remove_file(&from);
            continue;
        }
        if from.exists() {
            fs::rename(&from, &to).map_err(|source| Error::io("rotate the log", &from, source))?;
        }
    }
    if path.exists() {
        fs::rename(path, backup_path(path, 1))
            .map_err(|source| Error::io("rotate the log", path, source))?;
    }
    Ok(())
}

fn backup_path(path: &Path, index: u8) -> PathBuf {
    let mut file_name = path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("app.log"))
        .to_os_string();
    file_name.push(format!(".{index}"));
    path.with_file_name(file_name)
}
