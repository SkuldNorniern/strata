use log::{debug, info};

/// Component for handling Floating Action Bar (FAB) functionality
pub struct FabComponent;

impl FabComponent {
    /// Create a new FAB component
    pub fn new() -> Self {
        debug!("Creating new FabComponent");
        Self
    }

    /// Generate FAB actions for a given path
    pub fn generate_actions(&self, path: &str) -> Vec<FabAction> {
        debug!("Generating FAB actions for path: '{}'", path);
        
        let mut actions = Vec::new();
        
        if !path.is_empty() {
            // Add raw view action
            let raw_href = format!("/raw/{}", path);
            actions.push(FabAction {
                href: raw_href,
                title: "View raw markdown".to_string(),
                class: "fab-action-raw".to_string(),
            });
            
            // Add edit action (placeholder for future implementation)
            let edit_href = format!("/edit/{}", path);
            actions.push(FabAction {
                href: edit_href,
                title: "Edit this page".to_string(),
                class: "fab-action-edit".to_string(),
            });
        }
        
        debug!("Generated {} FAB actions for path: '{}'", actions.len(), path);
        actions
    }

    /// Generate complete FAB HTML
    pub fn generate_fab_html(&self, path: &str, actions: &[FabAction]) -> String {
        debug!("Generating FAB HTML for path: '{}' with {} actions", path, actions.len());
        let start_time = std::time::Instant::now();
        
        // Use fab-page for search pages and other non-home pages to ensure home button is visible
        // Only use fab-home for the actual home page (which is handled separately in handlers)
        let fab_class = "fab-page";
        
        let mut html = format!("<div class=\"fab glass {}\" id=\"fab\">", fab_class);
        html.push_str("<div class=\"fab-menu\">");
        
        // Home button
        html.push_str("<a href=\"/\" class=\"fab-item\" title=\"Home\"></a>");
        
        // Search bar
        html.push_str("<div class=\"fab-search\">");
        html.push_str("<form action=\"/search\" method=\"get\" style=\"display:flex;width:100%\">");
        html.push_str("<input type=\"text\" name=\"q\" placeholder=\"Search...\" style=\"flex:1;border:none;background:none;outline:none;color:inherit;font:inherit\">");
        html.push_str("</form>");
        html.push_str("</div>");
        
        // Action buttons
        if !actions.is_empty() {
            html.push_str("<div class=\"fab-actions\">");
            for action in actions {
                html.push_str(&format!(
                    "<a href=\"{}\" title=\"{}\" class=\"{}\"></a>",
                    action.href, action.title, action.class
                ));
            }
            html.push_str("</div>");
        }
        
        html.push_str("</div>");
        html.push_str("</div>");
        
        let duration = start_time.elapsed();
        info!("FAB HTML generated in {:?}ms for path: '{}'", duration.as_millis(), path);
        
        html
    }

    /// Generate home page FAB HTML (with fab-home class)
    pub fn generate_home_fab_html(&self, actions: &[FabAction]) -> String {
        debug!("Generating home page FAB HTML with {} actions", actions.len());
        let start_time = std::time::Instant::now();
        
        let mut html = format!("<div class=\"fab glass fab-home\" id=\"fab\">");
        html.push_str("<div class=\"fab-menu\">");
        
        // Home button (hidden on home page)
        html.push_str("<a href=\"/\" class=\"fab-item\" title=\"Home\"></a>");
        
        // Search bar
        html.push_str("<div class=\"fab-search\">");
        html.push_str("<form action=\"/search\" method=\"get\" style=\"display:flex;width:100%\">");
        html.push_str("<input type=\"text\" name=\"q\" placeholder=\"Search...\" style=\"flex:1;border:none;background:none;outline:none;color:inherit;font:inherit\">");
        html.push_str("</form>");
        html.push_str("</div>");
        
        // Action buttons (hidden on home page)
        if !actions.is_empty() {
            html.push_str("<div class=\"fab-actions\">");
            for action in actions {
                html.push_str(&format!(
                    "<a href=\"{}\" title=\"{}\" class=\"{}\"></a>",
                    action.href, action.title, action.class
                ));
            }
            html.push_str("</div>");
        }
        
        html.push_str("</div>");
        html.push_str("</div>");
        
        let duration = start_time.elapsed();
        info!("Home page FAB HTML generated in {:?}ms", duration.as_millis());
        
        html
    }
}

/// Represents a FAB action button
pub struct FabAction {
    pub href: String,
    pub title: String,
    pub class: String,
}

impl Default for FabComponent {
    fn default() -> Self {
        Self::new()
    }
}
