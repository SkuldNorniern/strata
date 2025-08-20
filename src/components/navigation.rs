use std::path::Path;
use log::{debug, info};
use crate::errors::WikiError;
use crate::services::FileService;

/// Component for handling navigation and sidebar generation
pub struct NavigationComponent {
    file_service: FileService,
}

impl NavigationComponent {
    /// Create a new navigation component
    pub fn new(file_service: FileService) -> Self {
        debug!("Creating new NavigationComponent");
        Self { file_service }
    }

    /// Build sidebar HTML with table of contents
    pub fn build_sidebar_with_toc(&self, current_path: &str, toc: &str) -> Result<String, WikiError> {
        debug!("Building sidebar with TOC for path: '{}'", current_path);
        let start_time = std::time::Instant::now();
        
        let sidebar_html = self.build_sidebar_html(current_path)?;
        let toc_html = if !toc.is_empty() {
            format!("<div class=\"sidebar-toc\"><h4 class=\"sidebar-toc-title\">On This Page</h4>{}</div>", toc)
        } else {
            String::new()
        };
        
        let result = format!("{}{}", sidebar_html, toc_html);
        
        let duration = start_time.elapsed();
        info!("Sidebar with TOC built in {:?}ms for path: '{}'", duration.as_millis(), current_path);
        
        Ok(result)
    }

    /// Build basic sidebar HTML
    pub fn build_sidebar_html(&self, current_path: &str) -> Result<String, WikiError> {
        debug!("Building basic sidebar HTML for path: '{}'", current_path);
        let start_time = std::time::Instant::now();
        
        let mut html = String::new();
        html.push_str("<div class=\"sidebar-nav\">");
        html.push_str("<h3>Navigation</h3>");
        
        // Always list from root directory for consistent navigation
        let entries = self.file_service.list_directory(Path::new(""))?;
        debug!("Found {} entries in root directory", entries.len());
        
        html.push_str("<ul class=\"nav-list\">");
        for entry in entries {
            if !entry.name.starts_with('.') && entry.name != "index.md" { // Skip hidden files and index.md
                // entry_path should always be just the entry name for navigation structure
                let entry_path = entry.name.clone();
                
                let href = if entry.is_dir {
                    format!("/{}", entry_path)
                } else {
                    format!("/{}", entry_path.replace(".md", ""))
                };
                
                let display_name = if entry.is_dir {
                    entry.name.clone()
                } else {
                    entry.name.trim_end_matches(".md").to_string()
                };
                
                let is_current = current_path == entry_path || 
                    (entry.is_dir && current_path.starts_with(&format!("{}/", entry_path)));
                
                let current_class = if is_current { " class=\"current\"" } else { "" };
                
                if entry.is_dir {
                    html.push_str(&format!("<li class=\"nav-item has-sub{}\">", current_class));
                    html.push_str("<div class=\"nav-header\">");
                    html.push_str("<span class=\"nav-toggle\"></span>");
                    html.push_str(&format!("<span class=\"nav-text\">{}</span>", display_name));
                    html.push_str("</div>");
                    html.push_str("<ul class=\"nav-sub-list\">");
                    
                    // Recursively list sub-directories and files
                    debug!("Listing sub-directory: {:?}", entry_path);
                    if let Ok(sub_entries) = self.file_service.list_directory(Path::new(&entry_path)) {
                        debug!("Found {} sub-entries in {:?}", sub_entries.len(), entry_path);
                        for sub_entry in sub_entries {
                            if !sub_entry.name.starts_with('.') {
                                let sub_href = if sub_entry.is_dir {
                                    format!("/{}/{}", entry_path, sub_entry.name)
                                } else {
                                    format!("/{}/{}", entry_path, sub_entry.name.replace(".md", ""))
                                };
                                
                                let sub_display_name = if sub_entry.is_dir {
                                    sub_entry.name.clone()
                                } else {
                                    sub_entry.name.trim_end_matches(".md").to_string()
                                };
                                
                                let sub_is_current = current_path == format!("{}/{}", entry_path, sub_entry.name) ||
                                    (sub_entry.is_dir && current_path.starts_with(&format!("{}/{}/", entry_path, sub_entry.name)));
                                
                                let sub_current_class = if sub_is_current { " class=\"current\"" } else { "" };
                                
                                html.push_str(&format!("<li{}>", sub_current_class));
                                html.push_str(&format!("<a href=\"{}\">{}</a>", sub_href, sub_display_name));
                                html.push_str("</li>");
                            }
                        }
                    }
                    
                    html.push_str("</ul>");
                    html.push_str("</li>");
                } else {
                    html.push_str(&format!("<li{}>", current_class));
                    html.push_str(&format!("<a href=\"{}\">{}</a>", href, display_name));
                    html.push_str("</li>");
                }
            }
        }
        html.push_str("</ul>");
        html.push_str("</div>");
        
        // Add JavaScript for toggle functionality
        html.push_str("<script>
            document.addEventListener('DOMContentLoaded', function() {
                const navToggles = document.querySelectorAll('.nav-toggle');
                const navTexts = document.querySelectorAll('.nav-text');
                
                console.log('Found', navToggles.length, 'nav toggles and', navTexts.length, 'nav texts');
                
                navToggles.forEach(function(toggle) {
                    toggle.addEventListener('click', function() {
                        const parent = this.parentElement;
                        parent.classList.toggle('expanded');
                        console.log('Toggle clicked, expanded:', parent.classList.contains('expanded'));
                    });
                });
                
                navTexts.forEach(function(text) {
                    text.addEventListener('click', function() {
                        const parent = this.parentElement;
                        parent.classList.toggle('expanded');
                        console.log('Text clicked, expanded:', parent.classList.contains('expanded'));
                    });
                });
            });
        </script>");
        
        let duration = start_time.elapsed();
        info!("Basic sidebar HTML built in {:?}ms for path: '{}'", duration.as_millis(), current_path);
        
        Ok(html)
    }
}
