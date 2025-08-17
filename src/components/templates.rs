use log::{debug, info};
use crate::errors::WikiError;

/// Component for handling HTML template rendering
pub struct TemplateComponent;

impl TemplateComponent {
    /// Create a new template component
    pub fn new() -> Self {
        debug!("Creating new TemplateComponent");
        Self
    }

    /// Render a page with navigation
    pub fn render_page_with_nav(
        &self,
        sidebar: &str,
        content: &str,
        fab: &str,
        title: &str,
    ) -> Result<String, WikiError> {
        debug!("Rendering page with navigation, title: '{}'", title);
        let start_time = std::time::Instant::now();
        
        let html = self.render_shell_template(sidebar, content, fab, title)?;
        
        let duration = start_time.elapsed();
        info!("Page with navigation rendered in {:?}ms, title: '{}'", duration.as_millis(), title);
        
        Ok(html)
    }

    /// Render a page with navigation and table of contents
    pub fn render_page_with_nav_and_toc(
        &self,
        sidebar: &str,
        content: &str,
        fab: &str,
        title: &str,
        _toc: &str,
    ) -> Result<String, WikiError> {
        debug!("Rendering page with navigation and TOC, title: '{}'", title);
        let start_time = std::time::Instant::now();
        
        let html = self.render_shell_template(sidebar, content, fab, title)?;
        
        let duration = start_time.elapsed();
        info!("Page with navigation and TOC rendered in {:?}ms, title: '{}'", duration.as_millis(), title);
        
        Ok(html)
    }

    /// Render the shell template with all components
    fn render_shell_template(
        &self,
        sidebar: &str,
        content: &str,
        fab: &str,
        title: &str,
    ) -> Result<String, WikiError> {
        debug!("Rendering shell template");
        
        let mut html = String::new();
        html.push_str("<!doctype html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"utf-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
        html.push_str(&format!("    <title>{} - Strata Wiki</title>\n", title));
        html.push_str("    <link rel=\"stylesheet\" href=\"/static/css/strata.css\">\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str("    <div class=\"layout\">\n");
        html.push_str("        <aside class=\"sidebar glass\">");
        html.push_str(sidebar);
        html.push_str("</aside>\n");
        html.push_str("        <main class=\"content\">\n");
        html.push_str("            <div class=\"article-card glass\">\n");
        html.push_str(content);
        html.push_str("            </div>\n");
        html.push_str("        </main>\n");
        html.push_str("    </div>\n");
        html.push_str("    <a class=\"back-to-top glass\" href=\"#top\" aria-label=\"Back to top\">↑</a>\n");
        html.push_str(fab);
        html.push_str("\n</body>\n");
        html.push_str("</html>");

        debug!("Shell template rendered successfully");
        Ok(html)
    }
}

impl Default for TemplateComponent {
    fn default() -> Self {
        Self::new()
    }
}
