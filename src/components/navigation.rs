use std::path::Path;
use crate::errors::WikiError;
use crate::services::FileService;

/// Component for handling navigation and sidebar functionality
pub struct NavigationComponent {
    file_service: FileService,
}

impl NavigationComponent {
    /// Create a new navigation component
    pub fn new(file_service: FileService) -> Self {
        Self { file_service }
    }

    /// Build sidebar HTML with navigation
    pub fn build_sidebar_html(&self, current_path: &str) -> Result<String, WikiError> {
        // List the root wiki directory for navigation
        let entries = self.file_service.list_directory(Path::new(""))?;
        let mut html = String::new();
        
        html.push_str("<div class=\"sidebar-nav\">");
        html.push_str("<div class=\"sidebar-title\">Navigation</div>");
        html.push_str("<ul class=\"nav-list\">");
        
        for entry in entries {
            let href = if entry.name == "index.md" || entry.name == "README.md" {
                "/".to_string()
            } else if entry.is_dir {
                format!("/{}", entry.name)
            } else {
                // For markdown files, remove .md extension in the URL
                let name_without_ext = entry.name.trim_end_matches(".md");
                format!("/{}", name_without_ext)
            };
            
            let display = if entry.is_dir { 
                format!("{}/", entry.name) 
            } else { 
                entry.name.trim_end_matches(".md").to_string() 
            };
            
            let active_class = if current_path == href { " active" } else { "" };
            
            // Check if this item has sub-items
            let has_sub_items = if entry.is_dir {
                if let Ok(sub_entries) = self.file_service.list_directory(Path::new(&entry.name)) {
                    !sub_entries.is_empty()
                } else {
                    false
                }
            } else {
                false
            };
            
            if has_sub_items {
                html.push_str(&format!(
                    "<li class=\"nav-item has-sub\"><a href=\"{}\" class=\"{}\"><span class=\"nav-toggle\"></span>{}</a>",
                    href, active_class, display
                ));
                
                // Add sub-items
                if let Ok(sub_entries) = self.file_service.list_directory(Path::new(&entry.name)) {
                    html.push_str("<ul class=\"nav-sub-list\">");
                    for sub_entry in sub_entries {
                        let sub_href = if sub_entry.name == "index.md" || sub_entry.name == "README.md" {
                            format!("/{}", entry.name)
                        } else if sub_entry.is_dir {
                            format!("/{}/{}", entry.name, sub_entry.name)
                        } else {
                            let name_without_ext = sub_entry.name.trim_end_matches(".md");
                            format!("/{}/{}", entry.name, name_without_ext)
                        };
                        
                        let sub_display = if sub_entry.is_dir { 
                            format!("{}/", sub_entry.name) 
                        } else { 
                            sub_entry.name.trim_end_matches(".md").to_string() 
                        };
                        
                        let sub_active_class = if current_path == sub_href { " active" } else { "" };
                        
                        html.push_str(&format!(
                            "<li class=\"nav-sub-item\"><a href=\"{}\" class=\"{}\">{}</a></li>",
                            sub_href, sub_active_class, sub_display
                        ));
                    }
                    html.push_str("</ul>");
                }
                
                html.push_str("</li>");
            } else {
                html.push_str(&format!(
                    "<li class=\"nav-item\"><a href=\"{}\" class=\"{}\">{}</a></li>",
                    href, active_class, display
                ));
            }
        }
        
        html.push_str("</ul></div>");
        
        // Add JavaScript for toggle functionality
        html.push_str("<script>
            document.addEventListener('DOMContentLoaded', function() {
                const navItems = document.querySelectorAll('.nav-item.has-sub');
                navItems.forEach(item => {
                    const link = item.querySelector('a');
                    const subList = item.querySelector('.nav-sub-list');
                    const toggle = item.querySelector('.nav-toggle');
                    
                    if (link && subList && toggle) {
                        link.addEventListener('click', function(e) {
                            e.preventDefault();
                            item.classList.toggle('expanded');
                            subList.classList.toggle('expanded');
                            toggle.classList.toggle('expanded');
                        });
                    }
                });
            });
        </script>");
        
        Ok(html)
    }

    /// Build sidebar with table of contents
    pub fn build_sidebar_with_toc(
        &self, 
        current_path: &str, 
        toc_html: &str
    ) -> Result<String, WikiError> {
        let mut sidebar = self.build_sidebar_html(current_path)?;
        
        // Add TOC section
        sidebar.push_str("<div class=\"sidebar-toc\">");
        sidebar.push_str("<div class=\"sidebar-toc-title\">On This Page</div>");
        sidebar.push_str(toc_html);
        sidebar.push_str("</div>");
        
        Ok(sidebar)
    }
}
