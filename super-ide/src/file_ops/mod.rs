//! File operations module for Super IDE
//! 
//! Provides file management capabilities including:
//! - File reading and writing
//! - File discovery and navigation
//! - File change detection
//! - Project structure analysis
//! - Integration with Git for version control

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{PathBuf, Path};
use tokio::fs;
use walkdir::WalkDir;
use notify::{RecommendedWatcher, RecursiveMode, Event, Watcher};
use notify::event::{EventKind, ModifyKind};
use chrono::{DateTime, Utc, TimeZone};
use std::sync::Arc;
use tokio::sync::RwLock;

/// File information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub is_directory: bool,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub is_hidden: bool,
    pub git_status: Option<FileGitStatus>,
}

/// Git status for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileGitStatus {
    pub status: String,
    pub staged: bool,
    pub modified: bool,
    pub untracked: bool,
}

/// Project structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub root_path: PathBuf,
    pub files: Vec<FileInfo>,
    pub directories: Vec<FileInfo>,
    pub total_files: usize,
    pub total_size: u64,
}

/// File operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationResult {
    pub success: bool,
    pub message: String,
    pub bytes_written: Option<u64>,
    pub lines_read: Option<u32>,
}

/// File change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub event_type: FileChangeType,
    pub timestamp: DateTime<Utc>,
}

/// Types of file change events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// File operation error types
#[derive(Debug, thiserror::Error)]
pub enum FileOperationError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Watch error: {0}")]
    WatchError(String),
}

/// File manager for handling file operations
#[derive(Debug)]
pub struct FileManager {
    base_path: PathBuf,
    file_watcher: Option<RecommendedWatcher>,
    change_events: Arc<RwLock<Vec<FileChangeEvent>>>,
}

