use log::{debug, info};
use crate::errors::WikiError;
use crate::types::MarkdownResult;

/// Service for handling markdown rendering
pub struct MarkdownService;

impl MarkdownService {
    /// Create a new markdown service
    pub fn new() -> Self {
        debug!("Creating new MarkdownService");
        Self
    }

    /// Render markdown with table of contents
    pub fn render_with_toc(&self, content: &str) -> Result<MarkdownResult, WikiError> {
        debug!("Starting markdown rendering with TOC, content length: {} chars", content.len());
        let start_time = std::time::Instant::now();
        
        let html = self.basic_markdown_to_html(content)?;
        let toc = self.generate_toc(content)?;
        
        let duration = start_time.elapsed();
        info!("Markdown rendering completed in {:?}ms", duration.as_millis());
        
        Ok(MarkdownResult {
            html,
            toc,
            title: self.extract_title(content),
        })
    }

    /// Extract title from markdown content
    fn extract_title(&self, content: &str) -> Option<String> {
        debug!("Extracting title from markdown content");
        
        // Look for frontmatter title first
        if content.starts_with("---") {
            for line in content.lines() {
                if line.starts_with("title:") {
                    let title = line.trim_start_matches("title:").trim().trim_matches('"').trim_matches('\'');
                    if !title.is_empty() {
                        debug!("Found title in frontmatter: '{}'", title);
                        return Some(title.to_string());
                    }
                }
                if line.starts_with("---") && line != content.lines().next().unwrap_or("") {
                    break; // End of frontmatter
                }
            }
        }
        
        // Look for first heading
        for line in content.lines() {
            if line.starts_with('#') {
                let title = line.trim_start_matches('#').trim();
                if !title.is_empty() {
                    debug!("Found title in heading: '{}'", title);
                    return Some(title.to_string());
                }
            }
        }
        
        debug!("No title found in markdown content");
        None
    }

