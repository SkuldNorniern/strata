use std::fs;
use std::path::Path;

use crate::fs_utils::{escape_attr, escape_html};
use crate::types::WikiError;

/// Build a nested sidebar navigation HTML from the base wiki directory.
/// Limits recursion to `max_depth` to keep the UI readable.
pub fn build_sidebar_html(base_dir: &Path, current_req_path: &str) -> Result<String, WikiError> {
    let mut html = String::new();
    html.push_str("<nav class=\"sidebar-nav\">");
    html.push_str("<div class=\"sidebar-title\">Navigation</div>");
    let mut root_prefix = String::new();
    build_dir_ul(
        base_dir,
        &mut html,
        &mut root_prefix,
        0,
        3,
        current_req_path,
    )?;
    html.push_str("</nav>");
    Ok(html)
}

/// Build sidebar with TOC included
pub fn build_sidebar_with_toc(base_dir: &Path, current_req_path: &str, toc_html: &str) -> Result<String, WikiError> {
    let mut html = String::new();
    html.push_str("<nav class=\"sidebar-nav\">");
    html.push_str("<div class=\"sidebar-title\">Navigation</div>");
    let mut root_prefix = String::new();
    build_dir_ul(
        base_dir,
        &mut html,
        &mut root_prefix,
        0,
        3,
        current_req_path,
    )?;
    
    // Add TOC to sidebar if it exists
    if !toc_html.is_empty() {
        html.push_str("<div class=\"sidebar-toc\">");
        html.push_str("<div class=\"sidebar-toc-title\">On this page</div>");
        html.push_str(toc_html);
        html.push_str("</div>");
    }
    
    html.push_str("</nav>");
    Ok(html)
}

fn build_dir_ul(
    dir: &Path,
    html: &mut String,
    req_prefix: &mut String,
    depth: usize,
    max_depth: usize,
    current_req_path: &str,
) -> Result<(), WikiError> {
    if depth > max_depth {
        return Ok(());
    }

    let mut entries = Vec::new();
    for entry_res in fs::read_dir(dir)? {
        let entry = entry_res?;
        let file_type = entry.file_type()?;
        let name_os = entry.file_name();
        let name = name_os.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        entries.push((name.to_string(), file_type.is_dir()));
    }
    entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    html.push_str("<ul class=\"nav-list\">");
    for (name, is_dir) in entries {
        if is_dir {
            // Directory node
            let child_dir = dir.join(&name);
            let saved_len = req_prefix.len();
            if !req_prefix.is_empty() {
                req_prefix.push('/');
            }
            req_prefix.push_str(&name);

            let href = format!("/{}", req_prefix);
            let is_current_branch = current_req_path.starts_with(&*req_prefix);
            let is_current_exact = current_req_path == *req_prefix;

            html.push_str("<li class=\"nav-item dir\">");
            html.push_str("<details");
            if is_current_branch {
                html.push_str(" open");
            }
            html.push_str(">");
            html.push_str("<summary>");
            if is_current_exact {
                html.push_str(&format!(
                    "<a class=\"active\" href=\"{}\">{}/</a>",
                    escape_attr(&href),
                    escape_html(&name)
                ));
            } else {
                html.push_str(&format!(
                    "<a href=\"{}\">{}/</a>",
                    escape_attr(&href),
                    escape_html(&name)
                ));
            }
            html.push_str("</summary>");
            build_dir_ul(&child_dir, html, req_prefix, depth + 1, max_depth, current_req_path)?;
            html.push_str("</details>");
            html.push_str("</li>");

            req_prefix.truncate(saved_len);
        } else if name.ends_with(".md") {
            // Markdown file node (skip index.md/README.md so the folder link represents it)
            let lower = name.to_ascii_lowercase();
            if lower == "index.md" || lower == "readme.md" {
                continue;
            }
            let title = name.trim_end_matches(".md");
            let href = if req_prefix.is_empty() {
                format!("/{}", title)
            } else {
                format!("/{}/{}", req_prefix, title)
            };
            let active = current_req_path == href.trim_start_matches('/');
            html.push_str("<li class=\"nav-item file\">");
            if active {
                html.push_str(&format!(
                    "<a class=\"active\" href=\"{}\">{}</a>",
                    escape_attr(&href),
                    escape_html(title)
                ));
            } else {
                html.push_str(&format!(
                    "<a href=\"{}\">{}</a>",
                    escape_attr(&href),
                    escape_html(title)
                ));
            }
            html.push_str("</li>");
        }
    }
    html.push_str("</ul>");

    Ok(())
}


