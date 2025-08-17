use axum::{
    extract::{Path as AxumPath, RawQuery, State},
    http::{header, Response},
    response::{Html, IntoResponse},
    body::Body,
};
use std::path::Path;

use crate::errors::WikiError;
use crate::types::AppState;
use crate::utils::{escape_attr, escape_html, last_modified_html, normalize_path, parse_query_param};
use crate::services::{FileService, SearchService, MarkdownService};
use crate::components::{FabComponent, NavigationComponent, TemplateComponent};

/// Handle root path requests
pub async fn handle_root(State(state): State<AppState>) -> Result<impl IntoResponse, WikiError> {
    let file_service = FileService::new(state.base_dir.as_ref().clone());
    let navigation = NavigationComponent::new(file_service.clone());
    let fab = FabComponent::new();
    let templates = TemplateComponent::new();
    
    // Check for index.md or README.md
    let index_md = state.base_dir.join("index.md");
    let readme_md = state.base_dir.join("README.md");
    
    if index_md.is_file() {
        let content = file_service.read_file(Path::new("index.md"))?;
        let markdown_service = MarkdownService::new();
        let result = markdown_service.render_with_toc(&content)?;
        let meta = last_modified_html(&index_md);
        let body = format!("{}{}", meta, result.html);
        let actions = fab.generate_actions("");
        let fab_html = fab.generate_fab_html("", &actions);
        let sidebar = navigation.build_sidebar_html("")?;
        let page = templates.render_page_with_nav_and_toc(&sidebar, &body, &fab_html, result.title.as_deref().unwrap_or("Wiki"), &result.toc)?;
        return Ok(Html(page).into_response());
    }
    
    if readme_md.is_file() {
        let content = file_service.read_file(Path::new("README.md"))?;
        let markdown_service = MarkdownService::new();
        let result = markdown_service.render_with_toc(&content)?;
        let meta = last_modified_html(&readme_md);
        let body = format!("{}{}", meta, result.html);
        let actions = fab.generate_actions("");
        let fab_html = fab.generate_fab_html("", &actions);
        let sidebar = navigation.build_sidebar_html("")?;
        let page = templates.render_page_with_nav_and_toc(&sidebar, &body, &fab_html, result.title.as_deref().unwrap_or("Wiki"), &result.toc)?;
        return Ok(Html(page).into_response());
    }
    
    // Show directory listing
    let html = render_directory_listing(&file_service, "")?;
    let sidebar = navigation.build_sidebar_html("")?;
    let actions = fab.generate_actions("");
    let fab_html = fab.generate_fab_html("", &actions);
    let page = templates.render_page_with_nav(&sidebar, &html, &fab_html, "Wiki")?;
    Ok(Html(page).into_response())
}