    /// Convert basic markdown to HTML
    fn basic_markdown_to_html(&self, content: &str) -> Result<String, WikiError> {
        debug!("Converting markdown to HTML");
        
        let mut html = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut in_code_block = false;

        // Track nested lists using a stack
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum ListKind { Unordered, Ordered }
        struct ListFrame { kind: ListKind, indent_level: usize }
        let mut list_stack: Vec<ListFrame> = Vec::new();

        // Helper to close N list levels
        let close_list_levels = |levels: usize, out: &mut String, stack: &mut Vec<ListFrame>| {
            for _ in 0..levels {
                if let Some(frame) = stack.pop() {
                    match frame.kind {
                        ListKind::Unordered => out.push_str("</ul>\n"),
                        ListKind::Ordered => out.push_str("</ol>\n"),
                    }
                }
            }
        };
        // Helper to open a list of kind at given level
        let open_list = |kind: ListKind, out: &mut String, stack: &mut Vec<ListFrame>, indent_level: usize| {
            match kind {
                ListKind::Unordered => out.push_str("<ul>\n"),
                ListKind::Ordered => out.push_str("<ol>\n"),
            }
            stack.push(ListFrame { kind, indent_level });
        };
        
        while i < lines.len() {
            let line = lines[i];
            
            if line.starts_with("---") {
                // Skip frontmatter
                i += 1;
                while i < lines.len() && !lines[i].starts_with("---") {
                    i += 1;
                }
                i += 1;
                continue;
            }
            
            // Code blocks: triple backticks start/end
            if line.starts_with("```") {
                // If we are inside any open lists, close them before code blocks
                if !list_stack.is_empty() {
                    let levels = list_stack.len();
                    close_list_levels(levels, &mut html, &mut list_stack);
                }

                in_code_block = !in_code_block;
                if in_code_block {
                    let lang = line.trim_start_matches("```").trim();
                    html.push_str(&format!("<pre><code class=\"language-{}\">", lang));
                } else {
                    html.push_str("</code></pre>\n");
                }
                i += 1;
                continue;
            }

            if in_code_block {
                html.push_str(&format!("{}\n", escape_html(line)));
                i += 1;
                continue;
            }

            if line.starts_with('#') {
                // Close any open lists before headers
                if !list_stack.is_empty() {
                    let levels = list_stack.len();
                    close_list_levels(levels, &mut html, &mut list_stack);
                }
                let level = line.chars().take_while(|&c| c == '#').count();
                let text = line.trim_start_matches('#').trim();
                if !text.is_empty() {
                    let anchor = text.to_lowercase()
                        .chars()
                        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '-' })
                        .collect::<String>()
                        .replace(" ", "-");
                    let processed_text = self.process_inline_markdown(text);
                    html.push_str(&format!("<h{} id=\"{}\">{}</h{}>\n", level, anchor, processed_text, level));
                }
            } else {
                // Compute indentation (tabs count as 1, every 4 spaces as 1)
                let mut pos = 0usize;
                let mut tab_count = 0usize;
                let mut space_count = 0usize;
                for ch in line.chars() {
                    match ch {
                        '\t' => { tab_count += 1; pos += 1; },
                        ' ' => { space_count += 1; pos += 1; },
                        _ => break,
                    }
                }
                let indent_level = tab_count + (space_count / 4);

                // Determine if this is a list item
                let rest = &line[pos..];
                let mut is_list_item = false;
                let mut kind: Option<ListKind> = None;
                let mut content_start = pos;

                // Unordered markers: -, *, + followed by space
                if rest.starts_with("- ") || rest.starts_with("* ") || rest.starts_with("+ ") {
                    is_list_item = true;
                    kind = Some(ListKind::Unordered);
                    content_start = pos + 2;
                } else {
                    // Ordered marker: digits + '. '
                    let mut j = pos;
                    while j < line.len() {
                        if let Some(ch) = line[j..].chars().next() {
                            if ch.is_ascii_digit() { j += ch.len_utf8(); } else { break; }
                        } else { break; }
                    }
                    // Need at least one digit, then '.' and space
                    if j > pos {
                        let after_digits = &line[j..];
                        if after_digits.starts_with(". ") {
                            is_list_item = true;
                            kind = Some(ListKind::Ordered);
                            content_start = j + 2;
                        }
                    }
                }

                if is_list_item {
                    let this_kind = kind.unwrap_or(ListKind::Unordered);

                    // Adjust stack according to indent level and kind
                    let current_depth = list_stack.len();
                    let target_depth = indent_level + 1; // root list has depth 1

                    if target_depth < current_depth {
                        // Close extra levels
                        let levels = current_depth - target_depth;
                        close_list_levels(levels, &mut html, &mut list_stack);
                    }
                    // If same level but kind changed, close one and reopen
                    if let Some(top) = list_stack.last() {
                        if top.indent_level + 1 == target_depth && top.kind != this_kind {
                            close_list_levels(1, &mut html, &mut list_stack);
                        }
                    }
                    // Open lists until reaching target depth
                    while list_stack.len() < target_depth {
                        let current_len = list_stack.len();
                        open_list(this_kind, &mut html, &mut list_stack, current_len);
                    }

                    // Now add list item
                    let item_text = &line[content_start..].trim_end();
                    let processed = self.process_inline_markdown(item_text.trim());
                    html.push_str(&format!("<li>{}</li>\n", processed));
                } else if line.matches('|').count() > 1 {
                    // Close lists before tables
                    if !list_stack.is_empty() {
                        let levels = list_stack.len();
                        close_list_levels(levels, &mut html, &mut list_stack);
                    }
                    // Table
                    let table_html = self.render_table(&lines, i)?;
                    html.push_str(&table_html);
                    // Skip table lines
                    while i < lines.len() && lines[i].contains('|') {
                        i += 1;
                    }
                    continue;
                } else if line.trim().is_empty() {
                    // On blank line, close any open lists
                    if !list_stack.is_empty() {
                        let levels = list_stack.len();
                        close_list_levels(levels, &mut html, &mut list_stack);
                    }
                    html.push_str("<br>\n");
                } else {
                    // Non-list paragraph; close any open lists first
                    if !list_stack.is_empty() {
                        let levels = list_stack.len();
                        close_list_levels(levels, &mut html, &mut list_stack);
                    }
                    // Regular paragraph
                    let processed = self.process_inline_markdown(line);
                    if !processed.trim().is_empty() {
                        html.push_str(&format!("<p>{}</p>\n", processed));
                    }
                }
            }
            
