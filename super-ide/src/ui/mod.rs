//! UI components and web interface for Super IDE

use axum::{
    extract::{WebSocketUpgrade, State},
    routing::{get, post, put, delete},
    extract::ws::WebSocket,
    response::{IntoResponse, Html},
    Router,
    Json,
};
use axum::extract::Path;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use futures::{StreamExt, SinkExt};
use log::info;
use tokio::sync::RwLock;

use crate::core::SuperIDE;

use crate::terminal::ws_handler::TerminalWebSocketState;
use crate::utils::file_manager::FileManager;
use crate::utils::event_bus::EventBus;

use crate::editor::{CompletionContext, CompletionItem};

// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "completion")]
    Completion {
        context: CompletionRequest,
        completions: Vec<CompletionItem>,
    },
    #[serde(rename = "ai_suggestion")]
    AiSuggestion {
        suggestion: String,
        confidence: f32,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
        code: String,
    },
    #[serde(rename = "code_analysis")]
    CodeAnalysis {
        analysis: String,
    },
    #[serde(rename = "bug_prediction")]
    BugPrediction {
        predictions: Vec<String>,
    },
}

// Completion request from client
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub document_id: String,
    pub cursor_position: (usize, usize),
    pub text_before: String,
    pub text_after: String,
    pub language: String,
}

// UI state for the web interface
#[derive(Debug, Clone)]
pub struct UiState {
    pub ide: Arc<SuperIDE>,
    pub event_sender: broadcast::Sender<UiEvent>,
}

// UI events
#[derive(Debug, Clone)]
pub enum UiEvent {
    CodeChanged {
        document_id: String,
        content: String,
        position: (usize, usize),
    },
    CompletionRequest {
        document_id: String,
        context: CompletionContext,
    },
    FileOpened {
        document_id: String,
        file_path: String,
    },
    FileSaved {
        document_id: String,
    },
}

// Unified App State
#[derive(Clone)]
pub struct AppState {
    pub ide: Arc<SuperIDE>,
    pub file_manager: Arc<RwLock<FileManager>>,
    pub event_bus: Arc<EventBus>,
    pub event_sender: broadcast::Sender<UiEvent>,
}

// Main UI handler
pub struct WebUI {
    app_state: AppState,
    server_task: Option<tokio::task::JoinHandle<()>>,
}

impl WebUI {
    /// Create a new web UI instance
    pub fn new(ide: Arc<SuperIDE>) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        let file_manager = Arc::new(RwLock::new(FileManager::default()));
        let event_bus = ide.event_bus().clone();
        
        Self {
            app_state: AppState {
                ide: ide.clone(),
                file_manager,
                event_bus,
                event_sender,
            },
            server_task: None,
        }
    }
    
    /// Start the web server
    pub async fn start(&mut self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        // Import API handlers into the UI module scope
        use crate::api::{load_file, save_file, create_file, delete_file, get_file_tree, search_files};
        use crate::api::{ai_chat, get_completions, analyze_code};
        use crate::api::{git_status, git_branches, git_commit};
        use crate::api::{project_info, get_config, health_check};
        
        let app = Router::new()
            // Static file serving for frontend
            .route("/", get(serve_frontend))
            .route("/health", get(health_check))
            
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
            
            // WebSocket endpoints
            .route("/ws", get(websocket_handler))
            .route("/ws/terminal", get(terminal_websocket_handler))
            
            // Legacy UI routes (for backward compatibility)
            .route("/api/files", get(list_files))
            .route("/api/open/:path", get(open_file))
            .route("/api/save/:document_id", get(save_document))
            .route("/api/complete", get(get_completion))
            .route("/api/analyze", post(analyze_code))
            .route("/api/ai/suggest", post(get_ai_suggestion))
            
            .layer(CorsLayer::new().allow_origin(Any))
            .with_state(self.app_state.clone());
            
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(&addr).await?;
        
        println!("🚀 Super IDE Web UI starting on http://localhost:{}", port);
        
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        });
        self.server_task = Some(tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("Server error: {}", e);
            }
        }));
        
        Ok(())
    }
    
    /// Stop the web server
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(task) = self.server_task.take() {
            task.abort();
        }
        Ok(())
    }
}

// HTTP Handlers

