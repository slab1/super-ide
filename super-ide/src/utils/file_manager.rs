//! File management utilities for Super IDE

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use thiserror::Error;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, Event, EventKind};
use tokio::sync::mpsc;
use futures::StreamExt;

/// File management errors
#[derive(Error, Debug)]
pub enum FileManagerError {
    #[error("File not found: {0}")]
    NotFound(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Watch error: {0}")]
    Watch(#[from] notify::Error),
}

/// File change event
#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed(PathBuf, PathBuf),
}

/// File change listener
#[derive(Debug)]
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    event_sender: mpsc::UnboundedSender<FileEvent>,
}

impl FileWatcher {
    pub fn new<P: AsRef<Path>>(
        path: P,
        mut event_sender: mpsc::UnboundedSender<FileEvent>,
    ) -> Result<Self, FileManagerError> {
        let event_sender_clone = event_sender.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, _>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) => {
                            for path in event.paths {
                                let _ = event_sender_clone.send(FileEvent::Created(path));
                            }
                        }
                        EventKind::Modify(_) => {
                            for path in event.paths {
                                let _ = event_sender_clone.send(FileEvent::Modified(path));
                            }
                        }
                        EventKind::Remove(_) => {
                            for path in event.paths {
                                let _ = event_sender_clone.send(FileEvent::Deleted(path));
                            }
                        }
                        EventKind::Any | EventKind::Access(_) => {
                            // Handle these patterns that were missing
                            // These are informational events, no action needed for now
                        }
                        EventKind::Other => {}
                    }
                }
            },
            notify::Config::default(),
        )?;
        
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
        
        Ok(Self {
            watcher,
            event_sender,
        })
    }
}

/// Main file manager
#[derive(Debug)]
pub struct FileManager {
    watchers: Vec<FileWatcher>,
}

impl FileManager {
    pub async fn new() -> Result<Self, FileManagerError> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        // Start background task to handle file events
        tokio::spawn(async move {
            let mut receiver = event_receiver;
            while let Some(event) = receiver.recv().await {
                match event {
                    FileEvent::Created(path) => {
                        println!("📁 File created: {}", path.display());
                    }
                    FileEvent::Modified(path) => {
                        println!("✏️ File modified: {}", path.display());
                    }
                    FileEvent::Deleted(path) => {
                        println!("🗑️ File deleted: {}", path.display());
                    }
                    FileEvent::Renamed(from, to) => {
                        println!("📝 File renamed: {} → {}", from.display(), to.display());
                    }
                }
            }
        });
        
