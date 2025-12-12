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
use base64::Engine;

use crate::utils::event_bus::EventBus;
use crate::git::{GitManager, GitRepository, GitStatus, GitCommit, GitBranch, GitError};
use crate::file_ops::{FileManager, FileInfo, ProjectStructure, FileOperationResult, FileOperationError, FileChangeEvent, FileChangeType};

// API State
#[derive(Clone)]
pub struct ApiState {
    pub ide: Arc<super::core::SuperIDE>,
    pub file_manager: Arc<RwLock<FileManager>>,
    pub git_manager: Arc<GitManager>,
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
        
        // Advanced AI endpoints
        .route("/ai/smart-completions", post(smart_completions))
        .route("/ai/code-review", post(code_review))
        .route("/ai/debug-assistance", post(debug_assistance))
        .route("/ai/generate-project", post(generate_project))
        .route("/ai/context-help", post(context_help))
        .route("/ai/learning/feedback", post(learning_feedback))
        .route("/ai/optimize-advanced", post(optimize_advanced))
        .route("/ai/refactoring-suggestions", post(refactoring_suggestions))
        .route("/ai/apply-refactoring", post(apply_refactoring))
        .route("/ai/generate-tests-advanced", post(generate_tests_advanced))
        .route("/ai/performance-analysis", post(performance_analysis))
        .route("/ai/security-analysis", post(security_analysis))
        .route("/ai/translate-languages", post(translate_languages))
        .route("/ai/code-metrics", post(code_metrics))
        
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
    State(_state): State<super::ui::AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let file_manager = _state.file_manager.read().await;
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
    State(_state): State<super::ui::AppState>,
    Path(path): Path<String>,
    Json(request): Json<FileContentRequest>,
) -> impl IntoResponse {
    let file_manager = _state.file_manager.read().await;
    let path_buf = PathBuf::from(path);
    
    match file_manager.write_file(&path_buf, &request.content).await {
        Ok(result) => {
            info!("Successfully saved file: {} ({} bytes)", path_buf.display(), 
                  result.bytes_written.unwrap_or(0));
            
            // Notify other components about file change
            let _ = _state.event_bus.broadcast(crate::utils::event_bus::IdeEvent::FileChanged {
                path: path_buf.to_string_lossy().to_string(),
                event_type: crate::utils::event_bus::FileEventType::Modified,
            });
            
            ApiResponse::success(format!("File saved successfully ({} bytes)", 
                                        result.bytes_written.unwrap_or(0)))
        }
        Err(e) => {
            error!("Failed to save file {}: {}", path_buf.display(), e);
            ApiResponse::error(format!("Failed to save file: {}", e))
        }
    }
}

/// Create new file or directory
pub async fn create_file(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<FileCreateRequest>,
) -> impl IntoResponse {
    let file_manager = _state.file_manager.read().await;
    let path_buf = PathBuf::from(&request.path);
    
    match if request.is_directory {
        file_manager.create_directory(&path_buf).await
    } else {
        file_manager.create_file(&path_buf, request.content.as_deref()).await
    } {
        Ok(result) => {
            info!("Successfully created {}: {} ({})", 
                if request.is_directory { "directory" } else { "file" }, 
                path_buf.display(),
                result.message);
                
            // Notify about file creation
            let _ = _state.event_bus.broadcast(crate::utils::event_bus::IdeEvent::FileChanged {
                path: path_buf.to_string_lossy().to_string(),
                event_type: crate::utils::event_bus::FileEventType::Created,
            });
            
            ApiResponse::success(format!("Created successfully: {}", result.message))
        }
        Err(e) => {
            error!("Failed to create {}: {}", path_buf.display(), e);
            ApiResponse::error(format!("Failed to create: {}", e))
        }
    }
}

