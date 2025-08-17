use std::path::Path;
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
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        self.search_directory(Path::new(""), query, &mut results)?;
        
        // Sort by relevance (simple implementation)
        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(results)
    }

    /// Recursively search through directories
    fn search_directory(
        &self,
        current_path: &Path,
        query: &str,
        results: &mut Vec<SearchResult>,
    ) -> Result<(), WikiError> {
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
                if let Ok(content) = self.file_service.read_file(&entry_path) {
                    if content.to_lowercase().contains(&query.to_lowercase()) {
                        let relevance = self.calculate_relevance(&content, query);
                        let excerpt = self.generate_excerpt(&content, query);
                        
                        results.push(SearchResult {
                            title: entry.name.trim_end_matches(".md").to_string(),
                            path: entry_path.to_string_lossy().to_string(),
                            excerpt,
                            relevance,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Calculate search relevance score
    fn calculate_relevance(&self, content: &str, query: &str) -> f32 {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();
        
        let mut score = 0.0;
        
        // Exact match gets highest score
        if content_lower.contains(&query_lower) {
            score += 10.0;
        }
        
        // Word boundary matches
        let words: Vec<&str> = query_lower.split_whitespace().collect();
        for word in words {
            if content_lower.contains(word) {
                score += 2.0;
            }
        }
        
        // Title matches get bonus
        if let Some(first_line) = content.lines().next() {
            if first_line.to_lowercase().contains(&query_lower) {
                score += 5.0;
            }
        }
        
        score
    }

    /// Generate search result excerpt
    fn generate_excerpt(&self, content: &str, query: &str) -> String {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();
        
        if let Some(pos) = content_lower.find(&query_lower) {
            let start = pos.saturating_sub(50);
            let end = (pos + query.len() + 50).min(content.len());
            
            let excerpt = &content[start..end];
            if start > 0 {
                format!("...{}...", excerpt)
            } else {
                format!("{}...", excerpt)
            }
        } else {
            // Fallback to first line
            content.lines().next().unwrap_or("").to_string()
        }
    }
}