        Ok(Self {
            watchers: Vec::new(),
        })
    }
    
    /// Read file content
    pub async fn read_file(&self, path: &Path) -> Result<String, FileManagerError> {
        fs::read_to_string(path).map_err(|e| FileManagerError::Io(e))
    }
    
    /// Write file content
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<(), FileManagerError> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, content).map_err(|e| FileManagerError::Io(e))
    }
    
    /// Create a new file
    pub async fn create_file(&self, path: &Path, content: Option<&str>) -> Result<(), FileManagerError> {
        let file_content = content.unwrap_or("");
        self.write_file(path, file_content).await
    }
    
    /// Delete a file
    pub async fn delete_file(&self, path: &Path) -> Result<(), FileManagerError> {
        fs::remove_file(path).map_err(|e| FileManagerError::Io(e))
    }
    
    /// Copy a file
    pub async fn copy_file(&self, from: &Path, to: &Path) -> Result<(), FileManagerError> {
        fs::copy(from, to).map(|_| ()).map_err(|e| FileManagerError::Io(e))
    }
    
    /// Move/rename a file
    pub async fn move_file(&self, from: &Path, to: &Path) -> Result<(), FileManagerError> {
        fs::rename(from, to).map_err(|e| FileManagerError::Io(e))
    }
    
    /// Check if file exists
    pub async fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    
    /// Get file metadata
    pub async fn metadata(&self, path: &Path) -> Result<fs::Metadata, FileManagerError> {
        fs::metadata(path).map_err(|e| FileManagerError::Io(e))
    }
    
    /// List directory contents
    pub async fn list_dir(&self, path: &Path) -> Result<Vec<DirEntry>, FileManagerError> {
        let entries = fs::read_dir(path)?;
        let mut dir_entries = Vec::new();
        
        for entry in entries {
            let entry = entry.map_err(FileManagerError::Io)?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            
            dir_entries.push(DirEntry {
                path,
                name: entry.file_name().to_string_lossy().to_string(),
                is_file: metadata.is_file(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: metadata.modified()
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            });
        }
        
        Ok(dir_entries)
    }
    
    /// Create a directory
    pub async fn create_dir(&self, path: &Path) -> Result<(), FileManagerError> {
        fs::create_dir_all(path).map_err(|e| FileManagerError::Io(e))
    }
    
    /// Remove a directory
    pub async fn remove_dir(&self, path: &Path) -> Result<(), FileManagerError> {
        fs::remove_dir_all(path).map_err(|e| FileManagerError::Io(e))
    }
    
    /// Search for files matching a pattern
    pub fn search_files(&self, root: &Path, pattern: &str) -> Result<Vec<PathBuf>, FileManagerError> {
        let mut results = Vec::new();
        
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    
                    if name.contains(pattern) {
                        if entry.metadata()?.is_file() {
                            results.push(path.clone());
                        }
                    }
                    
                    // Recursively search subdirectories
                    if path.is_dir() {
                        let sub_results = self.search_files(&path, pattern)?;
                        results.extend(sub_results);
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Watch a directory for changes
    pub async fn watch_directory(&mut self, path: &Path) -> Result<(), FileManagerError> {
        let (event_sender, _) = mpsc::unbounded_channel();
        let watcher = FileWatcher::new(path, event_sender)?;
        
        self.watchers.push(watcher);
        println!("👁️ Watching directory: {}", path.display());
        
        Ok(())
    }
    
    /// Unwatch a directory
    pub async fn unwatch_directory(&mut self, path: &Path) -> Result<(), FileManagerError> {
        self.watchers.retain(|watcher| {
            // This would need to track which directories are being watched
            // For now, we'll implement a basic version
            true
        });
        
        println!("👁️ Stopped watching directory: {}", path.display());
        Ok(())
    }
    
    /// Get file hash for change detection
    pub async fn get_file_hash(&self, path: &Path) -> Result<String, FileManagerError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let content = self.read_file(path).await?;
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        
        Ok(format!("{:x}", hasher.finish()))
    }
    
    /// Check if file is git tracked
    pub async fn is_git_tracked(&self, path: &Path) -> bool {
        if let Ok(output) = tokio::process::Command::new("git")
            .args(&["ls-files", "--error-unmatch", path.to_string_lossy().as_ref()])
            .output()
            .await
        {
            output.status.success()
        } else {
            false
        }
    }
    
    /// Get git status for a file
    pub async fn get_git_status(&self, path: &Path) -> Result<GitStatus, FileManagerError> {
        if let Ok(output) = tokio::process::Command::new("git")
            .args(&["status", "--porcelain", path.to_string_lossy().as_ref()])
            .current_dir(path.parent().unwrap_or(path))
            .output()
            .await
        {
            let status_str = String::from_utf8_lossy(&output.stdout);
            
            if status_str.trim().is_empty() {
                Ok(GitStatus::Unmodified)
            } else {
                let status_char = status_str.chars().next().unwrap_or(' ');
                match status_char {
                    'M' => Ok(GitStatus::Modified),
                    'A' => Ok(GitStatus::Added),
                    'D' => Ok(GitStatus::Deleted),
                    'R' => Ok(GitStatus::Renamed),
                    'C' => Ok(GitStatus::Copied),
                    '?' => Ok(GitStatus::Untracked),
                    _ => Ok(GitStatus::Unknown),
                }
            }
        } else {
            Ok(GitStatus::NotInRepo)
        }
    }
}

/// Directory entry information
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_file: bool,
    pub is_dir: bool,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
}

/// Git status for a file
#[derive(Debug, Clone)]
pub enum GitStatus {
    Unmodified,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Unknown,
    NotInRepo,
}

impl std::fmt::Display for GitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitStatus::Unmodified => write!(f, "Unmodified"),
            GitStatus::Modified => write!(f, "Modified"),
            GitStatus::Added => write!(f, "Added"),
            GitStatus::Deleted => write!(f, "Deleted"),
            GitStatus::Renamed => write!(f, "Renamed"),
            GitStatus::Copied => write!(f, "Copied"),
            GitStatus::Untracked => write!(f, "Untracked"),
            GitStatus::Unknown => write!(f, "Unknown"),
            GitStatus::NotInRepo => write!(f, "Not in repository"),
        }
    }
}