            i += 1;
        }
        
        // Close any remaining open lists
        if !list_stack.is_empty() {
            let levels = list_stack.len();
            close_list_levels(levels, &mut html, &mut list_stack);
        }

        debug!("Markdown to HTML conversion completed, output length: {} chars", html.len());
        Ok(html)
    }

    /// Process inline markdown elements like links and code
    fn process_inline_markdown(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Process images ![alt](url) first
        result = self.process_images(&result);
        
        // Process links [text](url)
        result = self.process_links(&result);
        
        // Process inline code `code` - handle backticks properly
        result = self.process_inline_code(&result);
        
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
                            // Skip if content is empty or contains only whitespace
                            if !content.trim().is_empty() {
                                result.push_str(open_tag);
                                result.push_str(&content);
                                result.push_str(close_tag);
                                i = j + marker_len;
                                found = true;
                                break;
                            }
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
                // Find closing bracket
                let mut j = i + 2;
                while j < chars.len() && chars[j] != ']' {
                    j += 1;
                }
                
                if j < chars.len() && j + 1 < chars.len() && chars[j + 1] == '(' {
                    let alt_text: String = chars[i + 2..j].iter().collect();
                    let mut k = j + 2;
                    while k < chars.len() && chars[k] != ')' {
                        k += 1;
                    }
                    
                    if k < chars.len() {
                        let url: String = chars[j + 2..k].iter().collect();
                        result.push_str(&format!("<img src=\"{}\" alt=\"{}\">", 
                            escape_attr(&url), escape_attr(&alt_text)));
                        i = k + 1;
                        continue;
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
            if i < chars.len() && chars[i] == '[' {
                // Find closing bracket
                let mut j = i + 1;
                while j < chars.len() && chars[j] != ']' {
                    j += 1;
                }
                
                if j < chars.len() && j + 1 < chars.len() && chars[j + 1] == '(' {
                    let link_text: String = chars[i + 1..j].iter().collect();
                    let mut k = j + 2;
                    while k < chars.len() && chars[k] != ')' {
                        k += 1;
                    }
                    
                    if k < chars.len() {
                        let mut url: String = chars[j + 2..k].iter().collect();
                        
                        // Strip .md extension for internal links
                        if url.ends_with(".md") && !url.starts_with("http") {
                            url = url[..url.len() - 3].to_string();
                        }
                        
                        result.push_str(&format!("<a href=\"{}\">{}</a>", 
                            escape_attr(&url), escape_html(&link_text)));
                        i = k + 1;
                        continue;
                    }
                }
            }
            
            result.push(chars[i]);
            i += 1;
        }
        
        result
    }

    /// Process inline code `code`
    fn process_inline_code(&self, text: &str) -> String {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();
        
        while i < chars.len() {
            if i < chars.len() && chars[i] == '`' {
                // Find closing backtick
                let mut j = i + 1;
                while j < chars.len() && chars[j] != '`' {
                    j += 1;
                }
                
                if j < chars.len() {
                    let code_content: String = chars[i + 1..j].iter().collect();
                    // Skip if content is empty or contains only whitespace
                    if !code_content.trim().is_empty() {
                        result.push_str(&format!("<code>{}</code>", escape_html(&code_content)));
                        i = j + 1;
                        continue;
                    }
                }
            }
            
            result.push(chars[i]);
            i += 1;
        }
        
        result
    }

    /// Render table from markdown
    fn render_table(&self, lines: &[&str], start_idx: usize) -> Result<String, WikiError> {
        let mut html = String::new();
        html.push_str("<table>\n<thead>\n<tr>\n");
        
        // Parse header
        if start_idx < lines.len() {
            let header_line = lines[start_idx];
            let cells: Vec<&str> = header_line.split('|').collect();
            for cell in cells.iter().skip(1).take(cells.len().saturating_sub(2)) {
                let cell_content = cell.trim();
                if !cell_content.is_empty() {
                    html.push_str(&format!("<th>{}</th>\n", escape_html(cell_content)));
                }
            }
        }
        
        html.push_str("</tr>\n</thead>\n<tbody>\n");
        
        // Parse data rows
        let mut i = start_idx + 2; // Skip header and separator
        while i < lines.len() && lines[i].contains('|') {
            let row_line = lines[i];
            let cells: Vec<&str> = row_line.split('|').collect();
            
            html.push_str("<tr>\n");
            for cell in cells.iter().skip(1).take(cells.len().saturating_sub(2)) {
                let cell_content = cell.trim();
                if !cell_content.is_empty() {
                    html.push_str(&format!("<td>{}</td>\n", escape_html(cell_content)));
                } else {
                    html.push_str("<td></td>\n");
                }
            }
            html.push_str("</tr>\n");
            i += 1;
        }
        
        html.push_str("</tbody>\n</table>\n");
        Ok(html)
    }

    /// Generate table of contents
    fn generate_toc(&self, content: &str) -> Result<String, WikiError> {
        debug!("Generating table of contents");
        
        let mut toc = String::new();
        let mut items = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut in_code_block = false;
        
        while i < lines.len() {
            let line = lines[i];
            
            // Check for code block boundaries
            if line.starts_with("```") {
                in_code_block = !in_code_block;
                i += 1;
                continue;
            }
            
            // Skip processing if we're inside a code block
            if in_code_block {
                i += 1;
                continue;
            }
            
            // Process headers only when not in code blocks
            if line.starts_with('#') {
                let level = line.chars().take_while(|&c| c == '#').count();
                if level <= 6 { // Support H1-H6
                    let text = line.trim_start_matches('#').trim();
                    if !text.is_empty() {
                        let anchor = text.to_lowercase()
                            .chars()
                            .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '-' })
                            .collect::<String>()
                            .replace(" ", "-");
                        
                        items.push((level, text, anchor));
                    }
                }
            }
            
            i += 1;
        }
        
        if !items.is_empty() {
            toc.push_str("<ul class=\"toc\">\n");
            for (level, text, anchor) in &items {
                let indent = "  ".repeat(level - 1);
                toc.push_str(&format!("{}<li><a href=\"#{}\">{}</a></li>\n", 
                    indent, anchor, escape_html(text)));
            }
            toc.push_str("</ul>\n");
        }
        
        debug!("Generated TOC with {} items", items.len());
        Ok(toc)
    }
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

/// Escape HTML attribute values
fn escape_attr(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}