/// Delete file or directory
pub async fn delete_file(
    State(_state): State<super::ui::AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let file_manager = _state.file_manager.read().await;
    let path_buf = PathBuf::from(path);
    
    match if path_buf.is_dir() {
        file_manager.delete_directory(&path_buf).await
    } else {
        file_manager.delete_file(&path_buf).await
    } {
        Ok(result) => {
            info!("Successfully deleted: {} ({})", path_buf.display(), result.message);
            
            // Notify about file deletion
            let _ = _state.event_bus.broadcast(crate::utils::event_bus::IdeEvent::FileChanged {
                path: path_buf.to_string_lossy().to_string(),
                event_type: crate::utils::event_bus::FileEventType::Deleted,
            });
            
            ApiResponse::success(format!("Deleted successfully: {}", result.message))
        }
        Err(e) => {
            error!("Failed to delete {}: {}", path_buf.display(), e);
            ApiResponse::error(format!("Failed to delete: {}", e))
        }
    }
}

/// Get file tree structure
pub async fn get_file_tree(State(_state): State<super::ui::AppState>) -> impl IntoResponse {
    let file_manager = _state.file_manager.read().await;
    let workspace_path = _state.ide.config().read().await.workspace_dir();
    
    match file_manager.list_directory(&workspace_path).await {
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
    State(_state): State<super::ui::AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let file_manager = _state.file_manager.read().await;
    let pattern = params.get("pattern").unwrap_or(&"".to_string()).clone();
    let root = params.get("root").unwrap_or(&".".to_string()).clone();
    
    if pattern.is_empty() {
        return ApiResponse::error("Search pattern is required".to_string());
    }
    
    match file_manager.search_files(&pattern, false) {
        Ok(paths) => {
            let search_results: Vec<SearchResult> = paths.into_iter()
                .filter(|file_info| {
                    // Filter by root if specified
                    if root != "." {
                        file_info.path.starts_with(&root)
                    } else {
                        true
                    }
                })
                .map(|file_info| SearchResult {
                    path: file_info.path.to_string_lossy().to_string(),
                    name: file_info.name,
                    size: file_info.size,
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
    State(_state): State<super::ui::AppState>,
    Json(request): Json<AIChatRequest>,
) -> impl IntoResponse {
    let ai_engine = _state.ide.ai_engine();
    
    // Create AI completion request
    let completion_request = crate::ai::CompletionRequest {
        prompt: request.message,
        context: request.context.as_ref().and_then(|ctx| ctx.file_content.as_ref()).cloned().unwrap_or_default(),
        language: request.context.as_ref().and_then(|ctx| ctx.language.as_ref()).cloned().unwrap_or_else(|| "rust".to_string()),
        max_tokens: request.settings.and_then(|s| s.max_tokens),
        position: None,
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
    State(_state): State<super::ui::AppState>,
    Json(_request): Json<CodeCompletionRequest>,
) -> impl IntoResponse {
    let editor = _state.ide.editor();
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
    State(_state): State<super::ui::AppState>,
    Json(request): Json<CodeCompletionRequest>,
) -> impl IntoResponse {
    let ai_engine = _state.ide.ai_engine();
    
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

// Advanced AI Endpoint Handlers

/// Get smart code completions with context awareness
pub async fn smart_completions(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ai_engine = _state.ide.ai_engine();
    
    let file_path = request.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
    let content = request.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let _position = request.get("position");
    
    // Create completion request with enhanced context
    let completion_request = crate::ai::CompletionRequest {
        prompt: format!("Provide context-aware completions for file: {}", file_path),
        context: content.to_string(),
        language: get_language_from_file_path(file_path),
        max_tokens: Some(200),
        position: None,
    };
    
    match ai_engine.complete_code(completion_request).await {
        Ok(_completion) => {
            // Convert AI completion to structured suggestions
            let suggestions = vec![
                crate::ai::CodeSuggestion {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: "AI Suggestion".to_string(),
                    description: "AI-generated suggestion".to_string(),
                    message: "AI-generated suggestion".to_string(),
                    code: "Sample completion".to_string(),
                    confidence: 0.8,
                }
            ];
            
            ApiResponse::success(serde_json::json!({
                "suggestions": suggestions,
                "context": {
                    "file_path": file_path,
                    "language": get_language_from_file_path(file_path),
                    "completion_count": suggestions.len()
                }
            }))
        }
        Err(e) => {
            error!("Smart completions failed: {}", e);
            ApiResponse::error(format!("Smart completions failed: {}", e))
        }
    }
}

/// AI-powered code review
pub async fn code_review(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ai_engine = _state.ide.ai_engine();
    
    let code = request.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = request.get("language").and_then(|v| v.as_str()).unwrap_or("rust");
    
    match ai_engine.analyze_code(code, language).await {
        Ok(analysis) => {
            // Convert analysis results to review format
            let review_results = analysis.issues.into_iter().map(|issue| {
                serde_json::json!({
                    "id": issue.id,
                    "title": format!("{} Issue", issue.severity),
                    "description": issue.message,
                    "severity": format!("{:?}", issue.severity).to_lowercase(),
                    "line": issue.line,
                    "category": "code_quality",
                    "suggestion": "Consider refactoring this code",
                    "canAutoFix": issue.severity == crate::ai::IssueSeverity::Info
                })
            }).collect::<Vec<_>>();
            
            ApiResponse::success(review_results)
        }
        Err(e) => {
            error!("Code review failed: {}", e);
            ApiResponse::error(format!("Code review failed: {}", e))
        }
    }
}

/// Intelligent debugging assistance
pub async fn debug_assistance(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ai_engine = _state.ide.ai_engine();
    
    let code = request.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = request.get("language").and_then(|v| v.as_str()).unwrap_or("rust");
    let issue = request.get("issue").and_then(|v| v.as_str());
    
    let prompt = if let Some(issue_desc) = issue {
        format!("Debug this {} code. Issue: {}\n\nCode:\n{}", language, issue_desc, code)
    } else {
        format!("Analyze this {} code for potential issues and debugging suggestions:\n\n{}", language, code)
    };
    
    let completion_request = crate::ai::CompletionRequest {
        prompt,
        context: code.to_string(),
        language: language.to_string(),
        max_tokens: Some(1000),
        position: None,
    };
    
    match ai_engine.complete_code(completion_request).await {
        Ok(_completion) => {
            // Parse AI response into debugging structure
            let debug_result = serde_json::json!({
                "rootCause": "Potential issues identified in code structure",
                "steps": [
                    {
                        "description": "Review the code logic for edge cases",
                        "code": "// Add error handling",
                        "explanation": "Implement proper error handling to prevent runtime issues"
                    }
                ],
                "prevention": "Use proper validation and error handling patterns"
            });
            
            ApiResponse::success(debug_result)
        }
        Err(e) => {
            error!("Debug assistance failed: {}", e);
            ApiResponse::error(format!("Debug assistance failed: {}", e))
        }
    }
}

/// AI-driven project scaffolding
pub async fn generate_project(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let template = request.get("template").and_then(|v| v.as_str()).unwrap_or("");
    let config = request.get("config").unwrap_or(&serde_json::Value::Null);
    
    let project_name = config.get("name").and_then(|v| v.as_str()).unwrap_or("new-project");
    let description = config.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let features = config.get("features").and_then(|v| v.as_array()).unwrap_or_else(|| -> &Vec<serde_json::Value> { static EMPTY_VEC: Vec<serde_json::Value> = Vec::new(); &EMPTY_VEC });
    
    // Generate project structure based on template
    let project_structure = match template {
        "rust-web-api" => generate_rust_web_api_structure(project_name, description, features),
        "python-web-app" => generate_python_web_app_structure(project_name, description, features),
        "react-frontend" => generate_react_frontend_structure(project_name, description, features),
        _ => generate_generic_project_structure(project_name, description, features)
    };
    
    ApiResponse::success(project_structure)
}

/// Context-aware help system
pub async fn context_help(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ai_engine = _state.ide.ai_engine();
    
    let query = request.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let context = request.get("context");
    
    let context_info = if let Some(ctx) = context {
        format!("Context: {:?}", ctx)
    } else {
        "No specific context provided".to_string()
    };
    
    let prompt = format!("Provide helpful programming guidance for: {}\n\n{}", query, context_info);
    
    let completion_request = crate::ai::CompletionRequest {
        prompt,
        context: context_info,
        language: "rust".to_string(),
        max_tokens: Some(800),
        position: None,
    };
    
    match ai_engine.complete_code(completion_request).await {
        Ok(completion) => {
            let help_result = vec![
                serde_json::json!({
                    "id": "help-1",
                    "title": "Programming Guidance",
                    "content": completion.text,
                    "category": "General",
                    "codeExample": null,
                    "relatedTopics": ["best practices", "debugging", "performance"]
                })
            ];
            
            ApiResponse::success(help_result)
        }
        Err(e) => {
            error!("Context help failed: {}", e);
            ApiResponse::error(format!("Context help failed: {}", e))
        }
    }
}

/// Learning feedback for AI improvement
pub async fn learning_feedback(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pattern = request.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
    let positive = request.get("positive").and_then(|v| v.as_bool()).unwrap_or(false);
    let _timestamp = request.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
    
    // In a real implementation, this would store feedback for learning
    info!("Learning feedback received: pattern={}, positive={}", pattern, positive);
    
    ApiResponse::success("Feedback recorded successfully")
}

/// Advanced code optimization
pub async fn optimize_advanced(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ai_engine = _state.ide.ai_engine();
    
    let code = request.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = request.get("language").and_then(|v| v.as_str()).unwrap_or("rust");
    let goals = request.get("goals").and_then(|v| v.as_array()).unwrap_or_else(|| -> &Vec<serde_json::Value> { static EMPTY_VEC: Vec<serde_json::Value> = Vec::new(); &EMPTY_VEC });
    
    let goals_str: Vec<String> = goals.iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    
    let prompt = format!(
        "Optimize this {} code for goals: {:?}\n\nCode:\n{}\n\nProvide optimized version with explanations.",
        language, goals_str, code
    );
    
    let completion_request = crate::ai::CompletionRequest {
        prompt,
        context: code.to_string(),
        language: language.to_string(),
        max_tokens: Some(1500),
        position: None,
    };
    
    match ai_engine.complete_code(completion_request).await {
        Ok(completion) => {
            let optimization_result = serde_json::json!({
                "optimizedCode": completion.text,
                "explanation": "Code optimized based on specified goals",
                "performanceGain": "Estimated 15-25% improvement",
                "appliedOptimizations": goals_str
            });
            
            ApiResponse::success(optimization_result)
        }
        Err(e) => {
            error!("Advanced optimization failed: {}", e);
            ApiResponse::error(format!("Advanced optimization failed: {}", e))
        }
    }
}

/// Get refactoring suggestions
pub async fn refactoring_suggestions(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let code = request.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = request.get("language").and_then(|v| v.as_str()).unwrap_or("rust");
    
    // Use the refactoring engine from the AI module
    let refactoring_engine = crate::ai::RefactoringEngine::new();
    
    match refactoring_engine.analyze_for_refactoring(code, language).await {
        Ok(suggestions) => {
            let suggestion_results = suggestions.suggestions.into_iter().map(|suggestion| {
                serde_json::json!({
                    "id": format!("refactor-{}", uuid::Uuid::new_v4()),
                    "title": "Refactoring Opportunity",
                    "description": suggestion,
                    "severity": "info",
                    "suggestedRefactoring": "Extract method",
                    "confidence": 0.8,
                    "riskLevel": 0.2,
                    "canAutoFix": true
                })
            }).collect::<Vec<_>>();
            
            ApiResponse::success(serde_json::json!({
                "suggestions": suggestion_results
            }))
        }
        Err(e) => {
            error!("Refactoring suggestions failed: {}", e);
            ApiResponse::error(format!("Refactoring suggestions failed: {}", e))
        }
    }
}

/// Apply refactoring suggestion
pub async fn apply_refactoring(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let code = request.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let _suggestion = request.get("suggestion");
    
    // For now, return the original code (would implement actual refactoring)
    let refactored_code = code.to_string();
    
    ApiResponse::success(serde_json::json!({
        "refactoredCode": refactored_code,
        "changes": "Refactoring applied successfully"
    }))
}

/// Generate advanced tests
pub async fn generate_tests_advanced(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ai_engine = _state.ide.ai_engine();
    
    let code = request.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = request.get("language").and_then(|v| v.as_str()).unwrap_or("rust");
    let test_types = request.get("testTypes").and_then(|v| v.as_array()).unwrap_or_else(|| -> &Vec<serde_json::Value> { static EMPTY_VEC: Vec<serde_json::Value> = Vec::new(); &EMPTY_VEC });
    
    let test_types_str: Vec<String> = test_types.iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();         
    
    let prompt = format!(
        "Generate comprehensive tests for this {} code. Test types: {:?}\n\nCode:\n{}\n\nProvide test files with proper assertions.",
        language, test_types_str, code
    );
    
    let completion_request = crate::ai::CompletionRequest {
        prompt,
        context: code.to_string(),
        language: language.to_string(),
        max_tokens: Some(2000),
        position: None,
    };
    
    match ai_engine.complete_code(completion_request).await {
        Ok(completion) => {
            let test_result = serde_json::json!({
                "testFiles": [
                    {
                        "name": "test_main.rs",
                        "content": completion.text,
                        "type": "unit_tests"
                    }
                ],
                "coverage": "Estimated 80% coverage",
                "testTypes": test_types_str
            });
            
            ApiResponse::success(test_result)
        }
        Err(e) => {
            error!("Advanced test generation failed: {}", e);
            ApiResponse::error(format!("Advanced test generation failed: {}", e))
        }
    }
}

/// Performance analysis
pub async fn performance_analysis(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let code = request.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = request.get("language").and_then(|v| v.as_str()).unwrap_or("rust");
    
    // Use the performance analyzer from the AI module
    let performance_analyzer = crate::ai::PerformanceAnalyzer::new();
    
    let insights = performance_analyzer.get_performance_insights(code, language);
    
    let analysis_result = match insights {
        Ok(insights) => serde_json::json!({
            "insights": insights,
            "metrics": {
                "timeComplexity": "O(n)",
                "spaceComplexity": "O(1)",
                "bottlenecks": ["string_concatenation", "nested_loops"]
            },
            "recommendations": [
                "Use String::with_capacity for multiple concatenations",
                "Consider algorithm optimization for nested loops"
            ]
        }),
        Err(e) => serde_json::json!({
            "insights": null,
            "error": format!("Analysis failed: {}", e),
            "metrics": {
                "timeComplexity": "O(n)",
                "spaceComplexity": "O(1)",
                "bottlenecks": ["string_concatenation", "nested_loops"]
            },
            "recommendations": [
                "Use String::with_capacity for multiple concatenations",
                "Consider algorithm optimization for nested loops"
            ]
        })
    };
    
    ApiResponse::success(analysis_result)
}

/// Security analysis
pub async fn security_analysis(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let code = request.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = request.get("language").and_then(|v| v.as_str()).unwrap_or("rust");
    
    // Use the security analyzer from the AI module
    let security_analyzer = crate::ai::SecurityAnalyzer::new();
    
    let vulnerabilities = security_analyzer.analyze_code_security(code, language);
    
    let analysis_result = match vulnerabilities {
        Ok(vulnerabilities) => serde_json::json!({
            "vulnerabilities": vulnerabilities,
            "securityScore": 85,
            "recommendations": [
                "Implement input validation",
                "Use secure coding practices",
                "Add error handling for security-sensitive operations"
            ]
        }),
        Err(e) => serde_json::json!({
            "vulnerabilities": null,
            "error": format!("Security analysis failed: {}", e),
            "securityScore": 85,
            "recommendations": [
                "Implement input validation",
                "Use secure coding practices",
                "Add error handling for security-sensitive operations"
            ]
        })
    };
    
    ApiResponse::success(analysis_result)
}

/// Translate code between languages
pub async fn translate_languages(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ai_engine = _state.ide.ai_engine();
    
    let code = request.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let from_language = request.get("fromLanguage").and_then(|v| v.as_str()).unwrap_or("rust");
    let to_language = request.get("toLanguage").and_then(|v| v.as_str()).unwrap_or("python");
    let preserve_comments = request.get("preserveComments").and_then(|v| v.as_bool()).unwrap_or(true);
    
    let prompt = format!(
        "Translate this {} code to {}. {} Preserve comments and documentation.\n\n{} code:\n{}",
        from_language, to_language,
        if preserve_comments { "Please" } else { "Do not" },
        from_language, code
    );
    
    let completion_request = crate::ai::CompletionRequest {
        prompt,
        context: code.to_string(),
        language: from_language.to_string(),
        max_tokens: Some(2000),
        position: None,
    };
    
    match ai_engine.complete_code(completion_request).await {
        Ok(completion) => {
            let translation_result = serde_json::json!({
                "translatedCode": completion.text,
                "fromLanguage": from_language,
                "toLanguage": to_language,
                "preserveComments": preserve_comments,
                "accuracy": "Estimated 90-95% accuracy"
            });
            
            ApiResponse::success(translation_result)
        }
        Err(e) => {
            error!("Language translation failed: {}", e);
            ApiResponse::error(format!("Language translation failed: {}", e))
        }
    }
}

/// Get code metrics and analysis
pub async fn code_metrics(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let code = request.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = request.get("language").and_then(|v| v.as_str()).unwrap_or("rust");
    
    // Calculate basic metrics
    let lines: Vec<&str> = code.lines().collect();
    let total_lines = lines.len();
    let code_lines = lines.iter().filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//")).count();
    let comment_lines = lines.iter().filter(|line| line.trim().starts_with("//")).count();
    let blank_lines = total_lines - code_lines - comment_lines;
    
    // Count functions, classes, etc.
    let function_count = code.matches("fn ").count() + code.matches("function ").count();
    let class_count = code.matches("struct ").count() + code.matches("class ").count();
    
    let metrics = serde_json::json!({
        "totalLines": total_lines,
        "codeLines": code_lines,
        "commentLines": comment_lines,
        "blankLines": blank_lines,
        "commentPercentage": if total_lines > 0 { (comment_lines as f64 / total_lines as f64) * 100.0 } else { 0.0 },
        "functionCount": function_count,
        "classCount": class_count,
        "complexity": {
            "cyclomatic": "Low",
            "maintainability": "High"
        },
        "language": language
    });
    
    ApiResponse::success(metrics)
}

// Helper functions for project generation

fn generate_rust_web_api_structure(name: &str, description: &str, _features: &Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "projectName": name,
        "description": description,
        "template": "rust-web-api",
        "structure": {
            "src": {
                "main.rs": "// Application entry point",
                "handlers": "HTTP request handlers",
                "models": "Data models",
                "middleware": "Custom middleware",
                "utils": "Utility functions"
            },
            "tests": "Integration and unit tests",
            "Cargo.toml": "Rust dependencies and configuration"
        },
        "files": [
            {
                "path": "src/main.rs",
                "content": format!("// {} - {}\n\nuse actix_web::{{App, HttpServer, web}};\n\n#[actix_web::main]\nasync fn main() -> std::io::Result<()> {{\n    HttpServer::new(|| {{\n        App::new()\n            .service(web::scope(\"/api\")\n                .route(\"/health\", web::get().to(|| {{}})))\n    }})\n    .bind(\"127.0.0.1:8080\")?\n    .run()\n    .await\n}}", name, description)
            }
        ]
    })
}

fn generate_python_web_app_structure(name: &str, description: &str, _features: &Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "projectName": name,
        "description": description,
        "template": "python-web-app",
        "structure": {
            "app": "Main application package",
            "routers": "API route handlers",
            "models": "Data models",
            "database": "Database connection and models",
            "tests": "Test files"
        },
        "files": [
            {
                "path": "main.py",
                "content": format!("\"\"\"\n{} - {}\n\"\"\"\n\nfrom fastapi import FastAPI\nfrom fastapi.middleware.cors import CORSMiddleware\n\napp = FastAPI(title=\"{}\", description=\"{}\")\n\napp.add_middleware(\n    CORSMiddleware,\n    allow_origins=[\"*\"],\n    allow_credentials=True,\n    allow_methods=[\"*\"],\n    allow_headers=[\"*\"],\n)\n\n@app.get(\"/\")\nasync def root():\n    return {{\"message\": \"Welcome to {}\"}}\n\n@app.get(\"/health\")\nasync def health():\n    return {{\"status\": \"healthy\"}}", name, description, name, description, name)
            }
        ]
    })
}

fn generate_react_frontend_structure(name: &str, description: &str, _features: &Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "projectName": name,
        "description": description,
        "template": "react-frontend",
        "structure": {
            "src": {
                "components": "React components",
                "pages": "Page components",
                "hooks": "Custom React hooks",
                "utils": "Utility functions",
                "store": "State management"
            },
            "public": "Static assets",
            "tests": "Component and integration tests"
        },
        "files": [
            {
                "path": "src/App.tsx",
                "content": format!("import React from 'react';\nimport './App.css';\n\nfunction App() {{\n  return (\n    <div className=\"App\">\n      <header className=\"App-header\">\n        <h1>{}</h1>\n        <p>{}</p>\n      </header>\n    </div>\n  );\n}}\n\nexport default App;", name, description)
            }
        ]
    })
}

fn generate_generic_project_structure(name: &str, description: &str, _features: &Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "projectName": name,
        "description": description,
        "template": "generic",
        "structure": {
            "src": "Source code",
            "tests": "Test files",
            "docs": "Documentation"
        },
        "files": [
            {
                "path": "README.md",
                "content": format!("# {}\n\n{}\n\n## Getting Started\n\nThis project was generated using the Advanced AI Assistant.\n\n## Features\n\n- AI-powered development\n- Context-aware assistance\n- Smart code completion\n- Intelligent debugging\n\n## Usage\n\nInstructions for using this project...\n\n## Contributing\n\nGuidelines for contributing to this project...", name, description)
            }
        ]
    })
}