/// Serve frontend HTML
async fn serve_frontend() -> impl IntoResponse {
    Html(include_str!("./web/index.html"))
}

/// Main HTML interface (legacy)


/// Terminal WebSocket handler
async fn terminal_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let terminal_state = TerminalWebSocketState {
        ide: state.ide.clone(),
        terminal_manager: Arc::new(tokio::sync::RwLock::new(
            crate::terminal::TerminalManager::new(
                crate::terminal::TerminalConfig::default()
            )
        )),
    };
    
    crate::terminal::ws_handler::terminal_websocket_handler(ws, axum::extract::State(terminal_state)).await
}



/// List files in workspace
async fn list_files(State(state): State<AppState>) -> impl IntoResponse {
    let workspace_path = state.ide.config().read().await.workspace_dir();
    
    match std::fs::read_dir(workspace_path) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        files.push(FileInfo {
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: entry.path().to_string_lossy().to_string(),
                            size: metadata.len(),
                            modified: metadata.modified()
                                .map(chrono::DateTime::<chrono::Utc>::from)
                                .unwrap_or_else(|_| chrono::Utc::now()),
                            is_file: true,
                        });
                    } else if metadata.is_dir() {
                        files.push(FileInfo {
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: entry.path().to_string_lossy().to_string(),
                            size: 0,
                            modified: metadata.modified()
                                .map(chrono::DateTime::<chrono::Utc>::from)
                                .unwrap_or_else(|_| chrono::Utc::now()),
                            is_file: false,
                        });
                    }
                }
            }
            Json(files)
        }
        Err(_) => Json(vec![]),
    }
}

