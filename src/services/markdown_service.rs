use crate::errors::WikiError;
use crate::types::MarkdownResult;

/// Service for handling markdown rendering
pub struct MarkdownService;

impl MarkdownService {
    /// Create a new markdown service
    pub fn new() -> Self {
        Self
    }

    /// Render markdown with table of contents
    pub fn render_with_toc(&self, content: &str) -> Result<MarkdownResult, WikiError> {
        let (html, toc, title) = self.render_markdown_with_toc(content)?;
        
        Ok(MarkdownResult {
            html,
            toc,
            title,
        })
    }

    /// Render markdown content with table of contents generation
    fn render_markdown_with_toc(&self, content: &str) -> Result<(String, String, Option<String>), WikiError> {
        let html = self.basic_markdown_to_html(content);
        let toc = self.generate_toc(content);
        let title = self.extract_title(content);
        
        Ok((html, toc, title))
    }

    /// Basic markdown to HTML conversion
    fn basic_markdown_to_html(&self, content: &str) -> String {
        let mut html = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut in_code_block = false;
        let mut in_list = false;
        let mut in_ordered_list = false;
        let mut in_task_list = false;
        let mut in_frontmatter = false;
        let mut frontmatter_ended = false;
        let mut in_table = false;
        let mut table_headers = Vec::new();
        let mut table_rows = Vec::new();
        
        for (_i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Handle frontmatter
            if trimmed == "---" {
                if !frontmatter_ended {
                    in_frontmatter = !in_frontmatter;
                    if !in_frontmatter {
                        frontmatter_ended = true;
                    }
                    continue;
                }
            }
            
            // Skip frontmatter content
            if in_frontmatter {
                continue;
            }
            
            if trimmed.starts_with("```") {
                // Close any open elements
                if in_table {
                    html.push_str(&self.render_table(&table_headers, &table_rows));
                    in_table = false;
                    table_headers.clear();
                    table_rows.clear();
                }
                self.close_lists(&mut html, &mut in_list, &mut in_ordered_list, &mut in_task_list);
                
                in_code_block = !in_code_block;
                if in_code_block {
                    html.push_str("<pre><code>");
                } else {
                    html.push_str("</code></pre>");
                }
                continue;
            }
            
            if in_code_block {
                html.push_str(&format!("{}\n", line));
                continue;
            }
            
            // Handle horizontal rules
            if (trimmed == "---" || trimmed == "***" || trimmed == "___") && trimmed.len() >= 3 {
                self.close_lists(&mut html, &mut in_list, &mut in_ordered_list, &mut in_task_list);
                html.push_str("<hr>");
                continue;
            }
            
            // Handle tables
            if trimmed.starts_with("|") && trimmed.ends_with("|") {
                if !in_table {
                    in_table = true;
                    self.close_lists(&mut html, &mut in_list, &mut in_ordered_list, &mut in_task_list);
                }
                
                let cells: Vec<&str> = trimmed.split('|').filter(|s| !s.is_empty()).collect();
                
                if table_headers.is_empty() {
                    // First row is headers
                    table_headers = cells;
                } else if trimmed.contains("---") {
                    // Separator row, skip
                    continue;
                } else {
                    // Data row
                    table_rows.push(cells);
                }
                continue;
            } else if in_table {
                // End of table
                html.push_str(&self.render_table(&table_headers, &table_rows));
                in_table = false;
                table_headers.clear();
                table_rows.clear();
            }
            
            // Handle all heading levels
            if trimmed.starts_with("# ") {
                let title = &trimmed[2..];
                let id = title.to_lowercase().replace(" ", "-");
                html.push_str(&format!("<h1 id=\"{}\">{}</h1>", id, title));
            } else if trimmed.starts_with("## ") {
                let title = &trimmed[3..];
                let id = title.to_lowercase().replace(" ", "-");
                html.push_str(&format!("<h2 id=\"{}\">{}</h2>", id, title));
            } else if trimmed.starts_with("### ") {
                let title = &trimmed[4..];
                let id = title.to_lowercase().replace(" ", "-");
                html.push_str(&format!("<h3 id=\"{}\">{}</h3>", id, title));
            } else if trimmed.starts_with("#### ") {
                let title = &trimmed[5..];
                let id = title.to_lowercase().replace(" ", "-");
                html.push_str(&format!("<h4 id=\"{}\">{}</h4>", id, title));
            } else if trimmed.starts_with("##### ") {
                let title = &trimmed[6..];
                let id = title.to_lowercase().replace(" ", "-");
                html.push_str(&format!("<h5 id=\"{}\">{}</h5>", id, title));
            } else if trimmed.starts_with("###### ") {
                let title = &trimmed[7..];
                let id = title.to_lowercase().replace(" ", "-");
                html.push_str(&format!("<h6 id=\"{}\">{}</h6>", id, title));
            } else if trimmed.starts_with("- [ ] ") {
                self.close_lists(&mut html, &mut in_list, &mut in_ordered_list, &mut in_task_list);
                if !in_task_list {
                    html.push_str("<ul class=\"task-list\">");
                    in_task_list = true;
                }
                let content = self.process_inline_markdown(&trimmed[6..]);
                html.push_str(&format!("<li class=\"task-list-item\"><input type=\"checkbox\" disabled> {}</li>", content));
            } else if trimmed.starts_with("- [x] ") {
                self.close_lists(&mut html, &mut in_list, &mut in_ordered_list, &mut in_task_list);
                if !in_task_list {
                    html.push_str("<ul class=\"task-list\">");
                    in_task_list = true;
                }
                let content = self.process_inline_markdown(&trimmed[6..]);
                html.push_str(&format!("<li class=\"task-list-item\"><input type=\"checkbox\" checked disabled> {}</li>", content));
            } else if trimmed.starts_with("- ") {
                self.close_lists(&mut html, &mut in_list, &mut in_ordered_list, &mut in_task_list);
                if !in_list {
                    html.push_str("<ul>");
                    in_list = true;
                }
                let content = self.process_inline_markdown(&trimmed[2..]);
                html.push_str(&format!("<li>{}</li>", content));
            } else if trimmed.matches(|c: char| c.is_ascii_digit()).next().is_some() && trimmed.contains(". ") {
                // Handle ordered lists (1. item, 2. item, etc.)
                if in_list {
                    html.push_str("</ul>");
                    in_list = false;
                }
                if in_task_list {
                    html.push_str("</ul>");
                    in_task_list = false;
                }
                if !in_ordered_list {
                    html.push_str("<ol>");
                    in_ordered_list = true;
                }
                let content = self.process_inline_markdown(&trimmed[trimmed.find(". ").unwrap() + 2..]);
                html.push_str(&format!("<li>{}</li>", content));
            } else if trimmed.is_empty() {
                self.close_lists(&mut html, &mut in_list, &mut in_ordered_list, &mut in_task_list);
                html.push_str("<br>");
            } else if trimmed.starts_with("`") && trimmed.ends_with("`") && trimmed.len() > 2 {
                html.push_str(&format!("<code>{}</code>", &trimmed[1..trimmed.len()-1]));
            } else if trimmed.starts_with("> ") {
                // Handle blockquotes
                let content = self.process_inline_markdown(&trimmed[2..]);
                html.push_str(&format!("<blockquote><p>{}</p></blockquote>", content));
            } else {
                let content = self.process_inline_markdown(trimmed);
                html.push_str(&format!("<p>{}</p>", content));
            }
        }
        
        // Close any remaining open elements
        self.close_lists(&mut html, &mut in_list, &mut in_ordered_list, &mut in_task_list);
        if in_table {
            html.push_str(&self.render_table(&table_headers, &table_rows));
        }
        
        html
    }

