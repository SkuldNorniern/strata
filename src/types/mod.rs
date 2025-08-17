use std::sync::Arc;
use std::path::PathBuf;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub base_dir: Arc<PathBuf>,
    pub static_dir: Arc<PathBuf>,
}

/// Directory entry information
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub path: PathBuf,
}

/// Search result information
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub path: String,
    pub excerpt: String,
    pub relevance: f32,
}

/// Markdown rendering result
#[derive(Debug, Clone)]
pub struct MarkdownResult {
    pub html: String,
    pub toc: String,
    pub title: Option<String>,
}

/// Template rendering context
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub title: String,
    pub content: String,
    pub sidebar: String,
    pub fab: String,
    pub toc: Option<String>,
}
