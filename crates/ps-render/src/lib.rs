//! Markdown rendering for 1537paperstreet.

#![warn(missing_docs)]

mod pipeline;
mod toc;

pub use pipeline::{RenderedDocument, render, render_document};
pub use toc::TocItem;
