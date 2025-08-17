use std::path::Path;

/// Determine content type for a file based on its extension
pub fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()) {
        Some(ref ext) if ext == "html" => "text/html; charset=utf-8",
        Some(ref ext) if ext == "css" => "text/css; charset=utf-8",
        Some(ref ext) if ext == "js" => "application/javascript; charset=utf-8",
        Some(ref ext) if ext == "json" => "application/json; charset=utf-8",
        Some(ref ext) if ext == "svg" => "image/svg+xml",
        Some(ref ext) if ext == "png" => "image/png",
        Some(ref ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
        Some(ref ext) if ext == "gif" => "image/gif",
        Some(ref ext) if ext == "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// Generate FAB actions for Markdown pages
pub fn generate_fab_actions(req_path: &str) -> String {
    // Skip actions for empty paths (home page)
    if req_path.is_empty() {
        return String::new();
    }
    
    let raw_href = if req_path.ends_with(".md") {
        format!("/raw/{}", req_path)
    } else {
        format!("/raw/{}.md", req_path)
    };
    let edit_href = format!("file://wiki/{}.md", req_path);
    
    format!(
        "<a href=\"{}\" title=\"View raw\" class=\"fab-action-raw\"></a><a href=\"{}\" title=\"Edit this page\" class=\"fab-action-edit\"></a>",
        crate::fs_utils::escape_attr(&raw_href), 
        crate::fs_utils::escape_attr(&edit_href)
    )
}

/// Generate last modified metadata HTML
pub fn last_modified_html(path: &Path) -> String {
    match std::fs::metadata(path).and_then(|m| m.modified()) {
        Ok(mtime) => {
            match mtime.duration_since(std::time::UNIX_EPOCH) {
                Ok(dur) => {
                    let secs = dur.as_secs() as i64;
                    let datetime = time::OffsetDateTime::from_unix_timestamp(secs).ok();
                    if let Some(dt) = datetime {
                        let fmt = time::format_description::well_known::Rfc3339;
                        if let Ok(s) = dt.format(&fmt) {
                            return format!("<p class=\"meta\">Last modified: {}</p>", crate::fs_utils::escape_html(&s));
                        }
                    }
                    String::new()
                }
                Err(_) => String::new(),
            }
        }
        Err(_) => String::new(),
    }
}

// This function is no longer used - TOC is now in the sidebar
