//! Markdown rendering for 1537paperstreet.

#![warn(missing_docs)]

mod links;
mod mermaid;
mod pipeline;
mod toc;

pub use pipeline::{RenderedDocument, render, render_document, render_project};
pub use toc::TocItem;