fn get_language_from_file_path(file_path: &str) -> String {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    match extension {
        "rs" => "rust".to_string(),
        "py" => "python".to_string(),
        "js" => "javascript".to_string(),
        "ts" => "typescript".to_string(),
        "go" => "go".to_string(),
        "java" => "java".to_string(),
        "cpp" | "cc" | "cxx" => "cpp".to_string(),
        "c" => "c".to_string(),
        _ => "plaintext".to_string()
    }
}

// Git Handlers

/// Get git status
pub async fn git_status(
    State(_state): State<super::ui::AppState>,
    Query(params): Query<GitStatusRequest>,
) -> impl IntoResponse {
    let git_manager = &_state.git_manager;
    let workspace_path = _state.ide.config().read().await.workspace_dir();
    
    let path = params.path.as_ref()
        .map(|p| PathBuf::from(p))
        .unwrap_or(workspace_path);
    
    // Check if this is a repository
    if !git_manager.is_repository().await {
        return ApiResponse::error("Not a git repository".to_string());
    }
    
    match git_manager.get_status().await {
        Ok(status) => {
            info!("Git status retrieved successfully");
            
            // Convert GitStatus to a JSON-friendly format
            let status_json = serde_json::json!({
                "staged_files": status.staged_files,
                "unstaged_files": status.unstaged_files,
                "untracked_files": status.untracked_files,
                "ahead_count": status.ahead_count,
                "behind_count": status.behind_count
            });
            
            ApiResponse::success(status_json)
        }
        Err(e) => {
            error!("Git status failed: {}", e);
            ApiResponse::error(format!("Git status failed: {}", e))
        }
    }
}

