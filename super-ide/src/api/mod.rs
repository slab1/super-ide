//! RESTful API endpoints for Super IDE
//!
//! This module provides all the API endpoints that the frontend expects:
//! - File operations (/api/files/*)
//! - AI integration (/api/ai/*)
//! - Git operations (/api/git/*)
//! - Project management (/api/project/*)

use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use log::{info, warn, error};
use chrono::Utc;

use crate::core::SuperIDE;
use crate::utils::file_manager::{FileManager, FileManagerError, DirEntry};
use crate::utils::event_bus::EventBus;

// API State
#[derive(Clone)]
pub struct ApiState {
    pub ide: Arc<SuperIDE>,
    pub file_manager: Arc<RwLock<FileManager>>,
    pub event_bus: Arc<EventBus>,
}

// Request/Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContentRequest {
    pub content: String,
    pub encoding: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileCreateRequest {
    pub path: String,
    pub content: Option<String>,
    pub is_directory: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIChatRequest {
    pub message: String,
    pub context: Option<AIContext>,
    pub settings: Option<AISettings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIContext {
    pub file_path: Option<String>,
    pub file_content: Option<String>,
    pub language: Option<String>,
    pub cursor_position: Option<(usize, usize)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AISettings {
    pub provider: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeCompletionRequest {
    pub code: String,
    pub position: usize,
    pub language: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitStatusRequest {
    pub path: Option<String>,
}

// API Response helpers
impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        if self.success {
            (StatusCode::OK, Json(self)).into_response()
        } else {
            (StatusCode::BAD_REQUEST, Json(self)).into_response()
        }
    }
}

// Router creation
pub fn create_api_router(state: ApiState) -> Router {
    Router::new()
        // File operations
        .route("/api/files/:path", get(load_file))
        .route("/api/files/:path", put(save_file))
        .route("/api/files/create", post(create_file))
        .route("/api/files/:path", delete(delete_file))
        .route("/api/files/tree", get(get_file_tree))
        .route("/api/files/search", get(search_files))
        
        // AI endpoints
        .route("/api/ai/chat", post(ai_chat))
        .route("/api/ai/completions", post(get_completions))
        .route("/api/ai/analyze", post(analyze_code))
        
        // Git operations
        .route("/api/git/status", get(git_status))
        .route("/api/git/branches", get(git_branches))
        .route("/api/git/commit", post(git_commit))
        
        // Project operations
        .route("/api/project/info", get(project_info))
        .route("/api/project/config", get(get_config))
        .route("/api/health", get(health_check))
        
        .with_state(state)
}

// File Operations Handlers

/// Load file content
async fn load_file(
    State(state): State<ApiState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let file_manager = state.file_manager.read().await;
    let path_buf = PathBuf::from(path);
    
    match file_manager.read_file(&path_buf).await {
        Ok(content) => {
            info!("Successfully loaded file: {}", path_buf.display());
            ApiResponse::success(content)
        }
        Err(e) => {
            warn!("Failed to load file {}: {}", path_buf.display(), e);
            ApiResponse::error(format!("Failed to load file: {}", e))
        }
    }
}

/// Save file content
async fn save_file(
    State(state): State<ApiState>,
    Path(path): Path<String>,
    Json(request): Json<FileContentRequest>,
) -> impl IntoResponse {
    let file_manager = state.file_manager.read().await;
    let path_buf = PathBuf::from(path);
    
    match file_manager.write_file(&path_buf, &request.content).await {
        Ok(_) => {
            info!("Successfully saved file: {}", path_buf.display());
            
            // Notify other components about file change
            let _ = state.event_bus.send(crate::utils::event_bus::IdeEvent::FileChanged {
                path: path_buf.to_string_lossy().to_string(),
                event_type: crate::utils::event_bus::FileEventType::Modified,
            });
            
            ApiResponse::success("File saved successfully")
        }
        Err(e) => {
            error!("Failed to save file {}: {}", path_buf.display(), e);
            ApiResponse::error(format!("Failed to save file: {}", e))
        }
    }
}

/// Create new file or directory
async fn create_file(
    State(state): State<ApiState>,
    Json(request): Json<FileCreateRequest>,
) -> impl IntoResponse {
    let file_manager = state.file_manager.read().await;
    let path_buf = PathBuf::from(&request.path);
    
    match if request.is_directory {
        file_manager.create_dir(&path_buf).await
    } else {
        file_manager.create_file(&path_buf, request.content.as_deref()).await
    } {
        Ok(_) => {
            info!("Successfully created {}: {}", 
                if request.is_directory { "directory" } else { "file" }, 
                path_buf.display());
                
            // Notify about file creation
            let _ = state.event_bus.send(crate::utils::event_bus::IdeEvent::FileChanged {
                path: path_buf.to_string_lossy().to_string(),
                event_type: crate::utils::event_bus::FileEventType::Created,
            });
            
            ApiResponse::success("Created successfully")
        }
        Err(e) => {
            error!("Failed to create {}: {}", path_buf.display(), e);
            ApiResponse::error(format!("Failed to create: {}", e))
        }
    }
}

/// Delete file or directory
async fn delete_file(
    State(state): State<ApiState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let file_manager = state.file_manager.read().await;
    let path_buf = PathBuf::from(path);
    
    match if path_buf.is_dir() {
        file_manager.remove_dir(&path_buf).await
    } else {
        file_manager.delete_file(&path_buf).await
    } {
        Ok(_) => {
            info!("Successfully deleted: {}", path_buf.display());
            
            // Notify about file deletion
            let _ = state.event_bus.send(crate::utils::event_bus::IdeEvent::FileChanged {
                path: path_buf.to_string_lossy().to_string(),
                event_type: crate::utils::event_bus::FileEventType::Deleted,
            });
            
            ApiResponse::success("Deleted successfully")
        }
        Err(e) => {
            error!("Failed to delete {}: {}", path_buf.display(), e);
            ApiResponse::error(format!("Failed to delete: {}", e))
        }
    }
}

/// Get file tree structure
async fn get_file_tree(State(state): State<ApiState>) -> impl IntoResponse {
    let file_manager = state.file_manager.read().await;
    let workspace_path = state.ide.config().read().await.workspace_dir();
    
    match file_manager.list_dir(&workspace_path).await {
        Ok(entries) => {
            let file_tree: Vec<FileTreeNode> = entries.into_iter()
                .map(|entry| FileTreeNode::from(entry))
                .collect();
            
            info!("Successfully loaded file tree with {} items", file_tree.len());
            ApiResponse::success(file_tree)
        }
        Err(e) => {
            error!("Failed to load file tree: {}", e);
            ApiResponse::error(format!("Failed to load file tree: {}", e))
        }
    }
}

/// Search files by pattern
async fn search_files(
    State(state): State<ApiState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let file_manager = state.file_manager.read().await;
    let pattern = params.get("pattern").unwrap_or(&"".to_string()).clone();
    let root = params.get("root").unwrap_or(&".".to_string());
    
    if pattern.is_empty() {
        return ApiResponse::error("Search pattern is required".to_string());
    }
    
    match file_manager.search_files(&PathBuf::from(root), &pattern) {
        Ok(paths) => {
            let search_results: Vec<SearchResult> = paths.into_iter()
                .map(|path| SearchResult {
                    path: path.to_string_lossy().to_string(),
                    name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    size: path.metadata().map(|m| m.len()).unwrap_or(0),
                })
                .collect();
            
            info!("Found {} files matching pattern '{}'", search_results.len(), pattern);
            ApiResponse::success(search_results)
        }
        Err(e) => {
            error!("Search failed: {}", e);
            ApiResponse::error(format!("Search failed: {}", e))
        }
    }
}

// AI Handlers

/// AI chat endpoint
async fn ai_chat(
    State(state): State<ApiState>,
    Json(request): Json<AIChatRequest>,
) -> impl IntoResponse {
    let ai_engine = state.ide.ai_engine();
    
    // Create AI completion request
    let completion_request = crate::ai::CompletionRequest {
        prompt: request.message,
        context: request.context.map(|ctx| ctx.file_content.unwrap_or_default()).unwrap_or_default(),
        language: request.context.and_then(|ctx| ctx.language).unwrap_or_else(|| "rust".to_string()),
        max_tokens: request.settings.and_then(|s| s.max_tokens.map(|t| t as usize)),
    };
    
    match ai_engine.generate_completion(completion_request).await {
        Ok(completion) => {
            info!("AI chat completed successfully");
            ApiResponse::success(completion.text)
        }
        Err(e) => {
            error!("AI chat failed: {}", e);
            ApiResponse::error(format!("AI chat failed: {}", e))
        }
    }
}

/// Get code completions
async fn get_completions(
    State(state): State<ApiState>,
    Json(request): Json<CodeCompletionRequest>,
) -> impl IntoResponse {
    let editor = state.ide.editor();
    let editor_lock = editor.lock().await;
    
    // For now, return a simple completion response
    // In a real implementation, this would use the editor's completion system
    let completions = vec![
        crate::editor::CompletionItem {
            label: "completion_placeholder".to_string(),
            kind: crate::editor::CompletionKind::Keyword,
            detail: Some("AI-generated completion".to_string()),
            documentation: None,
            insert_text: "completion_text".to_string(),
            sort_text: "completion_text".to_string(),
        }
    ];
    
    ApiResponse::success(completions)
}

/// Analyze code using AI
async fn analyze_code(
    State(state): State<ApiState>,
    Json(request): Json<CodeCompletionRequest>,
) -> impl IntoResponse {
    let ai_engine = state.ide.ai_engine();
    
    match ai_engine.analyze_code(&request.code, &request.language).await {
        Ok(analysis) => {
            info!("Code analysis completed");
            ApiResponse::success(analysis)
        }
        Err(e) => {
            error!("Code analysis failed: {}", e);
            ApiResponse::error(format!("Code analysis failed: {}", e))
        }
    }
}

// Git Handlers

/// Get git status
async fn git_status(
    State(state): State<ApiState>,
    Query(params): Query<GitStatusRequest>,
) -> impl IntoResponse {
    let file_manager = state.file_manager.read().await;
    let workspace_path = state.ide.config().read().await.workspace_dir();
    
    let path = params.path.as_ref()
        .map(|p| PathBuf::from(p))
        .unwrap_or(workspace_path);
    
    match file_manager.get_git_status(&path).await {
        Ok(status) => {
            info!("Git status retrieved for: {}", path.display());
            ApiResponse::success(status.to_string())
        }
        Err(e) => {
            error!("Git status failed: {}", e);
            ApiResponse::error(format!("Git status failed: {}", e))
        }
    }
}

/// Get git branches
async fn git_branches(State(state): State<ApiState>) -> impl IntoResponse {
    let workspace_path = state.ide.config().read().await.workspace_dir();
    
    match tokio::process::Command::new("git")
        .args(&["branch", "-a", "--format=%(refname:short)%09%(objectname)%09%(committerdate:relative)%09%(subject)"])
        .current_dir(&workspace_path)
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                let branches: Vec<GitBranch> = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| {
                        let parts: Vec<&str> = line.split('\t').collect();
                        GitBranch {
                            name: parts.get(0).unwrap_or(&"").to_string(),
                            commit: parts.get(1).unwrap_or(&"").to_string(),
                            date: parts.get(2).unwrap_or(&"").to_string(),
                            message: parts.get(3).unwrap_or(&"").to_string(),
                        }
                    })
                    .collect();
                
                info!("Retrieved {} git branches", branches.len());
                ApiResponse::success(branches)
            } else {
                ApiResponse::error("Failed to get git branches".to_string())
            }
        }
        Err(e) => {
            error!("Git branches failed: {}", e);
            ApiResponse::error(format!("Git branches failed: {}", e))
        }
    }
}

