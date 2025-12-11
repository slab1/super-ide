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

use crate::utils::file_manager::{FileManager, DirEntry};
use crate::utils::event_bus::EventBus;

// API State
#[derive(Clone)]
pub struct ApiState {
    pub ide: Arc<super::core::SuperIDE>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct LearningProfileRequest {
    pub name: Option<String>,
    pub learning_style: Option<String>,
    pub current_level: Option<String>,
    pub preferences: Option<LearningPreferencesRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearningPreferencesRequest {
    pub difficulty_preference: Option<f32>,
    pub hint_frequency: Option<String>,
    pub code_completion_level: Option<String>,
    pub visual_aids_enabled: Option<bool>,
    pub voice_enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearningProgressUpdate {
    pub concept_id: String,
    pub mastery_level: f32,
    pub time_spent: Option<u64>,
    pub attempts: Option<u32>,
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
pub fn create_api_router(app_state: super::ui::AppState) -> Router<super::ui::AppState> {
    Router::new()
        // File operations
        .route("/files/:path", get(load_file))
        .route("/files/:path", put(save_file))
        .route("/files/create", post(create_file))
        .route("/files/:path", delete(delete_file))
        .route("/files/tree", get(get_file_tree))
        .route("/files/search", get(search_files))
        
        // AI endpoints
        .route("/ai/chat", post(ai_chat))
        .route("/ai/completions", post(get_completions))
        .route("/ai/analyze", post(analyze_code))
        
        // Learning endpoints
        .route("/learning/profile", get(get_learning_profile))
        .route("/learning/profile", put(update_learning_profile))
        .route("/learning/paths", get(get_learning_paths))
        .route("/learning/modules/:path_id", get(get_learning_module))
        .route("/learning/concepts/:concept_id", get(get_concept))
        .route("/learning/progress", get(get_learning_progress))
        .route("/learning/progress", put(update_learning_progress))
        .route("/learning/tutor/chat", post(tutor_chat))
        .route("/learning/tour", post(create_code_tour))
        .route("/learning/achievements", get(get_achievements))
        
        // Git operations
        .route("/git/status", get(git_status))
        .route("/git/branches", get(git_branches))
        .route("/git/commit", post(git_commit))
        
        // Project operations
        .route("/project/info", get(project_info))
        .route("/project/config", get(get_config))
        .route("/health", get(health_check))

        // External integrations
        .route("/external/mcp/search_tweets", post(mcp_search_tweets))
        .route("/external/mcp/user_info", post(mcp_get_user_info))
        .route("/external/mcp/user_tweets", post(mcp_get_user_tweets))
        .route("/external/mcp/functions", get(mcp_get_functions))
        .route("/external/browser/navigate", post(browser_navigate))
        .route("/external/browser/screenshot", post(browser_screenshot))
        .route("/external/browser/execute_script", post(browser_execute_script))
        .route("/external/browser/click", post(browser_click))
        .route("/external/browser/type", post(browser_type))
        .route("/external/browser/wait", post(browser_wait))
        .route("/external/browser/page_info", get(browser_get_page_info))
        .route("/external/browser/element_info", post(browser_get_element_info))
        .route("/external/status", get(external_status))

        .with_state(app_state)
}

// File Operations Handlers

/// Load file content
pub async fn load_file(
    State(state): State<super::ui::AppState>,
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
pub async fn save_file(
    State(state): State<super::ui::AppState>,
    Path(path): Path<String>,
    Json(request): Json<FileContentRequest>,
) -> impl IntoResponse {
    let file_manager = state.file_manager.read().await;
    let path_buf = PathBuf::from(path);
    
    match file_manager.write_file(&path_buf, &request.content).await {
        Ok(_) => {
            info!("Successfully saved file: {}", path_buf.display());
            
            // Notify other components about file change
            let _ = state.event_bus.broadcast(crate::utils::event_bus::IdeEvent::FileChanged {
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
pub async fn create_file(
    State(state): State<super::ui::AppState>,
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
            let _ = state.event_bus.broadcast(crate::utils::event_bus::IdeEvent::FileChanged {
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
pub async fn delete_file(
    State(state): State<super::ui::AppState>,
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
            let _ = state.event_bus.broadcast(crate::utils::event_bus::IdeEvent::FileChanged {
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
pub async fn get_file_tree(State(state): State<super::ui::AppState>) -> impl IntoResponse {
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
pub async fn search_files(
    State(state): State<super::ui::AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let file_manager = state.file_manager.read().await;
    let pattern = params.get("pattern").unwrap_or(&"".to_string()).clone();
    let root = params.get("root").unwrap_or(&".".to_string()).clone();
    
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
pub async fn ai_chat(
    State(state): State<super::ui::AppState>,
    Json(request): Json<AIChatRequest>,
) -> impl IntoResponse {
    let ai_engine = state.ide.ai_engine();
    
    // Create AI completion request
    let completion_request = crate::ai::CompletionRequest {
        prompt: request.message,
        context: request.context.as_ref().and_then(|ctx| ctx.file_content.as_ref()).cloned().unwrap_or_default(),
        language: request.context.as_ref().and_then(|ctx| ctx.language.as_ref()).cloned().unwrap_or_else(|| "rust".to_string()),
        max_tokens: request.settings.and_then(|s| s.max_tokens),
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
pub async fn get_completions(
    State(state): State<super::ui::AppState>,
    Json(_request): Json<CodeCompletionRequest>,
) -> impl IntoResponse {
    let editor = state.ide.editor();
    let _editor_lock = editor.lock().await;
    
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
pub async fn analyze_code(
    State(state): State<super::ui::AppState>,
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
pub async fn git_status(
    State(state): State<super::ui::AppState>,
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
pub async fn git_branches(State(state): State<super::ui::AppState>) -> impl IntoResponse {
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
pub async fn git_commit(
    State(state): State<super::ui::AppState>,
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
pub async fn project_info(State(state): State<super::ui::AppState>) -> impl IntoResponse {
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
pub async fn get_config(State(state): State<super::ui::AppState>) -> impl IntoResponse {
    let config = state.ide.config().read().await;
    
    let config_info = ConfigInfo {
        workspace_path: config.workspace_dir().to_string_lossy().to_string(),
        auto_save_interval: config.ide.auto_save_interval,
        font_size: config.editor.font_size as u16,
        theme_name: config.theme.name.clone(),
        ai_provider: format!("{:?}", config.ai.provider),
        enable_ai: config.ai.enable_local_inference,
    };
    
    ApiResponse::success(config_info)
}

/// Health check
pub async fn health_check(State(state): State<super::ui::AppState>) -> impl IntoResponse {
    let ide_state = state.ide.get_state().await;
    
    ApiResponse::success(HealthStatus {
        status: "healthy".to_string(),
        ide_running: true,
        documents_open: ide_state.active_tabs.len(),
        ai_enabled: state.ide.ai_engine().ai_provider().await.unwrap_or_else(|_| "local".to_string()) != "local",
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

// External Integration Handlers

/// Search Twitter tweets via MCP
pub async fn mcp_search_tweets(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<crate::external::api::TwitterSearchRequest>,
) -> impl IntoResponse {
    // For now, we'll create a simple MCP client
    // In a real implementation, this would be managed by the IDE state
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::api::McpApiClient::new(config);

    match client.search_tweets(request).await {
        Ok(data) => {
            info!("MCP Twitter search completed successfully");
            ApiResponse::success(data)
        }
        Err(e) => {
            error!("MCP Twitter search failed: {}", e);
            ApiResponse::error(format!("Twitter search failed: {}", e))
        }
    }
}

/// Get Twitter user info via MCP
pub async fn mcp_get_user_info(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<crate::external::api::TwitterUserRequest>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::api::McpApiClient::new(config);

    match client.get_twitter_user_info(request).await {
        Ok(data) => {
            info!("MCP Twitter user info retrieved successfully");
            ApiResponse::success(data)
        }
        Err(e) => {
            error!("MCP Twitter user info failed: {}", e);
            ApiResponse::error(format!("Twitter user info failed: {}", e))
        }
    }
}

/// Get Twitter user tweets via MCP
pub async fn mcp_get_user_tweets(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<crate::external::api::TwitterUserTweetsRequest>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::api::McpApiClient::new(config);

    match client.get_twitter_user_tweets(request).await {
        Ok(data) => {
            info!("MCP Twitter user tweets retrieved successfully");
            ApiResponse::success(data)
        }
        Err(e) => {
            error!("MCP Twitter user tweets failed: {}", e);
            ApiResponse::error(format!("Twitter user tweets failed: {}", e))
        }
    }
}

/// Get available MCP functions
pub async fn mcp_get_functions(
    State(_state): State<super::ui::AppState>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::api::McpApiClient::new(config);

    match client.get_available_functions().await {
        Ok(functions) => {
            info!("Retrieved {} MCP functions", functions.len());
            ApiResponse::success(functions)
        }
        Err(e) => {
            error!("Failed to get MCP functions: {}", e);
            ApiResponse::error(format!("Failed to get MCP functions: {}", e))
        }
    }
}

/// Navigate browser
pub async fn browser_navigate(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<crate::external::browser::BrowserNavigateRequest>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::browser::BrowserClient::new(config);

    match client.navigate(request).await {
        Ok(page_info) => {
            info!("Browser navigation completed successfully");
            ApiResponse::success(page_info)
        }
        Err(e) => {
            error!("Browser navigation failed: {}", e);
            ApiResponse::error(format!("Browser navigation failed: {}", e))
        }
    }
}

/// Take browser screenshot
pub async fn browser_screenshot(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<crate::external::browser::BrowserScreenshotRequest>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::browser::BrowserClient::new(config);

    match client.screenshot(request).await {
        Ok(image_data) => {
            info!("Browser screenshot taken successfully");
            // Return base64 encoded image
            let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);
            ApiResponse::success(base64_data)
        }
        Err(e) => {
            error!("Browser screenshot failed: {}", e);
            ApiResponse::error(format!("Browser screenshot failed: {}", e))
        }
    }
}

/// Execute JavaScript in browser
pub async fn browser_execute_script(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<crate::external::browser::BrowserExecuteScriptRequest>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::browser::BrowserClient::new(config);

    match client.execute_script(request).await {
        Ok(result) => {
            info!("Browser script execution completed successfully");
            ApiResponse::success(result)
        }
        Err(e) => {
            error!("Browser script execution failed: {}", e);
            ApiResponse::error(format!("Browser script execution failed: {}", e))
        }
    }
}

/// Click element in browser
pub async fn browser_click(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<crate::external::browser::BrowserClickRequest>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::browser::BrowserClient::new(config);

    match client.click(request).await {
        Ok(_) => {
            info!("Browser click completed successfully");
            ApiResponse::success("Click completed")
        }
        Err(e) => {
            error!("Browser click failed: {}", e);
            ApiResponse::error(format!("Browser click failed: {}", e))
        }
    }
}

/// Type text in browser
pub async fn browser_type(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<crate::external::browser::BrowserTypeRequest>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::browser::BrowserClient::new(config);

    match client.type_text(request).await {
        Ok(_) => {
            info!("Browser text input completed successfully");
            ApiResponse::success("Text input completed")
        }
        Err(e) => {
            error!("Browser text input failed: {}", e);
            ApiResponse::error(format!("Browser text input failed: {}", e))
        }
    }
}

/// Wait for element in browser
pub async fn browser_wait(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<crate::external::browser::BrowserWaitRequest>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::browser::BrowserClient::new(config);

    match client.wait_for_element(request).await {
        Ok(element_info) => {
            info!("Browser element wait completed successfully");
            ApiResponse::success(element_info)
        }
        Err(e) => {
            error!("Browser element wait failed: {}", e);
            ApiResponse::error(format!("Browser element wait failed: {}", e))
        }
    }
}

/// Get browser page info
pub async fn browser_get_page_info(
    State(_state): State<super::ui::AppState>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let client = crate::external::browser::BrowserClient::new(config);

    match client.get_page_info().await {
        Ok(page_info) => {
            info!("Browser page info retrieved successfully");
            ApiResponse::success(page_info)
        }
        Err(e) => {
            error!("Browser page info retrieval failed: {}", e);
            ApiResponse::error(format!("Browser page info retrieval failed: {}", e))
        }
    }
}

/// Get browser element info
pub async fn browser_get_element_info(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let selector = request.get("selector")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if selector.is_empty() {
        return ApiResponse::error("Selector is required".to_string());
    }

    let config = crate::external::ExternalConfig::default();
    let client = crate::external::browser::BrowserClient::new(config);

    match client.get_element_info(selector).await {
        Ok(element_info) => {
            info!("Browser element info retrieved successfully");
            ApiResponse::success(element_info)
        }
        Err(e) => {
            error!("Browser element info retrieval failed: {}", e);
            ApiResponse::error(format!("Browser element info retrieval failed: {}", e))
        }
    }
}

/// Get external integrations status
pub async fn external_status(
    State(_state): State<super::ui::AppState>,
) -> impl IntoResponse {
    let config = crate::external::ExternalConfig::default();
    let browser_client = crate::external::browser::BrowserClient::new(config.clone());

    let browser_available = browser_client.is_browser_available().await;

    // For MCP status, we could check if the server is running
    // For now, we'll just return basic status
    let status = serde_json::json!({
        "mcp_server_port": config.mcp_server_port,
        "browser_debug_port": config.browser_debug_port,
        "browser_available": browser_available,
        "external_api_path": config.external_api_path,
        "browser_path": config.browser_path,
    });

    info!("External integrations status retrieved");
    ApiResponse::success(status)
}

// Learning Handlers

/// Get student learning profile
pub async fn get_learning_profile(
    State(_state): State<super::ui::AppState>,
) -> impl IntoResponse {
    // Mock learning profile - in real implementation, this would load from database
    let profile = crate::learning::StudentProfile {
        id: "student-1".to_string(),
        name: "Demo Student".to_string(),
        learning_style: crate::learning::LearningStyle::Visual,
        current_level: crate::learning::SkillLevel::Beginner,
        progress: std::collections::HashMap::new(),
        preferences: crate::learning::StudentPreferences {
            difficulty_preference: 0.5,
            hint_frequency: crate::learning::HintFrequency::AfterStruggle,
            code_completion_level: crate::learning::CodeCompletionLevel::Smart,
            visual_aids_enabled: true,
            voice_enabled: false,
        },
        achievements: Vec::new(),
    };
    
    ApiResponse::success(profile)
}

/// Update student learning profile
pub async fn update_learning_profile(
    State(_state): State<super::ui::AppState>,
    Json(profile): Json<crate::learning::StudentProfile>,
) -> impl IntoResponse {
    info!("Learning profile updated for student: {}", profile.id);
    
    // In real implementation, save to database
    ApiResponse::success("Profile updated successfully")
}

/// Get available learning paths
pub async fn get_learning_paths(
    State(_state): State<super::ui::AppState>,
) -> impl IntoResponse {
    let paths = vec![
        crate::learning::LearningPath {
            id: "python-fundamentals".to_string(),
            title: "Python Fundamentals".to_string(),
            description: "Learn Python programming from basics to advanced concepts".to_string(),
            modules: vec!["variables".to_string(), "functions".to_string(), "oop".to_string()],
            estimated_total_duration: std::time::Duration::from_secs(7200),
            target_audience: "Beginners".to_string(),
            outcomes: vec!["Write Python programs".to_string(), "Understand OOP".to_string()],
        },
        crate::learning::LearningPath {
            id: "rust-systems".to_string(),
            title: "Rust Systems Programming".to_string(),
            description: "Master systems programming with Rust".to_string(),
            modules: vec!["ownership".to_string(), "borrowing".to_string(), "lifetimes".to_string()],
            estimated_total_duration: std::time::Duration::from_secs(10800),
            target_audience: "Intermediate".to_string(),
            outcomes: vec!["Memory-safe code".to_string(), "Systems programming".to_string()],
        },
    ];
    
    ApiResponse::success(paths)
}

/// Get learning module details
pub async fn get_learning_module(
    State(_state): State<super::ui::AppState>,
    Path(path_id): Path<String>,
) -> impl IntoResponse {
    let module = crate::learning::LearningModule {
        id: path_id,
        title: "Sample Module".to_string(),
        description: "Learn programming fundamentals".to_string(),
        concepts: vec![
            crate::learning::Concept {
                id: "concept-1".to_string(),
                name: "Variables and Data Types".to_string(),
                explanation: "Learn about variables, data types, and how to store information in your programs.".to_string(),
                code_examples: Vec::new(),
                visual_aids: Vec::new(),
                interactive_demos: Vec::new(),
            },
            crate::learning::Concept {
                id: "concept-2".to_string(),
                name: "Functions".to_string(),
                explanation: "Understand how to create reusable code blocks with functions.".to_string(),
                code_examples: Vec::new(),
                visual_aids: Vec::new(),
                interactive_demos: Vec::new(),
            },
        ],
        exercises: Vec::new(),
        estimated_duration: std::time::Duration::from_secs(1800),
        prerequisites: Vec::new(),
    };
    
    ApiResponse::success(module)
}

/// Get concept details
pub async fn get_concept(
    State(_state): State<super::ui::AppState>,
    Path(concept_id): Path<String>,
) -> impl IntoResponse {
    let concept = crate::learning::Concept {
        id: concept_id,
        name: "Sample Concept".to_string(),
        explanation: "This is a sample concept explanation.".to_string(),
        code_examples: Vec::new(),
        visual_aids: Vec::new(),
        interactive_demos: Vec::new(),
    };
    
    ApiResponse::success(concept)
}

/// Get learning progress
pub async fn get_learning_progress(
    State(_state): State<super::ui::AppState>,
) -> impl IntoResponse {
    let progress = crate::learning::LearningAnalytics {
        student_id: "student-1".to_string(),
        session_data: Vec::new(),
        concept_mastery: std::collections::HashMap::new(),
        learning_velocity: 0.0,
        struggle_patterns: Vec::new(),
        recommended_next_concepts: Vec::new(),
    };
    
    ApiResponse::success(progress)
}

/// Update learning progress
pub async fn update_learning_progress(
    State(_state): State<super::ui::AppState>,
    Json(progress): Json<std::collections::HashMap<String, crate::learning::ProgressMetrics>>,
) -> impl IntoResponse {
    info!("Learning progress updated for {} concepts", progress.len());
    
    // In real implementation, save to database
    ApiResponse::success("Progress updated successfully")
}

/// AI Tutor chat
pub async fn tutor_chat(
    State(state): State<super::ui::AppState>,
    Json(request): Json<AIChatRequest>,
) -> impl IntoResponse {
    let ai_engine = state.ide.ai_engine();
    
    // Create AI completion request with learning context
    let completion_request = crate::ai::CompletionRequest {
        prompt: format!("[LEARNING TUTOR] {}", request.message),
        context: request.context.as_ref().and_then(|ctx| ctx.file_content.as_ref()).cloned().unwrap_or_default(),
        language: request.context.as_ref().and_then(|ctx| ctx.language.as_ref()).cloned().unwrap_or_else(|| "rust".to_string()),
        max_tokens: request.settings.and_then(|s| s.max_tokens),
    };
    
    match ai_engine.generate_completion(completion_request).await {
        Ok(completion) => {
            info!("AI tutor chat completed successfully");
            ApiResponse::success(completion.text)
        }
        Err(e) => {
            error!("AI tutor chat failed: {}", e);
            ApiResponse::error(format!("AI tutor chat failed: {}", e))
        }
    }
}

/// Create code tour for a file
pub async fn create_code_tour(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let file_path = request.get("file_path")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    if file_path.is_empty() {
        return ApiResponse::error("File path is required".to_string());
    }
    
    let tour = crate::learning::CodeTour {
        id: format!("tour_{}", file_path),
        file_path: std::path::PathBuf::from(file_path),
        title: format!("Understanding {}", std::path::Path::new(file_path).file_name().unwrap_or_default().to_string_lossy()),
        description: "Interactive code walkthrough with explanations".to_string(),
        steps: vec![
            crate::learning::CodeTourStep {
                step_number: 1,
                line_range: (1, 10),
                title: "File Overview".to_string(),
                explanation: "This file contains the main program logic. Let's explore it step by step.".to_string(),
                highlighted_concepts: vec!["file_structure".to_string()],
                visual_aids: Vec::new(),
                interactive_elements: Vec::new(),
            }
        ],
        prerequisites: Vec::new(),
        estimated_duration: std::time::Duration::from_secs(300),
    };
    
    ApiResponse::success(tour)
}

/// Get learning achievements
pub async fn get_achievements(
    State(_state): State<super::ui::AppState>,
) -> impl IntoResponse {
    let achievements = vec![
        crate::learning::Achievement {
            id: "first-function".to_string(),
            title: "First Function".to_string(),
            description: "Created your first function".to_string(),
            icon: "🎯".to_string(),
            earned_at: chrono::Utc::now(),
            category: crate::learning::AchievementCategory::FirstSteps,
        },
        crate::learning::Achievement {
            id: "debug-master".to_string(),
            title: "Debug Master".to_string(),
            description: "Fixed 10 bugs on your own".to_string(),
            icon: "🐛".to_string(),
            earned_at: chrono::Utc::now(),
            category: crate::learning::AchievementCategory::Debugging,
        },
    ];
    
    ApiResponse::success(achievements)
}
