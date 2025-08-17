//! Strata Wiki - A modular, Rust-based wiki application
//! 
//! This crate provides a clean, modular architecture for building wiki applications
//! with separation of concerns and maintainable code structure.

pub mod components;
pub mod config;
pub mod errors;
pub mod services;
pub mod types;
pub mod utils;

// Re-export commonly used items
pub use config::Config;
pub use errors::WikiError;
pub use types::{AppState, DirEntry, SearchResult, MarkdownResult, TemplateContext};
pub use services::{FileService, SearchService, MarkdownService};
pub use components::{FabComponent, NavigationComponent, TemplateComponent};

// Re-export utility functions
pub use utils::{escape_html, escape_attr, last_modified_html, normalize_path, parse_query_param};
