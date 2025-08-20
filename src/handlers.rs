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

/// Handle path requests
pub async fn handle_path(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> Result<impl IntoResponse, WikiError> {
    log::info!("Path request received: '{}'", path);
    
    let normalized = normalize_path(&path);
    let requested = state.base_dir.join(&normalized);
    
    log::debug!("Normalized path: '{}', requested: {:?}", normalized, requested);
    
    let file_service = FileService::new(state.base_dir.as_ref().clone());
    let navigation = NavigationComponent::new(file_service.clone());
    let fab = FabComponent::new();
    let templates = TemplateComponent::new();
    
    // First check if the exact path exists
    if requested.exists() {
        if requested.is_dir() {
            log::debug!("Path is a directory, checking for index files");
            // Check for index.md or README.md in directory
            let index_md = requested.join("index.md");
            let readme_md = requested.join("README.md");
            
            if index_md.is_file() {
                log::debug!("Found index.md in directory");
                // Convert full path to relative path for FileService
                let content = file_service.read_file(Path::new(&format!("{}/index.md", normalized)))?;
                let markdown_service = MarkdownService::new();
                let result = markdown_service.render_with_toc(&content)?;
                let meta = last_modified_html(&index_md);
                let body = format!("{}{}", meta, result.html);
                let actions = fab.generate_actions(&normalized);
                let fab_html = fab.generate_fab_html(&normalized, &actions);
                let sidebar = navigation.build_sidebar_with_toc(&normalized, &result.toc)?;
                let title = result.title.as_deref().unwrap_or(&normalized);
                let page = templates.render_page_with_nav_and_toc(&sidebar, &body, &fab_html, title, &result.toc)?;
                log::info!("Serving index.md for directory: '{}'", normalized);
                return Ok(Html(page).into_response());
            }
            
            if readme_md.is_file() {
                log::debug!("Found README.md in directory");
                // Convert full path to relative path for FileService
                let content = file_service.read_file(Path::new(&format!("{}/README.md", normalized)))?;
                let markdown_service = MarkdownService::new();
                let result = markdown_service.render_with_toc(&content)?;
                let meta = last_modified_html(&readme_md);
                let body = format!("{}{}", meta, result.html);
                let actions = fab.generate_actions(&normalized);
                let fab_html = fab.generate_fab_html(&normalized, &actions);
                let sidebar = navigation.build_sidebar_with_toc(&normalized, &result.toc)?;
                let title = result.title.as_deref().unwrap_or(&normalized);
                let page = templates.render_page_with_nav_and_toc(&sidebar, &body, &fab_html, title, &result.toc)?;
                log::info!("Serving README.md for directory: '{}'", normalized);
                return Ok(Html(page).into_response());
            }
            
            // Directory listing
            log::debug!("No index files found, generating directory listing");
            let html = render_directory_listing(&file_service, &normalized)?;
            let sidebar = navigation.build_sidebar_html(&normalized)?;
            let actions = fab.generate_actions(&normalized);
            let fab_html = fab.generate_fab_html(&normalized, &actions);
            let page = templates.render_page_with_nav(&sidebar, &html, &fab_html, &normalized)?;
            log::info!("Serving directory listing for: '{}'", normalized);
            return Ok(Html(page).into_response());
        }
        
        if requested.is_file() {
            log::debug!("Path is a file, serving via static handler");
            return serve_path(&state, &normalized, &requested).await;
        }
    }
    
    // If the exact path doesn't exist, check for .md variant
    let md_variant = requested.with_extension("md");
    if md_variant.is_file() {
        log::debug!("Found .md variant: {:?}", md_variant);
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
        log::info!("Serving .md file: '{}'", normalized);
        return Ok(Html(page).into_response());
    }
    
    log::warn!("Path not found: '{}'", normalized);
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
    let raw_query = raw.unwrap_or_default();
    let query = parse_query_param(&raw_query, "q");
    
    log::info!("Search request received for query: '{}'", query);
    log::debug!("Raw query string: '{:?}'", raw_query);
    
    // Check for potentially problematic queries
    let query = if query.len() > 1000 {
        log::warn!("Very long search query received ({} chars), truncating", query.len());
        // Truncate very long queries to prevent issues
        &query[..1000]
    } else {
        &query
    };
    
    let start_time = std::time::Instant::now();
    
    let file_service = FileService::new(state.base_dir.as_ref().clone());
    let search_service = SearchService::new(file_service.clone());
    
    log::debug!("Search service created, starting search...");
    
    let results = match search_service.search(&query) {
        Ok(results) => {
            log::info!("Search completed successfully, found {} results", results.len());
            results
        }
        Err(e) => {
            log::error!("Search failed: {:?}", e);
            return Err(e);
        }
    };
    
    let search_content = render_search_results(&query, &results);
    
    log::debug!("Search results rendered, creating response...");
    
    // Use template component for consistent rendering
    let navigation = NavigationComponent::new(file_service);
    let sidebar = navigation.build_sidebar_html("")?;
    let fab = FabComponent::new();
    let actions = fab.generate_actions("");
    let fab_html = fab.generate_fab_html("", &actions);
    let templates = TemplateComponent::new();
    
    let page = templates.render_page_with_nav(&sidebar, &search_content, &fab_html, "Search")?;
    
    let duration = start_time.elapsed();
    log::info!("Search request completed in {:?}ms", duration.as_millis());
    
    Ok(Html(page).into_response())
}

/// Handle raw markdown requests
pub async fn handle_raw(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> Result<impl IntoResponse, WikiError> {
    let normalized = normalize_path(&path);
    let requested = state.base_dir.join(&normalized);
    
    let file_service = FileService::new(state.base_dir.as_ref().clone());
    let content: String;
    let display_path: String;
    
    if !requested.exists() {
        // Check for .md variant
        let md_variant = requested.with_extension("md");
        if md_variant.is_file() {
            let relative_path = md_variant.strip_prefix(&*state.base_dir)
                .map_err(|_| WikiError::InvalidPath)?;
            content = file_service.read_file(relative_path)?;
            display_path = relative_path.to_string_lossy().to_string();
        } else {
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
                <a href="/" class="error-btn secondary">Go Home</a>
            </div>
        </div>
    </div>
</body>
</html>"#;
            return Ok(Html(error_html.to_string()).into_response());
        }
    } else {
        let relative_path = requested.strip_prefix(&*state.base_dir)
            .map_err(|_| WikiError::InvalidPath)?;
        content = file_service.read_file(relative_path)?;
        display_path = relative_path.to_string_lossy().to_string();
    }
    
    // Create the rendered path (remove .md extension for display)
    let rendered_path = if display_path.ends_with(".md") {
        display_path[..display_path.len()-3].to_string()
    } else {
        display_path.clone()
    };
    
    // Return a proper HTML page with the raw markdown content
    let raw_html = format!(r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Raw: {}</title>
    <link rel="stylesheet" href="/static/css/strata.css">
</head>
<body>
    <div class="raw-viewer">
        <div class="raw-header glass">
            <div class="raw-title">
                <h1>Raw Markdown: {}</h1>
                <p class="raw-path">/raw/{}</p>
            </div>
            <div class="raw-actions">
                <a href="/{}" class="raw-btn primary">← Back to Rendered View</a>
                <a href="/" class="raw-btn secondary">Go Home</a>
            </div>
        </div>
        <div class="raw-content glass">
            <pre class="raw-markdown"><code>{}</code></pre>
        </div>
    </div>
</body>
</html>"#, 
        display_path, 
        display_path, 
        display_path, 
        rendered_path,
        escape_html(&content)
    );
    
    Ok(Html(raw_html).into_response())
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
    content.push_str(&format!("<h2 class=\"search-header\">Search Results for \"{}\"</h2>", escape_html(query)));
    content.push_str(&format!("<p class=\"results-count\">Found {} result{}</p>", results.len(), if results.len() == 1 { "" } else { "s" }));
    
    if results.is_empty() {
        content.push_str("<p class=\"no-results\">No results found for your search.</p>");
        content.push_str("<div class=\"search-tips\">");
        content.push_str("<h3>Search Tips:</h3>");
        content.push_str("<ul>");
        content.push_str("<li>Try using different keywords</li>");
        content.push_str("<li>Check spelling and try synonyms</li>");
        content.push_str("<li>Use shorter, more general terms</li>");
        content.push_str("<li>Browse the <a href=\"/guide/\">guides</a> or <a href=\"/reference/\">reference</a> sections</li>");
        content.push_str("</ul>");
        content.push_str("</div>");
    } else {
        content.push_str("<div class=\"search-results-list\">");
        for result in results {
            let href = format!("/{}", result.path);
            let path_display = result.path.replace(".md", "");
            
            content.push_str("<div class=\"search-result-item glass\">");
            content.push_str(&format!(
                "<h3 class=\"result-title\"><a href=\"{}\">{}</a></h3>",
                escape_attr(&href), escape_html(&result.title)
            ));
            content.push_str(&format!(
                "<p class=\"result-path\"><code>{}</code></p>",
                escape_html(&path_display)
            ));
            content.push_str(&format!(
                "<p class=\"result-excerpt\">{}</p>",
                escape_html(&result.excerpt)
            ));
            content.push_str(&format!(
                "<div class=\"result-meta\">Relevance: {:.1}</div>",
                result.relevance
            ));
            content.push_str("</div>");
        }
        content.push_str("</div>");
    }
    
    content.push_str("</div>");
    content
}


