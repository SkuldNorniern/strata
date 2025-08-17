use std::path::{Path, PathBuf};
use crate::errors::WikiError;
use crate::types::DirEntry;

/// Service for handling file system operations
#[derive(Clone)]
pub struct FileService {
    base_dir: PathBuf,
}

impl FileService {
    /// Create a new file service
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// List directory contents with filtering
    pub fn list_directory(&self, path: &Path) -> Result<Vec<DirEntry>, WikiError> {
        let full_path = if path.as_os_str().is_empty() {
            self.base_dir.clone()
        } else {
            self.base_dir.join(path)
        };
        
        if !full_path.exists() {
            return Err(WikiError::NotFound);
        }

        let mut entries = Vec::new();
        let read_dir = std::fs::read_dir(&full_path)
            .map_err(|e| WikiError::Io(e))?;

        for entry in read_dir {
            let entry = entry.map_err(|e| WikiError::Io(e))?;
            let file_type = entry.file_type().map_err(|e| WikiError::Io(e))?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Skip hidden files and directories
            if name.starts_with('.') {
                continue;
            }

            let is_dir = file_type.is_dir();
            let entry_path = entry.path();
            let relative_path = entry_path
                .strip_prefix(&self.base_dir)
                .map_err(|_| WikiError::InvalidPath)?;

            entries.push(DirEntry {
                name,
                is_dir,
                path: relative_path.to_path_buf(),
            });
        }

        // Sort: directories first, then files, both alphabetically
        entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        Ok(entries)
    }

    /// Read file content
    pub fn read_file(&self, path: &Path) -> Result<String, WikiError> {
        let full_path = if path.as_os_str().is_empty() {
            self.base_dir.clone()
        } else {
            self.base_dir.join(path)
        };
        std::fs::read_to_string(&full_path)
            .map_err(|e| WikiError::Io(e))
    }

    /// Check if file exists
    pub fn file_exists(&self, path: &Path) -> bool {
        self.base_dir.join(path).exists()
    }

    /// Get file metadata
    pub fn get_metadata(&self, path: &Path) -> Result<std::fs::Metadata, WikiError> {
        let full_path = self.base_dir.join(path);
        std::fs::metadata(&full_path)
            .map_err(|e| WikiError::Io(e))
    }

    /// Determine content type for a file
    pub fn content_type_for(&self, path: &Path) -> &'static str {
        match path.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()) {
            Some(ref ext) if ext == "html" => "text/html; charset=utf-8",
            Some(ref ext) if ext == "css" => "text/css; charset=utf-8",
            Some(ref ext) if ext == "js" => "application/javascript; charset=utf-8",
            Some(ref ext) if ext == "json" => "application/json; charset=utf-8",
            Some(ref ext) if ext == "svg" => "image/svg+xml",
            Some(ref ext) if ext == "png" => "image/png",
            Some(ref ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
            Some(ref ext) if ext == "gif" => "image/gif",
            Some(ref ext) if ext == "txt" => "text/plain; charset=utf-8",
            Some(ref ext) if ext == "md" => "text/markdown; charset=utf-8",
            _ => "application/octet-stream",
        }
    }
}
