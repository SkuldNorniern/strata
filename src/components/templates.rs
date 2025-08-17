use std::fs;
use std::path::Path;
use crate::errors::WikiError;
use crate::types::TemplateContext;
use crate::utils::escape_attr;

/// Component for handling HTML template rendering
pub struct TemplateComponent;

impl TemplateComponent {
    /// Create a new template component
    pub fn new() -> Self {
        Self
    }

    /// Load and render the main HTML shell template
    pub fn render_shell_template(&self, context: &TemplateContext) -> Result<String, WikiError> {
        // Try to load base template with multiple possible paths
        let possible_paths = [
            "static/html/base.html",
            "./static/html/base.html",
            "../static/html/base.html",
        ];
        
        let mut base_tpl = None;
        
        for path_str in &possible_paths {
            let base_path = Path::new(path_str);
            
            if let Ok(base) = fs::read_to_string(base_path) {
                base_tpl = Some(base);
                break;
            }
        }
        
        if let Some(base) = base_tpl {
            let mut html = base;
            
            // Replace base template placeholders
            html = html.replace("{{TITLE}}", &escape_attr(&context.title));
            html = html.replace("{{STYLE}}", "<link rel=\"stylesheet\" href=\"/static/css/strata.css\">");
            html = html.replace("{{SIDEBAR}}", &context.sidebar);
            html = html.replace("{{CONTENT}}", &context.content);
            html = html.replace("{{FAB}}", &context.fab);
            
            return Ok(html);
        }
        
        // Fallback inline shell
        Ok(format!(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{}</title><link rel=\"stylesheet\" href=\"/static/css/strata.css\"></head><body><a id=\"top\"></a><div class=\"layout\"><aside class=\"sidebar glass\">{}</aside><main class=\"content\"><div class=\"article-card glass\">{}</div></main></div><a class=\"back-to-top glass\" href=\"#top\" aria-label=\"Back to top\">↑</a>{}</body></html>",
            context.title, context.sidebar, context.content, context.fab
        ))
    }

    /// Generate a complete page with navigation and content
    pub fn render_page_with_nav(
        &self,
        navigation: &str,
        content: &str,
        fab: &str,
        title: &str,
    ) -> Result<String, WikiError> {
        let context = TemplateContext {
            title: title.to_string(),
            content: content.to_string(),
            sidebar: navigation.to_string(),
            fab: fab.to_string(),
            toc: None,
        };
        
        self.render_shell_template(&context)
    }

    /// Generate a complete page with navigation, TOC, and content
    pub fn render_page_with_nav_and_toc(
        &self,
        navigation: &str,
        content: &str,
        fab: &str,
        title: &str,
        toc: &str,
    ) -> Result<String, WikiError> {
        let context = TemplateContext {
            title: title.to_string(),
            content: content.to_string(),
            sidebar: navigation.to_string(),
            fab: fab.to_string(),
            toc: Some(toc.to_string()),
        };
        
        self.render_shell_template(&context)
    }
}

impl Default for TemplateComponent {
    fn default() -> Self {
        Self::new()
    }
}