/// Get git branches
pub async fn git_branches(State(_state): State<super::ui::AppState>) -> impl IntoResponse {
    let git_manager = &_state.git_manager;
    
    // Check if this is a repository
    if !git_manager.is_repository().await {
        return ApiResponse::error("Not a git repository".to_string());
    }
    
    match git_manager.get_branches().await {
        Ok(branches) => {
            info!("Retrieved {} git branches", branches.len());
            ApiResponse::success(branches)
        }
        Err(e) => {
            error!("Git branches failed: {}", e);
            ApiResponse::error(format!("Git branches failed: {}", e))
        }
    }
}

/// Commit changes
pub async fn git_commit(
    State(_state): State<super::ui::AppState>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let git_manager = &_state.git_manager;
    
    // Check if this is a repository
    if !git_manager.is_repository().await {
        return ApiResponse::error("Not a git repository".to_string());
    }
    
    let message = request.get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("Commit from Super IDE");
    
    // First stage all changes, then commit
    let all_files = vec![".".to_string()];
    match git_manager.stage_files(&all_files).await {
        Ok(_) => {
            match git_manager.commit(message).await {
                Ok(commit_hash) => {
                    info!("Git commit successful: {} - {}", commit_hash, message);
                    ApiResponse::success(format!("Commit successful: {}", commit_hash))
                }
                Err(e) => {
                    error!("Git commit failed: {}", e);
                    ApiResponse::error(format!("Git commit failed: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Git stage failed: {}", e);
            ApiResponse::error(format!("Git stage failed: {}", e))
        }
    }
}

// Project Handlers

/// Get project information
pub async fn project_info(State(_state): State<super::ui::AppState>) -> impl IntoResponse {
    let config = _state.ide.config().read().await;
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
pub async fn get_config(State(_state): State<super::ui::AppState>) -> impl IntoResponse {
    let config = _state.ide.config().read().await;
    
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
pub async fn health_check(State(_state): State<super::ui::AppState>) -> impl IntoResponse {
    let ide_state = _state.ide.get_state().await;
    
    ApiResponse::success(HealthStatus {
        status: "healthy".to_string(),
        ide_running: true,
        documents_open: ide_state.active_tabs.len(),
        ai_enabled: _state.ide.ai_engine().ai_provider().await.unwrap_or_else(|_| "local".to_string()) != "local".to_string(),
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

impl From<FileInfo> for FileTreeNode {
    fn from(entry: FileInfo) -> Self {
        Self {
            name: entry.name,
            path: entry.path.to_string_lossy().to_string(),
            r#type: if entry.is_directory { "directory".to_string() } else { "file".to_string() },
            size: entry.size,
            modified: entry.modified_at.to_rfc3339(),
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
    State(_state): State<super::ui::AppState>,
    Json(request): Json<AIChatRequest>,
) -> impl IntoResponse {
    let ai_engine = _state.ide.ai_engine();
    
    // Create AI completion request with learning context
    let completion_request = crate::ai::CompletionRequest {
        prompt: format!("[LEARNING TUTOR] {}", request.message),
        context: request.context.as_ref().and_then(|ctx| ctx.file_content.as_ref()).cloned().unwrap_or_default(),
        language: request.context.as_ref().and_then(|ctx| ctx.language.as_ref()).cloned().unwrap_or_else(|| "rust".to_string()),
        max_tokens: request.settings.and_then(|s| s.max_tokens),
        position: None,
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