    /// Render table HTML
    fn render_table(&self, headers: &[&str], rows: &[Vec<&str>]) -> String {
        let mut html = String::new();
        html.push_str("<table>");
        
        // Headers
        html.push_str("<thead><tr>");
        for header in headers {
            html.push_str(&format!("<th>{}</th>", header.trim()));
        }
        html.push_str("</tr></thead>");
        
        // Body
        html.push_str("<tbody>");
        for row in rows {
            html.push_str("<tr>");
            for (i, cell) in row.iter().enumerate() {
                if i < headers.len() {
                    html.push_str(&format!("<td>{}</td>", cell.trim()));
                }
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody></table>");
        
        html
    }

    /// Process inline markdown elements like links and code
    fn process_inline_markdown(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Process images ![alt](url) first
        result = self.process_images(&result);
        
        // Process links [text](url)
        result = self.process_links(&result);
        
        // Process inline code `code`
        result = self.replace_emphasis(&result, "`", "<code>", "</code>");
        
        // Process strikethrough ~~text~~
        result = self.replace_emphasis(&result, "~~", "<del>", "</del>");
        
        // Process bold italic ***text*** first (before **text**)
        result = self.replace_emphasis(&result, "***", "<strong><em>", "</em></strong>");
        
        // Process bold **text**
        result = self.replace_emphasis(&result, "**", "<strong>", "</strong>");
        
        // Process italic *text* last (after **text**)
        result = self.replace_emphasis(&result, "*", "<em>", "</em>");
        
        result
    }
    
    /// Helper function to replace emphasis markers
    fn replace_emphasis(&self, text: &str, marker: &str, open_tag: &str, close_tag: &str) -> String {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();
        let marker_chars: Vec<char> = marker.chars().collect();
        let marker_len = marker_chars.len();
        
        while i < chars.len() {
            if i + marker_len <= chars.len() {
                let mut is_marker = true;
                for (k, &marker_char) in marker_chars.iter().enumerate() {
                    if chars[i + k] != marker_char {
                        is_marker = false;
                        break;
                    }
                }
                
                if is_marker {
                    // Look for closing marker
                    let mut j = i + marker_len;
                    let mut found = false;
                    while j + marker_len <= chars.len() {
                        let mut is_closing = true;
                        for (k, &marker_char) in marker_chars.iter().enumerate() {
                            if chars[j + k] != marker_char {
                                is_closing = false;
                                break;
                            }
                        }
                        
                        if is_closing {
                            let content: String = chars[i + marker_len..j].iter().collect();
                            result.push_str(open_tag);
                            result.push_str(&content);
                            result.push_str(close_tag);
                            i = j + marker_len;
                            found = true;
                            break;
                        }
                        j += 1;
                    }
                    
                    if !found {
                        // No closing marker found, add as regular text
                        result.push(chars[i]);
                        i += 1;
                        continue;
                    }
                    continue;
                }
            }
            
            result.push(chars[i]);
            i += 1;
        }
        
        result
    }
    
    /// Process images ![alt](url)
    fn process_images(&self, text: &str) -> String {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();
        
        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '!' && chars[i + 1] == '[' {
                let mut j = i + 2;
                while j < chars.len() && chars[j] != ']' {
                    j += 1;
                }
                
                if j < chars.len() && chars[j] == ']' {
                    if j + 1 < chars.len() && chars[j + 1] == '(' {
                        let mut k = j + 2;
                        while k < chars.len() && chars[k] != ')' {
                            k += 1;
                        }
                        
                        if k < chars.len() && chars[k] == ')' {
                            let alt_text: String = chars[i + 2..j].iter().collect();
                            let url: String = chars[j + 2..k].iter().collect();
                            result.push_str(&format!("<img src=\"{}\" alt=\"{}\">", url, alt_text));
                            i = k + 1;
                            continue;
                        }
                    }
                }
            }
            
            result.push(chars[i]);
            i += 1;
        }
        
        result
    }
    
    /// Process links [text](url)
    fn process_links(&self, text: &str) -> String {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();
        
        while i < chars.len() {
            if chars[i] == '[' {
                let mut j = i + 1;
                while j < chars.len() && chars[j] != ']' {
                    j += 1;
                }
                
                if j < chars.len() && chars[j] == ']' {
                    if j + 1 < chars.len() && chars[j + 1] == '(' {
                        let mut k = j + 2;
                        while k < chars.len() && chars[k] != ')' {
                            k += 1;
                        }
                        
                        if k < chars.len() && chars[k] == ')' {
                            let link_text: String = chars[i + 1..j].iter().collect();
                            let mut url: String = chars[j + 2..k].iter().collect();
                            
                            if url.ends_with(".md") {
                                url = url[..url.len()-3].to_string();
                            }
                            
                            result.push_str(&format!("<a href=\"{}\">{}</a>", url, link_text));
                            i = k + 1;
                            continue;
                        }
                    }
                }
            }
            
            result.push(chars[i]);
            i += 1;
        }
        
        result
    }

    /// Generate table of contents from markdown content
    fn generate_toc(&self, content: &str) -> String {
        let mut toc = String::new();
        toc.push_str("<div class=\"toc\"><div class=\"toc-title\">Table of Contents</div><ul>");
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                let title = &trimmed[2..];
                let id = title.to_lowercase().replace(" ", "-");
                toc.push_str(&format!("<li><a href=\"#{}\">{}</a></li>", id, title));
            } else if trimmed.starts_with("## ") {
                let title = &trimmed[3..];
                let id = title.to_lowercase().replace(" ", "-");
                toc.push_str(&format!("<li><a href=\"#{}\" style=\"margin-left: 20px;\">{}</a></li>", id, title));
            } else if trimmed.starts_with("### ") {
                let title = &trimmed[4..];
                let id = title.to_lowercase().replace(" ", "-");
                toc.push_str(&format!("<li><a href=\"#{}\" style=\"margin-left: 40px;\">{}</a></li>", id, title));
            } else if trimmed.starts_with("#### ") {
                let title = &trimmed[5..];
                let id = title.to_lowercase().replace(" ", "-");
                toc.push_str(&format!("<li><a href=\"#{}\" style=\"margin-left: 60px;\">{}</a></li>", id, title));
            } else if trimmed.starts_with("##### ") {
                let title = &trimmed[6..];
                let id = title.to_lowercase().replace(" ", "-");
                toc.push_str(&format!("<li><a href=\"#{}\" style=\"margin-left: 80px;\">{}</a></li>", id, title));
            } else if trimmed.starts_with("###### ") {
                let title = &trimmed[7..];
                let id = title.to_lowercase().replace(" ", "-");
                toc.push_str(&format!("<li><a href=\"#{}\" style=\"margin-left: 100px;\">{}</a></li>", id, title));
            }
        }
        
        toc.push_str("</ul></div>");
        toc
    }

    /// Extract title from markdown content
    fn extract_title(&self, content: &str) -> Option<String> {
        content.lines()
            .find(|line| line.trim().starts_with("# "))
            .map(|line| line.trim()[2..].to_string())
    }

    /// Helper function to close all open lists
    fn close_lists(&self, html: &mut String, in_list: &mut bool, in_ordered_list: &mut bool, in_task_list: &mut bool) {
        if *in_list {
            html.push_str("</ul>");
            *in_list = false;
        }
        if *in_ordered_list {
            html.push_str("</ol>");
            *in_ordered_list = false;
        }
        if *in_task_list {
            html.push_str("</ul>");
            *in_task_list = false;
        }
    }
}