/// Open a file
async fn open_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let path_clone = path.clone();
    let path_buf = std::path::PathBuf::from(path);
    
    let editor = state.ide.editor();
    let editor_lock = editor.lock().await;
    
    match editor_lock.open_file(path_buf).await {
        Ok(document_id) => {
            // Notify via WebSocket
            let _ = state.event_sender.send(UiEvent::FileOpened {
                document_id: document_id.clone(),
                file_path: path_clone,
            });
            
            Json(serde_json::json!({
                "success": true,
                "document_id": document_id,
                "message": "File opened successfully"
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

/// Save document
async fn save_document(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> impl IntoResponse {
    let editor = state.ide.editor();
    let editor_lock = editor.lock().await;
    
    match editor_lock.save_active_document().await {
        Ok(_) => {
            let _ = state.event_sender.send(UiEvent::FileSaved {
                document_id: document_id.clone(),
            });
            
            Json(serde_json::json!({
                "success": true,
                "message": "Document saved successfully"
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

/// Get auto-completion suggestions
async fn get_completion(
    State(state): State<AppState>,
    Json(request): Json<CompletionRequest>,
) -> impl IntoResponse {
    match state.ide.get_code_completions(&request.document_id, (request.cursor_position.0, request.cursor_position.1), &request.text_before).await {
        Ok(completions) => {
            Json(serde_json::json!({"success": true, "completions": completions}))
        }
        Err(e) => {
            Json(serde_json::json!({"success": false, "error": e.to_string(), "completions": []}))
        }
    }
}



/// Get AI suggestion
async fn get_ai_suggestion(
    State(state): State<AppState>,
    Json(payload): Json<AiSuggestionRequest>,
) -> impl IntoResponse {
    let ai_engine = state.ide.ai_engine();
    
    let request = crate::ai::CompletionRequest {
        prompt: payload.context.clone(),
        context: payload.context,
        language: payload.language,
        max_tokens: None,
    };
    
    match ai_engine.generate_completion(request).await {
        Ok(suggestion) => {
            Json(serde_json::json!({
                "success": true,
                "suggestion": suggestion.text,
                "confidence": suggestion.confidence,
                "type": "completion"
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}

/// WebSocket handler for real-time features
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_connection(socket, state))
}

/// WebSocket connection handler
async fn websocket_connection(
    socket: WebSocket,
    state: AppState,
) {
    println!("🔗 New WebSocket connection established");

    // Subscribe to events
    let mut event_receiver = state.event_sender.subscribe();

    // Handle messages from client
    let (mut sender, mut receiver) = socket.split();

    // Clone IDE for the event task
    let ide_clone = state.ide.clone();

    // Start event forwarding task
    let event_task = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            let message = match event {
                UiEvent::CodeChanged { document_id, content: _, position } => {
                    WsMessage::CodeAnalysis {
                        analysis: format!("Code changed in document {} at position {:?}", document_id, position)
                    }
                },
                UiEvent::FileOpened { document_id: _, file_path } => {
                    WsMessage::CodeAnalysis {
                        analysis: format!("Opened file: {}", file_path)
                    }
                },
                UiEvent::FileSaved { document_id } => {
                    WsMessage::CodeAnalysis {
                        analysis: format!("Saved document: {}", document_id)
                    }
                },
                UiEvent::CompletionRequest { document_id, context } => {
                    // Get actual completions from the IDE
                    let completions = ide_clone.get_code_completions(
                        &document_id,
                        (context.cursor_position.line, context.cursor_position.column),
                        &context.text_before_cursor
                    ).await.unwrap_or_default();

                    WsMessage::Completion {
                        context: CompletionRequest {
                            document_id,
                            cursor_position: (context.cursor_position.line, context.cursor_position.column),
                            text_before: context.text_before_cursor,
                            text_after: context.text_after_cursor,
                            language: context.language,
                        },
                        completions,
                    }
                }
            };

            if let Ok(json) = serde_json::to_string(&message) {
                if sender.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming WebSocket messages
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            if let Ok(text) = msg.to_text() {
                if let Ok(client_message) = serde_json::from_str::<ClientMessage>(text) {
                    handle_client_message(client_message, &state).await;
                }
            }
        } else {
            break;
        }
    }

    event_task.abort();
    println!("🔌 WebSocket connection closed");
}

// Request/Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub is_file: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeAnalysisRequest {
    pub code: String,
    pub language: String,
    pub document_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiSuggestionRequest {
    pub context: String,
    pub language: String,
    pub document_id: String,
}

// Client message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "request_completion")]
    RequestCompletion {
        document_id: String,
        cursor_position: (usize, usize),
        text_context: String,
    },
    #[serde(rename = "save_feedback")]
    SaveFeedback {
        suggestion_id: String,
        rating: i32,
        accepted: bool,
        context: String,
    },
    #[serde(rename = "code_change")]
    CodeChange {
        document_id: String,
        content: String,
        position: (usize, usize),
    },
}

// Handle client messages
async fn handle_client_message(message: ClientMessage, state: &AppState) {
    match message {
        ClientMessage::RequestCompletion { document_id, cursor_position, text_context } => {
            let context = CompletionContext {
                cursor_position: crate::editor::CursorPosition {
                    line: cursor_position.0,
                    column: cursor_position.1,
                },
                language: "Rust".to_string(), // Would be detected from document
                text_before_cursor: text_context.clone(),
                text_after_cursor: String::new(),
            };
            
            let _ = state.event_sender.send(UiEvent::CompletionRequest {
                document_id,
                context,
            });
        }
        ClientMessage::SaveFeedback { suggestion_id, rating, accepted, context } => {
            // Implement user feedback functionality
            let _feedback = crate::ai::UserFeedback {
                timestamp: chrono::Utc::now(),
                suggestion_id: suggestion_id.clone(),
                rating,
                accepted,
                context: context.clone(),
                feedback_type: if accepted { "acceptance".to_string() } else { "rejection".to_string() },
            };
            
            // Send feedback to AI engine for learning
            let ai_engine = state.ide.ai_engine();
            // Use learn_from_feedback method with string-based pattern ID
            let pattern_id = format!("suggestion_{}", suggestion_id);
            if let Err(e) = ai_engine.learn_from_feedback(pattern_id, accepted).await {
                eprintln!("Failed to process user feedback: {}", e);
            }
            
            // Log feedback for analytics
            info!(
                "User feedback: suggestion_id={}, rating={}, accepted={}, context_len={}",
                suggestion_id,
                rating,
                accepted,
                context.len()
            );
        }
        ClientMessage::CodeChange { document_id, content, position } => {
            let _ = state.event_sender.send(UiEvent::CodeChanged {
                document_id,
                content,
                position,
            });
        }
    }
}

// Export for main application - removed to avoid conflicts