/// Commit changes
async fn git_commit(
    State(state): State<ApiState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let workspace_path = state.ide.config().read().await.workspace_dir();
    
    let message = request.get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("Commit from Super IDE");
    
    match tokio::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(&workspace_path)
        .output()
        .await
    {
        Ok(_) => {
            match tokio::process::Command::new("git")
                .args(&["commit", "-m", message])
                .current_dir(&workspace_path)
                .output()
                .await
            {
                Ok(output) => {
                    if output.status.success() {
                        info!("Git commit successful: {}", message);
                        ApiResponse::success("Commit successful")
                    } else {
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        ApiResponse::error(format!("Commit failed: {}", error_msg))
                    }
                }
                Err(e) => {
                    error!("Git commit failed: {}", e);
                    ApiResponse::error(format!("Git commit failed: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Git add failed: {}", e);
            ApiResponse::error(format!("Git add failed: {}", e))
        }
    }
}

// Project Handlers

/// Get project information
async fn project_info(State(state): State<ApiState>) -> impl IntoResponse {
    let config = state.ide.config().read().await;
    let workspace_path = config.workspace_dir();
    
    let project_info = ProjectInfo {
        name: workspace_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        path: workspace_path.to_string_lossy().to_string(),
        workspace_path: workspace_path.to_string_lossy().to_string(),
        ai_enabled: config.ai.enable_local_inference,
        ai_provider: format!("{:?}", config.ai.provider),
        last_opened: Utc::now().to_rfc3339(),
    };
    
    ApiResponse::success(project_info)
}

/// Get configuration
async fn get_config(State(state): State<ApiState>) -> impl IntoResponse {
    let config = state.ide.config().read().await;
    
    let config_info = ConfigInfo {
        workspace_path: config.workspace_dir().to_string_lossy().to_string(),
        auto_save_interval: config.ide.auto_save_interval,
        font_size: config.editor.font_size,
        theme_name: config.theme.name.clone(),
        ai_provider: format!("{:?}", config.ai.provider),
        enable_ai: config.ai.enable_local_inference,
    };
    
    ApiResponse::success(config_info)
}

/// Health check
async fn health_check(State(state): State<ApiState>) -> impl IntoResponse {
    let ide_state = state.ide.get_state().await;
    
    ApiResponse::success(HealthStatus {
        status: "healthy".to_string(),
        ide_running: true,
        documents_open: ide_state.active_tabs.len(),
        ai_enabled: state.ide.ai_engine().is_available().await,
        timestamp: Utc::now().to_rfc3339(),
    })
}

// Supporting types

#[derive(Debug, Serialize, Deserialize)]
pub struct FileTreeNode {
    pub name: String,
    pub path: String,
    pub r#type: String, // "file" or "directory"
    pub size: u64,
    pub modified: String,
    pub children: Option<Vec<FileTreeNode>>,
}

impl From<DirEntry> for FileTreeNode {
    fn from(entry: DirEntry) -> Self {
        Self {
            name: entry.name,
            path: entry.path.to_string_lossy().to_string(),
            r#type: if entry.is_file { "file".to_string() } else { "directory".to_string() },
            size: entry.size,
            modified: entry.modified.to_rfc3339(),
            children: None, // Will be populated recursively for directories
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub commit: String,
    pub date: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub workspace_path: String,
    pub ai_enabled: bool,
    pub ai_provider: String,
    pub last_opened: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub workspace_path: String,
    pub auto_save_interval: u64,
    pub font_size: u16,
    pub theme_name: String,
    pub ai_provider: String,
    pub enable_ai: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub ide_running: bool,
    pub documents_open: usize,
    pub ai_enabled: bool,
    pub timestamp: String,
}