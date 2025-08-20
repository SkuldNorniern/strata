use std::path::Path;
use log::{debug, info, warn, error};
use crate::errors::WikiError;
use crate::types::SearchResult;
use crate::services::FileService;

/// Service for handling search operations
pub struct SearchService {
    file_service: FileService,
}

impl SearchService {
    /// Create a new search service
    pub fn new(file_service: FileService) -> Self {
        Self { file_service }
    }

    /// Search for content in the wiki
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, WikiError> {
        if query.trim().is_empty() {
            debug!("Empty search query received");
            return Ok(Vec::new());
        }

        info!("Starting search for query: '{}'", query);
        let start_time = std::time::Instant::now();
        
        // Wrap the search in a panic handler to prevent crashes
        let search_result = std::panic::catch_unwind(|| {
            let mut results = Vec::new();
            self.search_directory(Path::new(""), query, &mut results).map(|_| results)
        });
        
        match search_result {
            Ok(Ok(mut results)) => {
                // Sort by relevance (simple implementation)
                results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
                
                let duration = start_time.elapsed();
                info!("Search completed in {:?}ms, found {} results", duration.as_millis(), results.len());
                
                if results.is_empty() {
                    warn!("No results found for query: '{}'", query);
                } else {
                    debug!("Top result relevance: {:.1}", results.first().unwrap().relevance);
                }
                
                Ok(results)
            }
            Ok(Err(e)) => {
                error!("Search failed with error: {:?}", e);
                Err(e)
            }
            Err(_) => {
                error!("Search panicked, returning empty results");
                warn!("Search panicked for query: '{}', returning empty results", query);
                Ok(Vec::new())
            }
        }
    }

    /// Recursively search through directories
    fn search_directory(
        &self,
        current_path: &Path,
        query: &str,
        results: &mut Vec<SearchResult>,
    ) -> Result<(), WikiError> {
        debug!("Searching directory: {:?}", current_path);
        
        let entries = self.file_service.list_directory(current_path)?;
        
        for entry in entries {
            let entry_path = if current_path.as_os_str().is_empty() {
                entry.path.clone()
            } else {
                current_path.join(&entry.name)
            };

            if entry.is_dir {
                // Recursively search subdirectories
                self.search_directory(&entry_path, query, results)?;
            } else if entry.name.ends_with(".md") {
                // Search in markdown files
                debug!("Searching markdown file: {:?}", entry_path);
                match self.file_service.read_file(&entry_path) {
                    Ok(content) => {
                        // Check if content contains the query (case-insensitive)
                        if content.to_lowercase().contains(&query.to_lowercase()) {
                            // Safely generate excerpt and calculate relevance
                            let excerpt = self.generate_excerpt_safe(&content, query);
                            let relevance = self.calculate_relevance(&content, query);
                            let title = self.extract_title(&content, &entry.name);
                            
                            debug!("Found match in {:?} with relevance: {:.1}", entry_path, relevance);
                            
                            results.push(SearchResult {
                                title,
                                path: entry_path.to_string_lossy().to_string(),
                                excerpt,
                                relevance,
                            });
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read file {:?}: {:?}", entry_path, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Extract title from markdown content or use filename
    fn extract_title(&self, content: &str, filename: &str) -> String {
        // Try to extract title from frontmatter or first heading
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("---") {
                // Look for title in frontmatter
                for line in content.lines() {
                    if line.starts_with("title:") {
                        let title = line.trim_start_matches("title:").trim().trim_matches('"').trim_matches('\'');
                        if !title.is_empty() {
                            return title.to_string();
                        }
                    }
                    if line.starts_with("---") && line != first_line {
                        break; // End of frontmatter
                    }
                }
            } else if first_line.starts_with('#') {
                // Extract from first heading
                let title = first_line.trim_start_matches('#').trim();
                if !title.is_empty() {
                    return title.to_string();
                }
            }
        }
        
        // Fallback to filename without extension
        filename.trim_end_matches(".md").to_string()
    }

    /// Calculate search relevance score
    fn calculate_relevance(&self, content: &str, query: &str) -> f32 {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();
        
        let mut score = 0.0;
        
        // Exact phrase match gets highest score
        if content_lower.contains(&query_lower) {
            score += 20.0;
        }
        
        // Word boundary matches
        let words: Vec<&str> = query_lower.split_whitespace().collect();
        for word in &words {
            if word.len() > 2 { // Only count words longer than 2 characters
                if content_lower.contains(word) {
                    score += 3.0;
                }
            }
        }
        
        // Title matches get bonus
        if let Some(first_line) = content.lines().next() {
            if first_line.to_lowercase().contains(&query_lower) {
                score += 15.0;
            }
            // Check individual words in title
            for word in &words {
                if word.len() > 2 && first_line.to_lowercase().contains(word) {
                    score += 5.0;
                }
            }
        }
        
        // Frontmatter matches get bonus
        if content.contains("---") {
            let frontmatter_end = content.find("---").unwrap_or(0);
            let frontmatter = &content[..frontmatter_end];
            if frontmatter.to_lowercase().contains(&query_lower) {
                score += 10.0;
            }
        }
        
        // Headings matches get bonus
        for line in content.lines() {
            if line.starts_with('#') {
                if line.to_lowercase().contains(&query_lower) {
                    score += 8.0;
                }
            }
        }
        
        score
    }

    /// Generate search result excerpt with better context
    fn generate_excerpt(&self, content: &str, query: &str) -> String {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();
        
        if let Some(pos) = content_lower.find(&query_lower) {
            // Convert byte position to char position for safe slicing
            let char_pos = content_lower.char_indices()
                .position(|(i, _)| i == pos)
                .unwrap_or(0);
            
            let start = char_pos.saturating_sub(100);
            let end = (char_pos + query.chars().count() + 100).min(content.chars().count());
            
            // Get the excerpt using char indices
            let excerpt: String = content.chars().skip(start).take(end - start).collect();
            
            // Try to start at a word boundary
            let mut final_start = 0;
            if start > 0 {
                if let Some(word_start) = excerpt.find(' ') {
                    final_start = word_start + 1;
                }
            }
            
            let final_excerpt = &excerpt[final_start..];
            
            if start > 0 {
                format!("...{}...", final_excerpt)
            } else {
                format!("{}...", final_excerpt)
            }
        } else {
            // Fallback to first meaningful content
            let lines: Vec<&str> = content.lines().collect();
            for line in lines.iter().take(3) {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("---") {
                    if trimmed.chars().count() > 50 {
                        let truncated: String = trimmed.chars().take(50).collect();
                        return format!("{}...", truncated);
                    } else {
                        return trimmed.to_string();
                    }
                }
            }
            
            // Last resort: first line
            content.lines().next().unwrap_or("").to_string()
        }
    }

    /// Safe version of generate_excerpt that handles UTF-8 errors gracefully
    fn generate_excerpt_safe(&self, content: &str, query: &str) -> String {
        match std::panic::catch_unwind(|| self.generate_excerpt(content, query)) {
            Ok(excerpt) => excerpt,
            Err(_) => {
                warn!("Failed to generate excerpt for content, using fallback");
                // Fallback to first line or simple content
                content.lines().next().unwrap_or("").to_string()
            }
        }
    }
}
