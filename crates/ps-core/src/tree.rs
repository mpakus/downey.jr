//! Lazy, one-level project tree loading.

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::{Error, Result, fsops};

/// The filesystem kind represented by a tree node.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TreeNodeKind {
    /// A directory that can be expanded lazily.
    Directory,
    /// A regular file.
    File,
    /// A symbolic link, which the tree reader does not follow.
    Symlink,
    /// Another filesystem object such as a socket or named pipe.
    Other,
}

/// One immediate child of a project directory.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    /// NFC-normalized display name.
    pub name: String,
    /// NFC-normalized path relative to the canonical project root.
    pub rel_path: PathBuf,
    /// Filesystem kind without following symbolic links.
    pub kind: TreeNodeKind,
}

struct SortableNode {
    node: TreeNode,
    folded_name: String,
}

/// Reads exactly one directory level inside a project.
///
/// Directories sort before other nodes, followed by a case-insensitive natural
/// name order. Names beginning with `.` are omitted unless `show_hidden` is true.
pub fn read_dir(project_root: &Path, rel_path: &Path, show_hidden: bool) -> Result<Vec<TreeNode>> {
    let canonical_root = project_root
        .canonicalize()
        .map_err(|source| Error::io("open the project directory", project_root, source))?;
    let directory = fsops::resolve(project_root, rel_path)?
        .canonicalize()
        .map_err(|source| Error::io("open the tree folder", rel_path, source))?;
    if !directory.is_dir() {
        return Err(Error::UnsafePath {
            path: rel_path.to_path_buf(),
            reason: "the tree path is not a folder",
        });
    }
    let directory_rel =
        directory
            .strip_prefix(&canonical_root)
            .map_err(|_| Error::PathOutsideProject {
                path: directory.clone(),
            })?;

    let entries = fs::read_dir(&directory)
        .map_err(|source| Error::io("read the tree folder", &directory, source))?;
    let mut nodes = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| Error::io("read a tree entry", &directory, source))?;
        let raw_name = entry.file_name();
        let name = raw_name
            .to_str()
            .ok_or_else(|| Error::UnsafePath {
                path: entry.path(),
                reason: "file names must be valid Unicode",
            })?
            .nfc()
            .collect::<String>();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|source| Error::io("inspect a tree entry", entry.path(), source))?;
        let kind = if file_type.is_dir() {
            TreeNodeKind::Directory
        } else if file_type.is_file() {
            TreeNodeKind::File
        } else if file_type.is_symlink() {
            TreeNodeKind::Symlink
        } else {
            TreeNodeKind::Other
        };
        nodes.push(SortableNode {
            folded_name: name.to_lowercase(),
            node: TreeNode {
                rel_path: directory_rel.join(&name),
                name,
                kind,
            },
        });
    }

    nodes.sort_unstable_by(|left, right| {
        directory_rank(left.node.kind)
            .cmp(&directory_rank(right.node.kind))
            .then_with(|| natural_cmp(&left.folded_name, &right.folded_name))
            .then_with(|| left.node.name.cmp(&right.node.name))
    });
    Ok(nodes.into_iter().map(|node| node.node).collect())
}

fn directory_rank(kind: TreeNodeKind) -> u8 {
    u8::from(kind != TreeNodeKind::Directory)
}

fn natural_cmp(left: &str, right: &str) -> Ordering {
    let left = left.as_bytes();
    let right = right.as_bytes();
    let mut left_index = 0;
    let mut right_index = 0;
    while left_index < left.len() && right_index < right.len() {
        if left[left_index].is_ascii_digit() && right[right_index].is_ascii_digit() {
            let left_end = digit_run_end(left, left_index);
            let right_end = digit_run_end(right, right_index);
            let left_significant = trim_leading_zeroes(&left[left_index..left_end]);
            let right_significant = trim_leading_zeroes(&right[right_index..right_end]);
            let numeric = left_significant
                .len()
                .cmp(&right_significant.len())
                .then_with(|| left_significant.cmp(right_significant))
                .then_with(|| (left_end - left_index).cmp(&(right_end - right_index)));
            if numeric != Ordering::Equal {
                return numeric;
            }
            left_index = left_end;
            right_index = right_end;
        } else {
            let ordering = left[left_index].cmp(&right[right_index]);
            if ordering != Ordering::Equal {
                return ordering;
            }
            left_index += 1;
            right_index += 1;
        }
    }
    left.len().cmp(&right.len())
}

fn digit_run_end(value: &[u8], start: usize) -> usize {
    value[start..]
        .iter()
        .position(|byte| !byte.is_ascii_digit())
        .map_or(value.len(), |offset| start + offset)
}

fn trim_leading_zeroes(value: &[u8]) -> &[u8] {
    let first_nonzero = value
        .iter()
        .position(|byte| *byte != b'0')
        .unwrap_or(value.len().saturating_sub(1));
    &value[first_nonzero..]
}
