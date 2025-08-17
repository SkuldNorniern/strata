use std::fs;
use std::path::Path;
use crate::types::WikiError;
use crate::fs_utils::escape_attr;
use crate::nav::build_sidebar_html;

/// Load and render the main HTML shell template
pub fn render_shell_template(
    title: &str,
    style: &str,
    sidebar: &str,
    content: &str,
    bottom_actions: &str,
) -> Result<String, WikiError> {
    // Try to load base template
    let base_path = Path::new("static/html/base.html");
    let fab_path = Path::new("static/html/fab.html");
    
    if let (Ok(base_tpl), Ok(fab_tpl)) = (fs::read_to_string(base_path), fs::read_to_string(fab_path)) {
        let mut html = base_tpl;
        let mut fab_html = fab_tpl;
        
        // Determine FAB class based on title/content
        let fab_class = if title == "Wiki" || title == "/" { "fab-home" } else { "fab-page" };
        fab_html = fab_html.replace("fab-page", fab_class);
        
        // Replace FAB placeholders
        fab_html = fab_html.replace("{{BOTTOM_ACTIONS}}", bottom_actions);
        
        // Replace base template placeholders
        html = html.replace("{{TITLE}}", title);
        html = html.replace("{{STYLE}}", style);
        html = html.replace("{{SIDEBAR}}", sidebar);
        html = html.replace("{{CONTENT}}", content);
        html = html.replace("{{FAB}}", &fab_html);
        
        return Ok(html);
    }
    
    // Fallback inline shell (also includes FAB)
    Ok(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{}</title>{}</head><body><a id=\"top\"></a><div class=\"layout\"><aside class=\"sidebar glass\">{}</aside><main class=\"content\"><div class=\"article-card glass\">{}</div></main></div><a class=\"back-to-top glass\" href=\"#top\" aria-label=\"Back to top\">↑</a>{}</body></html>",
        title, style, sidebar, content, bottom_actions
    ))
}

/// Generate a complete page with navigation and content
pub fn render_page_with_nav(
    base_dir: &Path,
    path: &str,
    title_override: Option<&str>,
    body: &str,
    bottom_actions: &str,
) -> Result<String, WikiError> {
    let title = title_override.unwrap_or_else(|| if path.is_empty() { "Wiki" } else { path });
    let style = external_style_link();
    let sidebar = build_sidebar_html(base_dir, path).unwrap_or_else(|_| String::new());
    
    render_shell_template(&escape_attr(title), &style, &sidebar, body, bottom_actions)
}

/// Generate a complete page with navigation, TOC in sidebar, and content
pub fn render_page_with_nav_and_toc(
    base_dir: &Path,
    path: &str,
    title_override: Option<&str>,
    body: &str,
    bottom_actions: &str,
    toc_html: &str,
) -> Result<String, WikiError> {
    let title = title_override.unwrap_or_else(|| if path.is_empty() { "Wiki" } else { path });
    let style = external_style_link();
    let sidebar = crate::nav::build_sidebar_with_toc(base_dir, path, toc_html).unwrap_or_else(|_| String::new());
    
    render_shell_template(&escape_attr(title), &style, &sidebar, body, bottom_actions)
}

/// Generate external CSS link
fn external_style_link() -> String {
    "<link rel=\"stylesheet\" href=\"/static/css/strata.css\">".to_string()
}
