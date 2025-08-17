use std::path::Path;

use axum::{body::Body, extract::{Path as AxumPath, RawQuery, State}, http::{header, StatusCode}, response::{Html, IntoResponse, Response}};

use crate::fs_utils::{ensure_safe_path, is_markdown, normalize_request_path};
use crate::render::render_markdown_with_toc;
use crate::templates::render_page_with_nav;
use crate::utils::{content_type_for, generate_fab_actions, last_modified_html};
use crate::types::{AppState, WikiError};

pub async fn handle_root(State(state): State<AppState>) -> Result<impl IntoResponse, WikiError> {
    render_path(&state, "").await
}

pub async fn handle_path(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> Result<impl IntoResponse, WikiError> {
    match render_path(&state, &path).await {
        Ok(response) => Ok(response),
        Err(WikiError::NotFound) => {
            // Return custom 404 page
            let not_found_path = Path::new("static/html/404.html");
            if let Ok(tpl) = std::fs::read_to_string(not_found_path) {
                return Ok((
                    StatusCode::NOT_FOUND,
                    Html(tpl)
                ).into_response());
            }
            
            // Fallback 404
            Ok((
                StatusCode::NOT_FOUND,
                Html(r#"
                <!doctype html>
                <html lang="en">
                <head><meta charset="utf-8"><title>404 - Not Found</title></head>
                <body style="font-family: system-ui; text-align: center; padding: 50px;">
                    <h1>404 - Page Not Found</h1>
                    <p>The requested page could not be found.</p>
                    <p><a href="/">Go to Home</a></p>
                </body>
                </html>
                "#)
            ).into_response())
        }
        Err(e) => Err(e),
    }
}
pub async fn handle_static(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> Result<impl IntoResponse, WikiError> {
    let norm = normalize_request_path(&path);
    let requested = state.static_dir.join(&norm);
    if requested.is_file() {
        let bytes = std::fs::read(&requested)?;
        let mut resp = Response::new(Body::from(bytes));
        resp.headers_mut().insert(header::CACHE_CONTROL, header::HeaderValue::from_static("public, max-age=3600"));
        let ct = content_type_for(&requested);
        resp.headers_mut().insert(header::CONTENT_TYPE, ct.parse().unwrap_or_else(|_| header::HeaderValue::from_static("application/octet-stream")));
        return Ok(resp);
    }
    Err(WikiError::NotFound)
}

pub async fn handle_raw(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> Result<impl IntoResponse, WikiError> {
    // Clean up the path - remove .md extension if it exists to avoid duplication
    let clean_path = if path.ends_with(".md") {
        &path[..path.len() - 3]
    } else {
        &path
    };
    
    let norm = normalize_request_path(clean_path);
    let requested = state.base_dir.join(&norm).with_extension("md");
    
    if requested.is_file() {
        let raw = std::fs::read_to_string(&requested)?;
        
        // Return raw markdown as plain text with no styling
        let mut resp = Response::new(Body::from(raw));
        resp.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/plain; charset=utf-8"));
        return Ok(resp);
    }
    
    // Return proper HTML 404 page for raw route
    let not_found_path = Path::new("static/html/404.html");
    if let Ok(tpl) = std::fs::read_to_string(not_found_path) {
        return Ok((
            StatusCode::NOT_FOUND,
            Html(tpl)
        ).into_response());
    }
    
    // Fallback 404
    Ok((
        StatusCode::NOT_FOUND,
        Html(r#"
        <!doctype html>
        <html lang="en">
        <head><meta charset="utf-8"><title>404 - Not Found</title></head>
        <body style="font-family: system-ui; text-align: center; padding: 50px;">
            <h1>404 - Page Not Found</h1>
            <p>The requested page could not be found.</p>
            <p><a href="/">Go to Home</a></p>
        </body>
        </html>
        "#)
    ).into_response())
}

pub async fn handle_search(State(state): State<AppState>, RawQuery(raw): RawQuery) -> Result<impl IntoResponse, WikiError> {
    let query = parse_query_param(raw, "q");
    let results = search_files(&state, &query)?;
    let search_content = render_search_results(&query, &results);
    
    // Load search template
    let search_path = Path::new("static/html/search.html");
    if let Ok(tpl) = std::fs::read_to_string(search_path) {
        let mut html = tpl;
        html = html.replace("{{QUERY}}", &crate::fs_utils::escape_attr(&query));
        html = html.replace("{{SEARCH_CONTENT}}", &search_content);
        html = html.replace("{{SIDEBAR}}", &crate::nav::build_sidebar_html(&state.base_dir, "").unwrap_or_else(|_| String::new()));
        // FAB is now included in the search template
        html = html.replace("{{FAB}}", "");
        
        return Ok(Html(html).into_response());
    }
    
    // Fallback to inline HTML if template fails
    let page = render_page_with_nav(&state.base_dir, "", None, &search_content, "")?;
    Ok(Html(page).into_response())
}

// 404 handling is now done directly in render_path function

async fn render_path(state: &AppState, req_path: &str) -> Result<Response, WikiError> {
    ensure_safe_path(req_path)?;

    let normalized = normalize_request_path(req_path);
    let requested = state.base_dir.join(&normalized);

    if requested.is_dir() {
        let index_md = requested.join("index.md");
        let readme_md = requested.join("README.md");
        if index_md.is_file() {
            let (html, toc, title) = render_markdown_with_toc(&index_md)?;
            let meta = last_modified_html(&index_md);
            let body = format!("{}{}", meta, html);
            let bottom_actions = generate_fab_actions(&normalized);
            let page = crate::templates::render_page_with_nav_and_toc(&state.base_dir, &normalized, title.as_deref(), &body, &bottom_actions, &toc)?;
            return Ok(Html(page).into_response());
        }
        if readme_md.is_file() {
            let (html, toc, title) = render_markdown_with_toc(&readme_md)?;
            let meta = last_modified_html(&readme_md);
            let body = format!("{}{}", meta, html);
            let bottom_actions = generate_fab_actions(&normalized);
            let page = crate::templates::render_page_with_nav_and_toc(&state.base_dir, &normalized, title.as_deref(), &body, &bottom_actions, &toc)?;
            return Ok(Html(page).into_response());
        }
        let html = render_directory_listing(&normalized, &requested)?;
        let page = render_page_with_nav(&state.base_dir, &normalized, None, &html, "")?;
        return Ok(Html(page).into_response());
    }

    if requested.is_file() {
        return serve_path(state, &normalized, &requested);
    }

    let md_variant = requested.with_extension("md");
    if md_variant.is_file() {
        let (html, toc, title) = render_markdown_with_toc(&md_variant)?;
        let meta = last_modified_html(&md_variant);
        let body = format!("{}{}", meta, html);
        let bottom_actions = generate_fab_actions(&normalized);
        let page = crate::templates::render_page_with_nav_and_toc(&state.base_dir, &normalized, title.as_deref(), &body, &bottom_actions, &toc)?;
        return Ok(Html(page).into_response());
    }

    Err(WikiError::NotFound)
}

fn serve_path(state: &AppState, req_path: &str, path: &Path) -> Result<Response, WikiError> {
    if is_markdown(path) {
        let (html, toc, title) = render_markdown_with_toc(path)?;
        let meta = last_modified_html(path);
        let body = format!("{}{}", meta, html);
        let bottom_actions = generate_fab_actions(req_path);
        let page = crate::templates::render_page_with_nav_and_toc(&state.base_dir, req_path, title.as_deref(), &body, &bottom_actions, &toc)?;
        return Ok(Html(page).into_response());
    }

    let bytes = std::fs::read(path)?;
    let content_type = content_type_for(path);
    let mut resp = Response::new(Body::from(bytes));
    // SAFETY: HeaderValue::from_static only fails for invalid header values; provided strings are static and valid
    resp.headers_mut().insert(header::CONTENT_TYPE, content_type.parse().unwrap_or_else(|_| header::HeaderValue::from_static("application/octet-stream")));
    Ok(resp)
}

/// Render directory listing HTML
fn render_directory_listing(req_path: &str, dir: &Path) -> Result<String, WikiError> {
    let entries = crate::fs_utils::list_dir_filtered(dir)?;
    let mut html = String::new();
    // Heading and parent link
    let title = if req_path.is_empty() { "/".to_string() } else { format!("/{}", req_path) };
    html.push_str(&format!("<h1>{}</h1>", crate::fs_utils::escape_html(&title)));
    if !req_path.is_empty() {
        if let Some((parent, _)) = req_path.rsplit_once('/') {
            let back = if parent.is_empty() { "/".to_string() } else { format!("/{}", parent) };
            html.push_str(&format!("<p><a href=\"{}\">⬑ Up</a></p>", crate::fs_utils::escape_attr(&back)));
        } else {
            html.push_str("<p><a href=\"/\">⬑ Up</a></p>");
        }
    }
    html.push_str("<ul class=\"listing\">\n");
    for entry in entries {
        let href = if req_path.is_empty() {
            format!("/{}", entry.name)
        } else {
            format!("/{}/{}", req_path, entry.name)
        };
        let display = if entry.is_dir { format!("{}/", entry.name) } else { entry.name };
        html.push_str(&format!("  <li><a href=\"{}\">{}</a></li>\n", crate::fs_utils::escape_attr(&href), crate::fs_utils::escape_html(&display)));
    }
    html.push_str("</ul>\n");
    Ok(html)
}

// This function is now handled by the templates module

// These functions are now in the utils module

fn search_files(state: &AppState, query: &str) -> Result<Vec<(String, String)>, WikiError> {
    if query.is_empty() { return Ok(Vec::new()); }
    let mut results = Vec::new();
    visit_dirs(&state.base_dir, &state.base_dir, &mut |rel, content| {
        if content.to_lowercase().contains(&query.to_lowercase()) {
            results.push((rel.trim_end_matches(".md").to_string(), rel));
        }
    })?;
    // De-duplicate and sort
    results.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    results.dedup_by(|a, b| a.1 == b.1);
    Ok(results)
}

fn visit_dirs<F>(root: &Path, dir: &Path, on_md: &mut F) -> Result<(), WikiError>
where F: FnMut(String, String) {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dirs(root, &path, on_md)?;
        } else if crate::fs_utils::is_markdown(&path) {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let rel_str = rel.to_string_lossy().to_string();
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            on_md(rel_str.clone(), content);
        }
    }
    Ok(())
}

fn render_search_results(query: &str, results: &[(String, String)]) -> String {
    let mut content = String::new();
    
    if query.is_empty() {
        content.push_str("<div class=\"no-query\">");
        content.push_str("✨ Enter a search query above to discover content in the wiki");
        content.push_str("</div>");
    } else if results.is_empty() {
        content.push_str("<div class=\"no-results\">");
        content.push_str("🔍 No results found for \"");
        content.push_str(&crate::fs_utils::escape_html(query));
        content.push_str("\"<br><small>Try different keywords or check your spelling</small>");
        content.push_str("</div>");
    } else {
        // Results count
        content.push_str("<div class=\"results-count\">");
        content.push_str(&format!("Found {} result{}", results.len(), if results.len() == 1 { "" } else { "s" }));
        content.push_str("</div>");
        
        // Results list
        content.push_str("<ul class=\"listing\">");
        for (title, rel) in results {
            let href = format!("/{}", rel.trim_end_matches(".md"));
            content.push_str(&format!("<li><a href=\"{}\">{}</a></li>", crate::fs_utils::escape_attr(&href), crate::fs_utils::escape_html(title)));
        }
        content.push_str("</ul>");
    }
    
    content
}

fn parse_query_param(raw: Option<String>, key: &str) -> String {
    let raw = match raw { Some(v) => v, None => return String::new() };
    for pair in raw.split('&') {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next().unwrap_or("");
        let v = parts.next().unwrap_or("");
        if k == key {
            return percent_decode_plus(v);
        }
    }
    String::new()
}

fn percent_decode_plus(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] as char {
            '+' => { out.push(' '); i += 1; }
            '%' if i + 2 < bytes.len() => {
                let h1 = bytes[i + 1] as char;
                let h2 = bytes[i + 2] as char;
                if let (Some(a), Some(b)) = (hex_val(h1), hex_val(h2)) {
                    out.push((a * 16 + b) as char);
                    i += 3;
                } else { out.push('%'); i += 1; }
            }
            c => { out.push(c); i += 1; }
        }
    }
    out
}

fn hex_val(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some((c as u8) - b'0'),
        'a'..='f' => Some(10 + (c as u8) - b'a'),
        'A'..='F' => Some(10 + (c as u8) - b'A'),
        _ => None,
    }
}


