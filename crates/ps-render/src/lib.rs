//! Markdown rendering for 1537paperstreet.

#![warn(missing_docs)]

mod chunks;
mod links;
mod mermaid;
mod pipeline;
mod sanitize;
mod toc;

pub use pipeline::{
    RenderOptions, RenderedDocument, render, render_document, render_document_with_options,
    render_project, render_project_with_options, render_with_options,
};
pub use toc::TocItem;
