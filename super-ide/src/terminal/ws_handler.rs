//! WebSocket handler for terminal functionality
//!
//! This module handles WebSocket connections for terminal sessions,
//! enabling real-time terminal communication between frontend and backend.

use axum::{
    extract::{WebSocketUpgrade, State},
    response::IntoResponse,
    extract::ws::WebSocket,
};
use futures::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use log::{info, warn, error};
use uuid::Uuid;


use crate::core::SuperIDE;
use crate::terminal::{TerminalManager};

// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TerminalMessage {
    #[serde(rename = "create_session")]
    CreateSession {
        session_id: Option<String>,
        shell: Option<String>,
        cwd: Option<String>,
    },
    #[serde(rename = "execute_command")]
    ExecuteCommand {
        session_id: String,
        command: String,
        cwd: Option<String>,
    },
    #[serde(rename = "resize")]
    Resize {
        session_id: String,
        width: u16,
        height: u16,
    },
    #[serde(rename = "close_session")]
    CloseSession {
        session_id: String,
    },
    #[serde(rename = "get_history")]
    GetHistory {
        session_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "session_created")]
    SessionCreated {
        session_id: String,
        success: bool,
        message: Option<String>,
    },
    #[serde(rename = "command_output")]
    CommandOutput {
        session_id: String,
        output: String,
        timestamp: String,
        is_error: bool,
    },
    #[serde(rename = "session_closed")]
    SessionClosed {
        session_id: String,
        exit_code: Option<i32>,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
        code: String,
        session_id: Option<String>,
    },
    #[serde(rename = "history")]
    History {
        session_id: String,
        commands: Vec<String>,
    },
    #[serde(rename = "status")]
    Status {
        session_id: String,
        status: String,
        cwd: Option<String>,
    },
}

// Terminal WebSocket state
#[derive(Clone)]
pub struct TerminalWebSocketState {
    pub ide: Arc<SuperIDE>,
    pub terminal_manager: Arc<RwLock<TerminalManager>>,
}

// WebSocket handler
pub async fn terminal_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<TerminalWebSocketState>,
) -> impl IntoResponse {
    info!("New terminal WebSocket connection request");
    
    ws.on_upgrade(move |socket| terminal_websocket_connection(socket, state))
}

// WebSocket connection handler
pub async fn terminal_websocket_connection(
    socket: WebSocket,
    state: TerminalWebSocketState,
) {
    info!("Terminal WebSocket connection established");
    
    let (mut sender, mut receiver) = socket.split();
    let session_id = Uuid::new_v4().to_string();
    
    // Store active sessions
    let mut active_sessions = std::collections::HashMap::new();
    let mut command_history = std::collections::HashMap::new();
    
    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                if let Ok(client_message) = serde_json::from_str::<TerminalMessage>(&text) {
                    let result = handle_terminal_message(
                        client_message,
                        &state,
                        &mut active_sessions,
                        &mut command_history,
                        &session_id,
                    ).await;
                    
                    if let Err(e) = result {
                        let error_message = e.to_string();
                        error!("Error handling terminal message: {}", error_message);
                        
                        // For now, just log the error without sending to WebSocket
                        // This avoids the complex Send trait issue in WebSocket context
                        // TODO: Implement proper error reporting in a future update
                    }
                } else {
                    warn!("Failed to parse terminal message: {}", text);
                    let error_msg = ServerMessage::Error {
                        message: "Invalid message format".to_string(),
                        code: "INVALID_MESSAGE".to_string(),
                        session_id: None,
                    };
                    
                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = sender.send(axum::extract::ws::Message::Text(json)).await;
                    }
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                info!("Terminal WebSocket connection closed by client");
                break;
            }
            Err(e) => {
                error!("Terminal WebSocket error: {}", e);
                break;
            }
            _ => {
                // Handle other message types (Binary, Pong, etc.)
            }
        }
    }
    
    // Cleanup: Close all active sessions
    info!("Cleaning up terminal sessions for connection: {}", session_id);
    let terminal_manager = state.terminal_manager.write().await;
    
    for (session_id, _) in active_sessions {
        if let Err(e) = terminal_manager.close_session(&session_id).await {
            error!("Failed to close terminal session {}: {}", session_id, e);
        }
    }
    
    info!("Terminal WebSocket connection closed");
}

