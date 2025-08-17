use std::path::Path;
use time::OffsetDateTime;

/// Escape HTML special characters
pub fn escape_html(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

/// Escape HTML attribute values
pub fn escape_attr(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

/// Generate last modified metadata HTML
pub fn last_modified_html(path: &Path) -> String {
    match std::fs::metadata(path).and_then(|m| m.modified()) {
        Ok(mtime) => {
            match mtime.duration_since(std::time::UNIX_EPOCH) {
                Ok(dur) => {
                    let secs = dur.as_secs() as i64;
                    let datetime = OffsetDateTime::from_unix_timestamp(secs).ok();
                    if let Some(dt) = datetime {
                        let fmt = time::format_description::well_known::Rfc3339;
                        if let Ok(s) = dt.format(&fmt) {
                            return format!("<p class=\"meta\">Last modified: {}</p>", escape_html(&s));
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

/// Normalize request path
pub fn normalize_path(path: &str) -> String {
    let mut normalized = path.trim_matches('/').to_string();
    if normalized.is_empty() {
        normalized = "".to_string();
    }
    normalized
}

/// Parse query parameter with basic URL decoding
pub fn parse_query_param(query: &str, param: &str) -> String {
    let query_string = query.trim_start_matches('?');
    for pair in query_string.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == param {
                // Basic URL decoding (replace %20 with space, etc.)
                return value.replace("%20", " ")
                    .replace("%21", "!")
                    .replace("%22", "\"")
                    .replace("%23", "#")
                    .replace("%24", "$")
                    .replace("%25", "%")
                    .replace("%26", "&")
                    .replace("%27", "'")
                    .replace("%28", "(")
                    .replace("%29", ")")
                    .replace("%2A", "*")
                    .replace("%2B", "+")
                    .replace("%2C", ",")
                    .replace("%2D", "-")
                    .replace("%2E", ".")
                    .replace("%2F", "/")
                    .replace("%3A", ":")
                    .replace("%3B", ";")
                    .replace("%3C", "<")
                    .replace("%3D", "=")
                    .replace("%3E", ">")
                    .replace("%3F", "?")
                    .replace("%40", "@")
                    .replace("%5B", "[")
                    .replace("%5C", "\\")
                    .replace("%5D", "]")
                    .replace("%5E", "^")
                    .replace("%5F", "_")
                    .replace("%60", "`")
                    .replace("%7B", "{")
                    .replace("%7C", "|")
                    .replace("%7D", "}")
                    .replace("%7E", "~");
            }
        }
    }
    String::new()
}