/// Handle path-based requests
pub async fn handle_path(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> Result<impl IntoResponse, WikiError> {
    let normalized = normalize_path(&path);
    let requested = state.base_dir.join(&normalized);
    
    let file_service = FileService::new(state.base_dir.as_ref().clone());
    let navigation = NavigationComponent::new(file_service.clone());
    let fab = FabComponent::new();
    let templates = TemplateComponent::new();
    
    // First check if the exact path exists
    if requested.exists() {
        if requested.is_dir() {
            // Check for index.md or README.md in directory
            let index_md = requested.join("index.md");
            let readme_md = requested.join("README.md");
            
            if index_md.is_file() {
                // Convert full path to relative path for FileService
                let content = file_service.read_file(Path::new(&format!("{}/index.md", normalized)))?;
                let markdown_service = MarkdownService::new();
                let result = markdown_service.render_with_toc(&content)?;
                let meta = last_modified_html(&index_md);
                let body = format!("{}{}", meta, result.html);
                let actions = fab.generate_actions(&normalized);
                let fab_html = fab.generate_fab_html(&normalized, &actions);
                let sidebar = navigation.build_sidebar_html(&normalized)?;
                let title = result.title.as_deref().unwrap_or(&normalized);
                let page = templates.render_page_with_nav_and_toc(&sidebar, &body, &fab_html, title, &result.toc)?;
                return Ok(Html(page).into_response());
            }
            
            if readme_md.is_file() {
                // Convert full path to relative path for FileService
                let content = file_service.read_file(Path::new(&format!("{}/README.md", normalized)))?;
                let markdown_service = MarkdownService::new();
                let result = markdown_service.render_with_toc(&content)?;
                let meta = last_modified_html(&readme_md);
                let body = format!("{}{}", meta, result.html);
                let actions = fab.generate_actions(&normalized);
                let fab_html = fab.generate_fab_html(&normalized, &actions);
                let sidebar = navigation.build_sidebar_html(&normalized)?;
                let title = result.title.as_deref().unwrap_or(&normalized);
                let page = templates.render_page_with_nav_and_toc(&sidebar, &body, &fab_html, title, &result.toc)?;
                return Ok(Html(page).into_response());
            }
            
            // Show directory listing
            let html = render_directory_listing(&file_service, &normalized)?;
            let sidebar = navigation.build_sidebar_html(&normalized)?;
            let actions = fab.generate_actions(&normalized);
            let fab_html = fab.generate_fab_html(&normalized, &actions);
            let page = templates.render_page_with_nav(&sidebar, &html, &fab_html, &normalized)?;
            return Ok(Html(page).into_response());
        }
        
        if requested.is_file() {
            return serve_path(&state, &normalized, &requested).await;
        }
    }
    
    // If the exact path doesn't exist, check for .md variant
    let md_variant = requested.with_extension("md");
    
    if md_variant.is_file() {
        // Convert full path to relative path for FileService
        let relative_path = md_variant.strip_prefix(&*state.base_dir)
            .map_err(|_| WikiError::InvalidPath)?;
        let content = file_service.read_file(relative_path)?;
        let markdown_service = MarkdownService::new();
        let result = markdown_service.render_with_toc(&content)?;
        let meta = last_modified_html(&md_variant);
        let body = format!("{}{}", meta, result.html);
        let actions = fab.generate_actions(&normalized);
        let fab_html = fab.generate_fab_html(&normalized, &actions);
        let sidebar = navigation.build_sidebar_with_toc(&normalized, &result.toc)?;
        let title = result.title.as_deref().unwrap_or(&normalized);
        let page = templates.render_page_with_nav_and_toc(&sidebar, &body, &fab_html, title, &result.toc)?;
        return Ok(Html(page).into_response());
    }
    
    Err(WikiError::NotFound)
}

async fn serve_path(state: &AppState, req_path: &str, path: &Path) -> Result<Response<Body>, WikiError> {
    let file_service = FileService::new(state.base_dir.as_ref().clone());
    
    if is_markdown(path) {
        // Convert full path to relative path for FileService
        let relative_path = path.strip_prefix(&*state.base_dir)
            .map_err(|_| WikiError::InvalidPath)?;
        let content = file_service.read_file(relative_path)?;
        let markdown_service = MarkdownService::new();
        let result = markdown_service.render_with_toc(&content)?;
        let meta = last_modified_html(path);
        let body = format!("{}{}", meta, result.html);
        let fab = FabComponent::new();
        let actions = fab.generate_actions(req_path);
        let fab_html = fab.generate_fab_html(req_path, &actions);
        let navigation = NavigationComponent::new(file_service);
        let sidebar = navigation.build_sidebar_with_toc(req_path, &result.toc)?;
        let templates = TemplateComponent::new();
        let page = templates.render_page_with_nav_and_toc(&sidebar, &body, &fab_html, result.title.as_deref().unwrap_or(req_path), &result.toc)?;
        return Ok(Html(page).into_response());
    }
    
    let bytes = std::fs::read(path)?;
    let content_type = file_service.content_type_for(path);
    let mut resp = Response::new(Body::from(bytes));
    resp.headers_mut().insert(header::CONTENT_TYPE, content_type.parse().unwrap_or_else(|_| header::HeaderValue::from_static("application/octet-stream")));
    Ok(resp)
}

/// Render directory listing HTML
fn render_directory_listing(file_service: &FileService, req_path: &str) -> Result<String, WikiError> {
    let entries = file_service.list_directory(Path::new(req_path))?;
    let mut html = String::new();
    
    // Heading and parent link
    let title = if req_path.is_empty() { "/".to_string() } else { format!("/{}", req_path) };
    html.push_str(&format!("<h1>{}</h1>", escape_html(&title)));
    
    if !req_path.is_empty() {
        if let Some((parent, _)) = req_path.rsplit_once('/') {
            let back = if parent.is_empty() { "/".to_string() } else { format!("/{}", parent) };
            html.push_str(&format!("<p><a href=\"{}\">⬑ Up</a></p>", escape_attr(&back)));
        } else {
            html.push_str("<p><a href=\"/\">⬑ Up</a></p>");
        }
    }
    
    html.push_str("<ul class=\"listing\">\n");
    for entry in entries {
        let href = if req_path.is_empty() {
            if entry.is_dir {
                format!("/{}", entry.name)
            } else {
                // For markdown files, remove .md extension in the URL
                let name_without_ext = entry.name.trim_end_matches(".md");
                format!("/{}", name_without_ext)
            }
        } else {
            if entry.is_dir {
                format!("/{}/{}", req_path, entry.name)
            } else {
                // For markdown files, remove .md extension in the URL
                let name_without_ext = entry.name.trim_end_matches(".md");
                format!("/{}/{}", req_path, name_without_ext)
            }
        };
        let display = if entry.is_dir { 
            format!("{}/", entry.name) 
        } else { 
            entry.name 
        };
        html.push_str(&format!("  <li><a href=\"{}\">{}</a></li>\n", escape_attr(&href), escape_html(&display)));
    }
    html.push_str("</ul>\n");
    Ok(html)
}

/// Check if a file is markdown
fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

/// Handle search requests
pub async fn handle_search(
    State(state): State<AppState>,
    RawQuery(raw): RawQuery,
) -> Result<impl IntoResponse, WikiError> {
    let query = parse_query_param(&raw.unwrap_or_default(), "q");
    let file_service = FileService::new(state.base_dir.as_ref().clone());
    let search_service = SearchService::new(file_service.clone());
    let results = search_service.search(&query)?;
    
    let search_content = render_search_results(&query, &results);
    
    // Load search template
    let search_path = Path::new("static/html/search.html");
    if let Ok(tpl) = std::fs::read_to_string(search_path) {
        let mut html = tpl;
        html = html.replace("{{QUERY}}", &escape_attr(&query));
        html = html.replace("{{SEARCH_CONTENT}}", &search_content);
        let navigation = NavigationComponent::new(file_service);
        html = html.replace("{{SIDEBAR}}", &navigation.build_sidebar_html("")?);
        // FAB is now included in the search template
        html = html.replace("{{FAB}}", "");
        
        return Ok(Html(html).into_response());
    }
    
    // Fallback to inline HTML if template fails
    let navigation = NavigationComponent::new(file_service);
    let sidebar = navigation.build_sidebar_html("")?;
    let fab = FabComponent::new();
    let actions = fab.generate_actions("");
    let fab_html = fab.generate_fab_html("", &actions);
    let templates = TemplateComponent::new();
    let page = templates.render_page_with_nav(&sidebar, &search_content, &fab_html, "Search")?;
    Ok(Html(page).into_response())
}

/// Handle raw markdown requests
pub async fn handle_raw(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> Result<impl IntoResponse, WikiError> {
    let normalized = normalize_path(&path);
    let requested = state.base_dir.join(&normalized);
    
    if !requested.exists() {
        // Check for .md variant
        let md_variant = requested.with_extension("md");
        if md_variant.is_file() {
            let file_service = FileService::new(state.base_dir.as_ref().clone());
            let relative_path = md_variant.strip_prefix(&*state.base_dir)
                .map_err(|_| WikiError::InvalidPath)?;
            let content = file_service.read_file(relative_path)?;
            return Ok(content.into_response());
        }
        
        // Return HTML 404 page for consistency
        let error_html = r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>404 - Not Found</title>
    <link rel="stylesheet" href="/static/css/strata.css">
</head>
<body>
    <div class="error-page">
        <div class="error-container glass">
            <div class="error-icon">404</div>
            <h1 class="error-title">Page Not Found</h1>
            <p class="error-message">The requested page could not be found.</p>
            <div class="error-actions">
                <a href="/" class="error-btn primary">Go Home</a>
                <a href="javascript:history.back()" class="error-btn secondary">Go Back</a>
            </div>
        </div>
    </div>
</body>
</html>"#;
        return Ok(Html(error_html.to_string()).into_response());
    }
    
    let file_service = FileService::new(state.base_dir.as_ref().clone());
    let content = file_service.read_file(&requested)?;
    
    // Return plain text markdown
    Ok(content.into_response())
}

/// Handle static file requests
pub async fn handle_static(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> Result<impl IntoResponse, WikiError> {
    let normalized = normalize_path(&path);
    let requested = state.static_dir.join(&normalized);
    
    if !requested.exists() {
        return Err(WikiError::NotFound);
    }
    
    let bytes = std::fs::read(&requested)?;
    let file_service = FileService::new(state.static_dir.as_ref().clone());
    let content_type = file_service.content_type_for(&requested);
    let mut resp = Response::new(Body::from(bytes));
    resp.headers_mut().insert(header::CONTENT_TYPE, content_type.parse().unwrap_or_else(|_| header::HeaderValue::from_static("application/octet-stream")));
    Ok(resp)
}

/// Render search results HTML
fn render_search_results(query: &str, results: &[crate::types::SearchResult]) -> String {
    let mut content = String::new();
    
    if query.is_empty() {
        content.push_str("<div class=\"search-results\">");
        content.push_str("<p class=\"no-query\">Enter a search query to find content.</p>");
        content.push_str("</div>");
        return content;
    }
    
    content.push_str("<div class=\"search-results\">");
    content.push_str(&format!("<p class=\"results-count\">Found {} results</p>", results.len()));
    
    if results.is_empty() {
        content.push_str("<p class=\"no-results\">No results found for your search.</p>");
    } else {
        content.push_str("<ul class=\"listing\">");
        for result in results {
            let href = format!("/{}", result.path);
            content.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>",
                escape_attr(&href), escape_html(&result.title)
            ));
        }
        content.push_str("</ul>");
    }
    
    content.push_str("</div>");
    content
}


