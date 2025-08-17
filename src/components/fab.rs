/// Component for handling Floating Action Bar (FAB) functionality
pub struct FabComponent;

impl FabComponent {
    /// Create a new FAB component
    pub fn new() -> Self {
        Self
    }

    /// Generate FAB actions for different page types
    pub fn generate_actions(&self, req_path: &str) -> String {
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
            crate::utils::escape_attr(&raw_href), 
            crate::utils::escape_attr(&edit_href)
        )
    }

    /// Generate FAB HTML with proper CSS classes
    pub fn generate_fab_html(&self, req_path: &str, actions: &str) -> String {
        let fab_class = if req_path.is_empty() { "fab-home" } else { "fab-page" };
        
        format!(r#"<div class="fab glass {}" id="fab">
            <div class="fab-menu">
                <a href="/" class="fab-item" title="Home"></a>
                <div class="fab-search">
                    <input type="text" placeholder="Search..." onkeypress="if(event.key==='Enter'){{window.location.href='/search?q='+this.value}}">
                </div>
                <div class="fab-actions">
                    {}
                </div>
            </div>
        </div>"#, fab_class, actions)
    }
}

impl Default for FabComponent {
    fn default() -> Self {
        Self::new()
    }
}
