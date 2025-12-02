//! UI components and web interface for Super IDE

use axum::{
    extract::{WebSocketUpgrade, State},
    routing::post,
    extract::ws::WebSocket,
    response::{IntoResponse, Html},
    http::StatusCode,
    routing::get,
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

use crate::core::SuperIDE;
use crate::config::AIProvider;

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

// Main UI handler
pub struct WebUI {
    state: UiState,
    server_task: Option<tokio::task::JoinHandle<()>>,
}

impl WebUI {
    /// Create a new web UI instance
    pub fn new(ide: Arc<SuperIDE>) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            state: UiState {
                ide,
                event_sender,
            },
            server_task: None,
        }
    }
    
    /// Start the web server
    pub async fn start(&mut self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/", get(index))
            .route("/api/health", get(health_check))
            .route("/api/files", get(list_files))
            .route("/api/open/:path", get(open_file))
            .route("/api/save/:document_id", get(save_document))
            .route("/api/complete", get(get_completion))
            .route("/api/analyze", post(analyze_code))
            .route("/api/ai/suggest", post(get_ai_suggestion))
            .route("/ws", get(websocket_handler))
            .layer(CorsLayer::new().allow_origin(Any))
            .with_state(self.state.clone());
            
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(&addr).await?;
        
        println!("🚀 Super IDE Web UI starting on http://localhost:{}", port);
        
        let server = axum::serve(listener, app);
        self.server_task = Some(tokio::spawn(server));
        
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

/// Main HTML interface
async fn index() -> impl IntoResponse {
    Html(include_str!("./web/index.html"))
}

/// Health check endpoint
async fn health_check(State(state): State<UiState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "ide_running": true,
        "documents_open": state.ide.get_state().await.active_tabs.len(),
        "ai_enabled": state.ide.ai_engine().ai_provider() != &AIProvider::Local,
    }))
}

/// List files in workspace
async fn list_files(State(state): State<UiState>) -> impl IntoResponse {
    let workspace_path = state.ide.config().read().await.workspace_dir();
    
    match std::fs::read_dir(workspace_path) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            files.push(FileInfo {
                                name: entry.file_name().to_string_lossy().to_string(),
                                path: entry.path().to_string_lossy().to_string(),
                                size: metadata.len(),
                                modified: metadata.modified()
                                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t))
                                    .unwrap_or_else(|_| chrono::Utc::now()),
                                is_file: true,
                            });
                        } else if metadata.is_dir() {
                            files.push(FileInfo {
                                name: entry.file_name().to_string_lossy().to_string(),
                                path: entry.path().to_string_lossy().to_string(),
                                size: 0,
                                modified: metadata.modified()
                                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t))
                                    .unwrap_or_else(|_| chrono::Utc::now()),
                                is_file: false,
                            });
                        }
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
    State(state): State<UiState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let path_buf = std::path::PathBuf::from(path);
    
    let editor = state.ide.editor();
    let mut editor_lock = editor.lock().await;
    
    match editor_lock.open_file(path_buf).await {
        Ok(document_id) => {
            // Notify via WebSocket
            let _ = state.event_sender.send(UiEvent::FileOpened {
                document_id: document_id.clone(),
                file_path: path.to_string(),
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
    State(state): State<UiState>,
    Path(document_id): Path<String>,
) -> impl IntoResponse {
    let editor = state.ide.editor();
    let mut editor_lock = editor.lock().await;
    
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
    State(state): State<UiState>,
) -> impl IntoResponse {
    // This would handle completion requests
    Json(vec![])
}

/// Analyze code using AI
async fn analyze_code(
    State(state): State<UiState>,
    Json(payload): Json<CodeAnalysisRequest>,
) -> impl IntoResponse {
    let ai_engine = state.ide.ai_engine();
    
    match ai_engine.analyze_code(&payload.code, &payload.language).await {
        Ok(analysis) => {
            Json(serde_json::json!({
                "success": true,
                "analysis": {
                    "language": analysis.language,
                    "complexity": {
                        "cyclomatic": analysis.complexity.cyclomatic_complexity,
                        "cognitive": analysis.complexity.cognitive_complexity,
                        "maintainability": analysis.complexity.maintainability_index,
                    },
                    "suggestions": analysis.suggestions.len(),
                    "functions": analysis.functions.len(),
                }
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

/// Get AI suggestion
async fn get_ai_suggestion(
    State(state): State<UiState>,
    Json(payload): Json<AiSuggestionRequest>,
) -> impl IntoResponse {
    let ai_engine = state.ide.ai_engine();
    
    match ai_engine.generate_completion(&payload.context, &payload.language).await {
        Ok(suggestion) => {
            Json(serde_json::json!({
                "success": true,
                "suggestion": suggestion,
                "confidence": 0.85,
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
    State(state): State<UiState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_connection(socket, state))
}

/// WebSocket connection handler
async fn websocket_connection(
    mut socket: WebSocket,
    state: UiState,
) {
    println!("🔗 New WebSocket connection established");
    
    // Subscribe to events
    let mut event_receiver = state.event_sender.subscribe();
    
    // Handle messages from client
    let (mut sender, mut receiver) = socket.split();
    
    // Start event forwarding task
    let event_task = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            let message = match event {
                UiEvent::CodeChanged { document_id, content, position } => {
                    WsMessage::CodeAnalysis {
                        analysis: format!("Code changed in document {} at position {:?}", document_id, position)
                    }
                },
                UiEvent::FileOpened { document_id, file_path } => {
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
                    WsMessage::Completion {
                        context: CompletionRequest {
                            document_id,
                            cursor_position: (context.cursor_position.line, context.cursor_position.column),
                            text_before: context.text_before_cursor,
                            text_after: context.text_after_cursor,
                            language: context.language,
                        },
                        completions: vec![], // Would be populated with actual completions
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
            if msg.is_text() {
                if let Ok(text) = msg.to_text() {
                    if let Ok(client_message) = serde_json::from_str::<ClientMessage>(text) {
                        handle_client_message(client_message, &state).await;
                    }
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
async fn handle_client_message(message: ClientMessage, state: &UiState) {
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
            // TODO: Implement user feedback
            // let feedback = UserFeedback {
            //     timestamp: chrono::Utc::now(),
            //     suggestion_id,
            //     rating,
            //     accepted,
            //     context,
            // };
            
            // let ai_engine = state.ide.ai_engine();
            // ai_engine.learn_from_feedback(feedback).await;
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