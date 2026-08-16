//! Document reading: encoding, BOM, line endings, and write access.

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Error, Result, fsops};

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
/// Files larger than this open as source-only and skip Markdown rendering.
pub const SOURCE_ONLY_BYTES: u64 = 8 * 1024 * 1024;

/// Line ending used by a document on disk.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum LineEnding {
    /// Unix `\n`.
    Lf,
    /// Windows `\r\n`.
    CrLf,
}

/// Text encoding detected while reading a document.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum DocumentEncoding {
    /// Valid UTF-8, with or without a BOM.
    Utf8,
    /// Bytes that are not valid UTF-8.
    Binary,
}

/// A heading shown in the document table of contents.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TocEntry {
    /// Heading depth from 1 through 6.
    pub level: u8,
    /// Plain-text heading label.
    pub title: String,
    /// Unique HTML identifier assigned to the heading.
    pub id: String,
}

/// Source text and on-disk traits returned by `doc_source`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSource {
    /// Decoded text. Empty when the file is not valid UTF-8.
    pub text: String,
    /// Dominant line ending in the file.
    pub eol: LineEnding,
    /// Whether the file began with a UTF-8 BOM.
    pub bom: bool,
    /// Detected encoding.
    pub encoding: DocumentEncoding,
    /// Whether the document can be edited and saved.
    pub writable: bool,
    /// Why the document is read-only, when it is.
    pub readonly_reason: Option<String>,
}

/// Opening metadata returned by `doc_open`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMeta {
    /// Registered project that owns the file.
    pub project_id: String,
    /// NFC-normalized path relative to the project root.
    #[ts(type = "string")]
    pub rel_path: PathBuf,
    /// First heading title, or the file stem when the document has none.
    pub title: String,
    /// Lowercase hexadecimal BLAKE3 hash of the on-disk bytes.
    pub hash: String,
    /// Size of the on-disk file in bytes.
    pub size: u64,
    /// Whether the document can be edited and saved.
    pub writable: bool,
    /// Why the document is read-only or source-only, when it is.
    pub readonly_reason: Option<String>,
    /// Whether Markdown rendering was skipped.
    pub source_only: bool,
    /// Number of HTML chunks produced for the viewer.
    pub chunk_count: u32,
    /// Headings in source order.
    pub toc: Vec<TocEntry>,
}

/// Synchronous `doc_open` payload: metadata plus the first HTML chunk.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DocOpenResult {
    /// Document identity, hash, and table of contents.
    pub meta: DocumentMeta,
    /// First viewer chunk, when the document was rendered.
    pub first_chunk: Option<String>,
}

/// Remaining HTML sent on `doc://chunk` after `doc_open` returns.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DocChunkEvent {
    /// Registered project that owns the file.
    pub project_id: String,
    /// NFC-normalized path relative to the project root.
    #[ts(type = "string")]
    pub rel_path: PathBuf,
    /// Zero-based chunk index. Index `0` is returned by `doc_open`.
    pub index: u32,
    /// One `<section class="chunk">` fragment.
    pub html: String,
}

/// Terminal event on `doc://done` after every chunk has been sent.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DocDoneEvent {
    /// Registered project that owns the file.
    pub project_id: String,
    /// NFC-normalized path relative to the project root.
    #[ts(type = "string")]
    pub rel_path: PathBuf,
    /// Number of HTML chunks produced for the viewer.
    pub chunk_count: u32,
}

/// A document loaded from a project folder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedDocument {
    /// Source text and on-disk traits.
    pub source: DocumentSource,
    /// Lowercase hexadecimal BLAKE3 hash of the on-disk bytes.
    pub hash: String,
    /// Size of the on-disk file in bytes.
    pub size: u64,
    /// Whether Markdown rendering should be skipped.
    pub source_only: bool,
}

/// Reads a project document through [`fsops::resolve`].
pub fn read_doc(project_root: &Path, rel_path: &Path) -> Result<LoadedDocument> {
    let path = fsops::resolve(project_root, rel_path)?;
    let metadata = fs::symlink_metadata(&path)
        .map_err(|source| Error::io("open the document", &path, source))?;
    if metadata.is_dir() {
        return Err(Error::UnsafePath {
            path: rel_path.to_path_buf(),
            reason: "folders cannot be opened as documents",
        });
    }

    let bytes = fs::read(&path).map_err(|source| Error::io("read the document", &path, source))?;
    let writable_on_disk = is_writable(&path);
    let bom = bytes.starts_with(UTF8_BOM);
    let payload = if bom { &bytes[3..] } else { bytes.as_slice() };
    let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    let too_large = size > SOURCE_ONLY_BYTES;
    let (encoding, text, reason) = match std::str::from_utf8(payload) {
        Ok(text) => {
            let reason = if too_large {
                Some(String::from(
                    "This file is larger than 8 MB, so it opens as source only.",
                ))
            } else if !writable_on_disk {
                Some(String::from(
                    "This file cannot be saved because it is not writable.",
                ))
            } else {
                None
            };
            (DocumentEncoding::Utf8, text.to_owned(), reason)
        }
        Err(_) => (
            DocumentEncoding::Binary,
            String::new(),
            Some(String::from(
                "This file is not valid UTF-8, so it can only be viewed as source.",
            )),
        ),
    };

    Ok(LoadedDocument {
        source: DocumentSource {
            eol: detect_eol(payload),
            bom,
            encoding,
            writable: writable_on_disk && reason.is_none(),
            readonly_reason: reason,
            text,
        },
        hash: blake3::hash(&bytes).to_hex().to_string(),
        size,
        source_only: encoding == DocumentEncoding::Binary || too_large,
    })
}

fn detect_eol(bytes: &[u8]) -> LineEnding {
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\n' => return LineEnding::Lf,
            b'\r' => {
                return if bytes.get(index + 1) == Some(&b'\n') {
                    LineEnding::CrLf
                } else {
                    LineEnding::Lf
                };
            }
            _ => index += 1,
        }
    }
    LineEnding::Lf
}

fn is_writable(path: &Path) -> bool {
    OpenOptions::new().write(true).open(path).is_ok()
}
