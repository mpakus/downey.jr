//! Core application logic for 1537paperstreet.

#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod fsops;
pub mod paths;
pub mod projects;
pub mod search;
pub mod store;
pub mod tree;
pub mod watch;

pub use error::{Error, Result};