impl FileManager {
    /// Create a new file manager
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            file_watcher: None,
            change_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize file monitoring
    pub async fn initialize_monitoring(&mut self) -> Result<()> {
        let path = self.base_path.clone();
        let events = self.change_events.clone();

        let watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                if let Ok(event) = result {
                    if let EventKind::Modify(_) = event.kind {
                        let file_event = FileChangeEvent {
                            path: event.paths[0].clone(),
                            event_type: FileChangeType::Modified,
                            timestamp: Utc::now(),
                        };

                        // Add to events list
                        tokio::spawn(async move {
                            let mut events_lock = events.write().await;
                            events_lock.push(file_event);
                            
                            // Keep only recent events (last 1000)
                            if events_lock.len() > 1000 {
                                events_lock.drain(0..events_lock.len() - 1000);
                            }
                        });
                    }
                }
            },
            notify::Config::default()
        ).map_err(|e| FileOperationError::WatchError(e.to_string()))?;

        watcher
            .watch(&self.base_path, RecursiveMode::Recursive)
            .map_err(|e| FileOperationError::WatchError(e.to_string()))?;

        self.file_watcher = Some(watcher);
        Ok(())
    }

    /// Read file contents
    pub async fn read_file(&self, path: &Path) -> Result<String> {
        let full_path = self.base_path.join(path);
        
        if !full_path.exists() {
            return Err(FileOperationError::FileNotFound(full_path).into());
        }

        if full_path.is_dir() {
            return Err(FileOperationError::InvalidPath("Path is a directory".to_string()).into());
        }

        let content = fs::read_to_string(&full_path).await?;
        Ok(content)
    }

    /// Write file contents
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<FileOperationResult> {
        let full_path = self.base_path.join(path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let _bytes_written = fs::write(&full_path, content).await?;
        
        Ok(FileOperationResult {
            success: true,
            message: "File written successfully".to_string(),
            bytes_written: Some(content.len() as u64),
            lines_read: Some(content.lines().count() as u32),
        })
    }

    /// Create a new file
    pub async fn create_file(&self, path: &Path, content: Option<&str>) -> Result<FileOperationResult> {
        let content = content.unwrap_or("");
        self.write_file(path, content).await
    }

    /// Delete a file
    pub async fn delete_file(&self, path: &Path) -> Result<FileOperationResult> {
        let full_path = self.base_path.join(path);
        
        if !full_path.exists() {
            return Err(FileOperationError::FileNotFound(full_path).into());
        }

        if full_path.is_dir() {
            return Err(FileOperationError::InvalidPath("Use delete_directory for directories".to_string()).into());
        }

        fs::remove_file(&full_path).await?;
        
        Ok(FileOperationResult {
            success: true,
            message: "File deleted successfully".to_string(),
            bytes_written: None,
            lines_read: None,
        })
    }

    /// Create a directory
    pub async fn create_directory(&self, path: &Path) -> Result<FileOperationResult> {
        let full_path = self.base_path.join(path);
        
        fs::create_dir_all(&full_path).await?;
        
        Ok(FileOperationResult {
            success: true,
            message: "Directory created successfully".to_string(),
            bytes_written: None,
            lines_read: None,
        })
    }

    /// Delete a directory
    pub async fn delete_directory(&self, path: &Path) -> Result<FileOperationResult> {
        let full_path = self.base_path.join(path);
        
        if !full_path.exists() {
            return Err(FileOperationError::FileNotFound(full_path).into());
        }

        if !full_path.is_dir() {
            return Err(FileOperationError::InvalidPath("Path is not a directory".to_string()).into());
        }

        fs::remove_dir_all(&full_path).await?;
        
        Ok(FileOperationResult {
            success: true,
            message: "Directory deleted successfully".to_string(),
            bytes_written: None,
            lines_read: None,
        })
    }

    /// List files and directories in a path
    pub async fn list_directory(&self, path: &Path) -> Result<Vec<FileInfo>> {
        let full_path = self.base_path.join(path);
        
        if !full_path.exists() {
            return Err(FileOperationError::FileNotFound(full_path).into());
        }

        if !full_path.is_dir() {
            return Err(FileOperationError::InvalidPath("Path is not a directory".to_string()).into());
        }

        let mut entries = Vec::new();
        let mut entries_read = fs::read_dir(&full_path).await?;

        while let Some(entry) = entries_read.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;
            let file_name = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            // Skip hidden files and directories
            if file_name.starts_with('.') && file_name != "." {
                continue;
            }

            let file_info = FileInfo {
                path: path.strip_prefix(&self.base_path)?.to_path_buf(),
                name: file_name.to_string(),
                extension: path.extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string()),
                size: metadata.len(),
                is_directory: metadata.is_dir(),
                created_at: metadata.created()
                    .map(|time| Utc.timestamp_opt(time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64, 0).unwrap())
                    .unwrap_or_else(|_| Utc::now()),
                modified_at: metadata.modified()
                    .map(|time| Utc.timestamp_opt(time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64, 0).unwrap())
                    .unwrap_or_else(|_| Utc::now()),
                is_hidden: file_name.starts_with('.'),
                git_status: None, // Would be populated with actual Git integration
            };

            entries.push(file_info);
        }

        // Sort entries: directories first, then files, alphabetically
        entries.sort_by(|a, b| {
            if a.is_directory != b.is_directory {
                b.is_directory.cmp(&a.is_directory) // Directories first
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        Ok(entries)
    }

    /// Get project structure
    pub async fn get_project_structure(&self, max_depth: u32) -> Result<ProjectStructure> {
        let mut files = Vec::new();
        let mut directories = Vec::new();
        let mut total_size = 0u64;

        for entry in WalkDir::new(&self.base_path)
            .max_depth(max_depth as usize)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "target" && name != "node_modules"
            })
        {
            if let Ok(entry) = entry {
                let path = entry.path();
                let file_name = path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                let relative_path = path.strip_prefix(&self.base_path).unwrap_or(path);

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue, // Skip files we can't read metadata for
                };

                let file_info = FileInfo {
                    path: relative_path.to_path_buf(),
                    name: file_name.to_string(),
                    extension: path.extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string()),
                    size: metadata.len(),
                    is_directory: metadata.is_dir(),
                    created_at: metadata.created()
                        .map(|time| Utc.timestamp_opt(time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64, 0).unwrap())
                        .unwrap_or_else(|_| Utc::now()),
                    modified_at: metadata.modified()
                        .map(|time| Utc.timestamp_opt(time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64, 0).unwrap())
                        .unwrap_or_else(|_| Utc::now()),
                    is_hidden: file_name.starts_with('.'),
                    git_status: None,
                };

                total_size += file_info.size;

                if file_info.is_directory {
                    directories.push(file_info);
                } else {
                    files.push(file_info);
                }
            }
        }

        Ok(ProjectStructure {
            root_path: self.base_path.clone(),
            files,
            directories,
            total_files: files.len() + directories.len(),
            total_size,
        })
    }

    /// Search for files by name
    pub async fn search_files(&self, pattern: &str, case_sensitive: bool) -> Result<Vec<FileInfo>> {
        let mut results = Vec::new();
        let pattern = if case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        for entry in WalkDir::new(&self.base_path)
            .max_depth(10)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "target" && name != "node_modules"
            })
        {
            if let Ok(entry) = entry {
                let path = entry.path();
                let file_name = path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                let search_name = if case_sensitive {
                    file_name.to_string()
                } else {
                    file_name.to_lowercase()
                };

                if search_name.contains(&pattern) {
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue, // Skip files we can't read metadata for
                    };
                    let relative_path = path.strip_prefix(&self.base_path).unwrap_or(path);

                    let file_info = FileInfo {
                        path: relative_path.to_path_buf(),
                        name: file_name.to_string(),
                        extension: path.extension()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string()),
                        size: metadata.len(),
                        is_directory: metadata.is_dir(),
                        created_at: metadata.created()
                            .map(|time| Utc.timestamp_opt(time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64, 0).unwrap())
                            .unwrap_or_else(|_| Utc::now()),
                        modified_at: metadata.modified()
                            .map(|time| Utc.timestamp_opt(time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64, 0).unwrap())
                            .unwrap_or_else(|_| Utc::now()),
                        is_hidden: file_name.starts_with('.'),
                        git_status: None,
                    };

                    results.push(file_info);
                }
            }
        }

        Ok(results)
    }

    /// Get file change events
    pub async fn get_recent_changes(&self, limit: usize) -> Vec<FileChangeEvent> {
        let events_lock = self.change_events.read().await;
        let events = events_lock.clone();
        let recent_events = if events.len() > limit {
            events[events.len() - limit..].to_vec()
        } else {
            events
        };
        recent_events
    }

    /// Check if a path exists
    pub fn path_exists(&self, path: &Path) -> bool {
        self.base_path.join(path).exists()
    }

    /// Get the full path for a relative path
    pub fn get_full_path(&self, path: &Path) -> PathBuf {
        self.base_path.join(path)
    }

    /// Get relative path from full path
    pub fn get_relative_path(&self, full_path: &Path) -> Option<PathBuf> {
        full_path.strip_prefix(&self.base_path).ok().map(|p| p.to_path_buf())
    }

    /// Detect language from file extension
    pub fn detect_language(&self, path: &Path) -> String {
        match path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .as_deref() {
            Some("rs") => "rust",
            Some("py") => "python", 
            Some("js") => "javascript",
            Some("ts") => "typescript",
            Some("java") => "java",
            Some("cpp") | Some("cc") | Some("cxx") => "cpp",
            Some("c") => "c",
            Some("cs") => "csharp",
            Some("go") => "go",
            Some("php") => "php",
            Some("rb") => "ruby",
            Some("swift") => "swift",
            Some("kt") => "kotlin",
            Some("scala") => "scala",
            Some("sh") | Some("bash") => "shell",
            Some("html") => "html",
            Some("css") => "css",
            Some("json") => "json",
            Some("xml") => "xml",
            Some("yaml") | Some("yml") => "yaml",
            Some("md") => "markdown",
            Some("txt") => "plaintext",
            _ => "plaintext",
        }.to_string()
    }

    /// Get file info for a specific file
    pub async fn get_file_info(&self, path: &Path) -> Result<FileInfo> {
        let full_path = self.base_path.join(path);
        
        if !full_path.exists() {
            return Err(FileOperationError::FileNotFound(full_path).into());
        }

        let metadata = fs::metadata(&full_path).await?;
        let file_name = full_path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        Ok(FileInfo {
            path: path.to_path_buf(),
            name: file_name.to_string(),
            extension: full_path.extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string()),
            size: metadata.len(),
            is_directory: metadata.is_dir(),
            created_at: metadata.created()
                .map(|time| Utc.timestamp_opt(time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64, 0).unwrap())
                .unwrap_or_else(|_| Utc::now()),
            modified_at: metadata.modified()
                .map(|time| Utc.timestamp_opt(time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64, 0).unwrap())
                .unwrap_or_else(|_| Utc::now()),
            is_hidden: file_name.starts_with('.'),
            git_status: None,
        })
    }
}
