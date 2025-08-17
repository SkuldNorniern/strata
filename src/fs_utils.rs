use std::path::{Component, Path};
use std::{ffi::OsStr, fs};

use crate::types::WikiError;

pub fn ensure_safe_path(req_path: &str) -> Result<(), WikiError> {
    let path = Path::new(req_path);
    for comp in path.components() {
        match comp {
            Component::ParentDir => return Err(WikiError::InvalidPath),
            Component::Normal(seg) => {
                if seg.is_empty() {
                    return Err(WikiError::InvalidPath);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn normalize_request_path(req_path: &str) -> String {
    let trimmed = req_path.trim_start_matches('/');
    let mut parts = Vec::new();
    for part in trimmed.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        parts.push(part);
    }
    parts.join("/")
}

pub struct DirEntryInfo {
    pub name: String,
    pub is_dir: bool,
}

pub fn list_dir_filtered(dir: &Path) -> Result<Vec<DirEntryInfo>, WikiError> {
    let mut entries = Vec::new();
    for entry_result in fs::read_dir(dir)? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;
        let name_os = entry.file_name();
        let name = name_os.to_string_lossy();
        if name.starts_with('.') {
            continue; // hide dotfiles
        }
        entries.push(DirEntryInfo { name: name.to_string(), is_dir: file_type.is_dir() });
    }
    // Directories first, then files; both alphabetically
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(entries)
}

pub fn is_markdown(path: &Path) -> bool {
    matches!(path.extension().and_then(OsStr::to_str), Some("md"))
}

pub fn escape_html(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

pub fn escape_attr(input: &str) -> String {
    escape_html(input)
}