// Handle terminal messages
async fn handle_terminal_message(
    message: TerminalMessage,
    state: &TerminalWebSocketState,
    active_sessions: &mut std::collections::HashMap<String, mpsc::UnboundedReceiver<String>>,
    command_history: &mut std::collections::HashMap<String, Vec<String>>,
    _connection_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match message {
        TerminalMessage::CreateSession { session_id, shell: _, cwd: _ } => {
            let actual_session_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
            info!("Creating terminal session: {}", actual_session_id);
            
            let terminal_manager = state.terminal_manager.write().await;
            
            match terminal_manager.create_session(Some(format!("Terminal {}", actual_session_id))).await {
                Ok(_session_id) => {
                    // Get output receiver for this session
                    let receiver = terminal_manager.get_output_receiver(&actual_session_id).await
                        .unwrap_or_else(|| mpsc::unbounded_channel::<String>().1);
                    active_sessions.insert(actual_session_id.clone(), receiver);
                    
                    // Initialize command history
                    command_history.entry(actual_session_id.clone()).or_insert_with(Vec::new);
                    
                    info!("Successfully created terminal session: {}", actual_session_id);
                    
                    let response = ServerMessage::SessionCreated {
                        session_id: actual_session_id,
                        success: true,
                        message: None,
                    };
                    
                    return send_message_to_client(response, state).await;
                }
                Err(e) => {
                    error!("Failed to create terminal session: {}", e);
                    
                    let response = ServerMessage::SessionCreated {
                        session_id: actual_session_id,
                        success: false,
                        message: Some(e.to_string()),
                    };
                    
                    return send_message_to_client(response, state).await;
                }
            }
        }
        
        TerminalMessage::ExecuteCommand { session_id, command, cwd } => {
            info!("Executing command '{}' in session {}", command, session_id);
            
            let terminal_manager = state.terminal_manager.write().await;
            
            match terminal_manager.execute_command(&session_id, &command, cwd.as_deref()).await {
                Ok(_) => {
                    // Add to command history
                    command_history.entry(session_id.clone())
                        .or_insert_with(Vec::new)
                        .push(command.clone());
                    
                    info!("Command executed successfully in session {}", session_id);
                }
                Err(e) => {
                    error!("Failed to execute command '{}' in session {}: {}", command, session_id, e);
                    
                    let error_msg = ServerMessage::Error {
                        message: format!("Failed to execute command: {}", e),
                        code: "COMMAND_FAILED".to_string(),
                        session_id: Some(session_id),
                    };
                    
                    return send_message_to_client(error_msg, state).await;
                }
            }
        }
        
        TerminalMessage::Resize { session_id, width, height } => {
            info!("Resizing terminal session {} to {}x{}", session_id, width, height);
            
            let terminal_manager = state.terminal_manager.write().await;
            
            if let Err(e) = terminal_manager.resize_session(&session_id, width, height).await {
                error!("Failed to resize terminal session {}: {}", session_id, e);
                
                let error_msg = ServerMessage::Error {
                    message: format!("Failed to resize terminal: {}", e),
                    code: "RESIZE_FAILED".to_string(),
                    session_id: Some(session_id),
                };
                
                return send_message_to_client(error_msg, state).await;
            }
        }
        
        TerminalMessage::CloseSession { session_id } => {
            info!("Closing terminal session: {}", session_id);
            
            let terminal_manager = state.terminal_manager.write().await;
            
            match terminal_manager.close_session(&session_id).await {
                Ok(exit_code) => {
                    // Remove from active sessions
                    active_sessions.remove(&session_id);
                    command_history.remove(&session_id);
                    
                    info!("Successfully closed terminal session: {} (exit code: {:?})", session_id, exit_code);
                    
                    let response = ServerMessage::SessionClosed {
                        session_id,
                        exit_code,
                    };
                    
                    return send_message_to_client(response, state).await;
                }
                Err(e) => {
                    error!("Failed to close terminal session {}: {}", session_id, e);
                    
                    let error_msg = ServerMessage::Error {
                        message: format!("Failed to close session: {}", e),
                        code: "CLOSE_FAILED".to_string(),
                        session_id: Some(session_id),
                    };
                    
                    return send_message_to_client(error_msg, state).await;
                }
            }
        }
        
        TerminalMessage::GetHistory { session_id } => {
            info!("Getting command history for session: {}", session_id);
            
            let history = command_history.get(&session_id)
                .cloned()
                .unwrap_or_else(Vec::new);
            
            let response = ServerMessage::History {
                session_id,
                commands: history,
            };
            
            return send_message_to_client(response, state).await;
        }
    }
    
    Ok(())
}

// Helper function to send messages to the client
async fn send_message_to_client(
    message: ServerMessage,
    _state: &TerminalWebSocketState,
) -> Result<(), Box<dyn std::error::Error>> {
    // This is a placeholder - in a real implementation, you'd need access to the WebSocket sender
    // For now, we'll just log the message
    info!("Would send to client: {:?}", message);
    Ok(())
}

// Background task to forward terminal output to WebSocket clients
pub async fn forward_terminal_output(
    _terminal_manager: Arc<RwLock<TerminalManager>>,
    mut output_receiver: mpsc::UnboundedReceiver<(String, String, bool)>,
    session_id: String,
) {
    while let Some((output, timestamp, is_error)) = output_receiver.recv().await {
        let message = ServerMessage::CommandOutput {
            session_id: session_id.clone(),
            output,
            timestamp,
            is_error,
        };
        
        if let Ok(json) = serde_json::to_string(&message) {
            // In a real implementation, you would send this to the WebSocket client
            info!("Terminal output for session {}: {}", session_id, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_terminal_message_serialization() {
        let message = TerminalMessage::CreateSession {
            session_id: None,
            shell: Some("bash".to_string()),
            cwd: Some("/home/user".to_string()),
        };
        
        let json = serde_json::to_string(&message).unwrap();
        let parsed: TerminalMessage = serde_json::from_str(&json).unwrap();
        
        match parsed {
            TerminalMessage::CreateSession { session_id: _, shell, cwd } => {
                assert_eq!(shell, Some("bash".to_string()));
                assert_eq!(cwd, Some("/home/user".to_string()));
            }
            _ => panic!("Wrong message type parsed"),
        }
    }

    #[tokio::test]
    async fn test_server_message_serialization() {
        let message = ServerMessage::SessionCreated {
            session_id: "test-session".to_string(),
            success: true,
            message: None,
        };
        
        let json = serde_json::to_string(&message).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        
        match parsed {
            ServerMessage::SessionCreated { session_id, success, .. } => {
                assert_eq!(session_id, "test-session");
                assert_eq!(success, true);
            }
            _ => panic!("Wrong message type parsed"),
        }
    }
}