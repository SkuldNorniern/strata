use std::path::{Path, PathBuf};
use std::fs;
use log::{debug, info, warn, error};
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
        debug!("Creating FileService with base directory: {:?}", base_dir);
        Self { base_dir }
    }

    /// List directory contents
    pub fn list_directory(&self, path: &Path) -> Result<Vec<DirEntry>, WikiError> {
        let full_path = self.base_dir.join(path);
        debug!("Listing directory: {:?} (full path: {:?})", path, full_path);
        
        if !full_path.exists() {
            warn!("Directory does not exist: {:?}", full_path);
            return Err(WikiError::NotFound);
        }
        
        if !full_path.is_dir() {
            warn!("Path is not a directory: {:?}", full_path);
            return Err(WikiError::InvalidPath);
        }
        
        let entries = fs::read_dir(&full_path)
            .map_err(|e| {
                error!("Failed to read directory {:?}: {}", full_path, e);
                WikiError::Io(e)
            })?;
        
        let mut result = Vec::new();
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let is_dir = entry.file_type()
                        .map(|ft| ft.is_dir())
                        .unwrap_or(false);
                    let name = entry.file_name().to_string_lossy().to_string();
                    let entry_path = if path.as_os_str().is_empty() {
                        PathBuf::from(&name)
                    } else {
                        path.join(&name)
                    };
                    
                    debug!("Found entry: {} (is_dir: {})", name, is_dir);
                    
                    result.push(DirEntry {
                        name,
                        is_dir,
                        path: entry_path,
                    });
                }
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                }
            }
        }
        
        info!("Listed directory {:?}, found {} entries", path, result.len());
        Ok(result)
    }

    /// Read file content
    pub fn read_file(&self, path: &Path) -> Result<String, WikiError> {
        let full_path = self.base_dir.join(path);
        debug!("Reading file: {:?} (full path: {:?})", path, full_path);
        
        if !full_path.exists() {
            warn!("File does not exist: {:?}", full_path);
            return Err(WikiError::NotFound);
        }
        
        if !full_path.is_file() {
            warn!("Path is not a file: {:?}", full_path);
            return Err(WikiError::InvalidPath);
        }
        
        let content = fs::read_to_string(&full_path)
            .map_err(|e| {
                error!("Failed to read file {:?}: {}", full_path, e);
                WikiError::Io(e)
            })?;
        
        info!("Read file {:?}, {} bytes", path, content.len());
        Ok(content)
    }

    /// Check if file exists
    pub fn file_exists(&self, path: &Path) -> bool {
        let full_path = self.base_dir.join(path);
        let exists = full_path.exists() && full_path.is_file();
        debug!("File exists check: {:?} -> {}", path, exists);
        exists
    }

    /// Get file metadata
    pub fn get_metadata(&self, path: &Path) -> Result<fs::Metadata, WikiError> {
        let full_path = self.base_dir.join(path);
        debug!("Getting metadata for: {:?} (full path: {:?})", path, full_path);
        
        let metadata = fs::metadata(&full_path)
            .map_err(|e| {
                error!("Failed to get metadata for {:?}: {}", full_path, e);
                WikiError::Io(e)
            })?;
        
        Ok(metadata)
    }

    /// Determine content type for a file
    pub fn content_type_for(&self, path: &Path) -> String {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let content_type = match extension.as_str() {
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "txt" => "text/plain",
            "md" => "text/markdown",
            _ => "application/octet-stream",
        };
        
        debug!("Content type for {:?}: {} (extension: {})", path, content_type, extension);
        content_type.to_string()
    }
}
