//! Terminal execution module for Super IDE
//! 
//! This module provides terminal execution capabilities including:
//! - Execute shell commands and programs
//! - Capture stdout/stderr in real-time  
//! - Support interactive terminals with PTY
//! - Handle process lifecycle (start, stop, kill)
//! - Integrate with the WebSocket UI for terminal display

pub mod ws_handler;

use std::collections::HashMap;
use std::process::{Stdio};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{Duration, Instant};

use crate::core::IdeResult;

/// Terminal session information
#[derive(Debug, Clone)]
pub struct TerminalSession {
    pub id: String,
    pub title: String,
    pub working_directory: std::path::PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: TerminalStatus,
}

/// Handle for an active terminal session with WebSocket support
pub struct TerminalSessionHandle {
    pub session_id: String,
    pub output_receiver: mpsc::UnboundedReceiver<String>,
    _tasks: Vec<tokio::task::JoinHandle<()>>,
}

/// Terminal execution status
#[derive(Debug, Clone, PartialEq)]
pub enum TerminalStatus {
    Running,
    Stopped,
    Error,
    Exited(i32),
}

/// Terminal output event
#[derive(Debug, Clone)]
pub struct TerminalOutput {
    pub session_id: String,
    pub data: String,
    pub is_error: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Terminal input event
#[derive(Debug, Clone)]
pub struct TerminalInput {
    pub session_id: String,
    pub data: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Process execution result
#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
}

/// Terminal configuration
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    pub shell: String,
    pub working_directory: Option<std::path::PathBuf>,
    pub max_output_lines: usize,
    pub command_timeout: Duration,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: "bash".to_string(),
            working_directory: None,
            max_output_lines: 1000,
            command_timeout: Duration::from_secs(30),
        }
    }
}

/// Terminal execution errors
#[derive(Debug, thiserror::Error)]
pub enum TerminalError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("Session already exists")]
    SessionExists,
    
    #[error("Invalid session")]
    InvalidSession,
    
    #[error("Process execution error: {0}")]
    ProcessExecution(String),
    
    #[error("Timeout exceeded")]
    Timeout,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Terminal manager for handling multiple terminal sessions
#[derive(Debug)]
pub struct TerminalManager {
    sessions: Arc<RwLock<HashMap<String, TerminalSession>>>,
    output_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<String>>>>,
    input_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<String>>>>,
    config: TerminalConfig,
}

impl TerminalManager {
    /// Create a new terminal manager
    pub fn new(config: TerminalConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            output_senders: Arc::new(RwLock::new(HashMap::new())),
            input_senders: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Create a new terminal session with WebSocket support
    pub async fn create_session(&self, shell: Option<&str>, cwd: Option<&str>) -> IdeResult<TerminalSessionHandle> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let shell = shell.unwrap_or("bash");
        let working_directory = cwd.map(std::path::PathBuf::from)
            .unwrap_or_else(|| self.config.working_directory.clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default()));
        
        let session = TerminalSession {
            id: session_id.clone(),
            title: format!("Terminal {}", session_id[..8].to_string()),
            working_directory: working_directory.clone(),
            created_at: chrono::Utc::now(),
            status: TerminalStatus::Stopped,
        };
        
        // Check if session already exists
        let mut sessions = self.sessions.write().await;
        if sessions.contains_key(&session_id) {
            return Err(TerminalError::SessionExists.into());
        }
        
        sessions.insert(session_id.clone(), session.clone());
        
        // Create output channel for WebSocket communication
        let (output_tx, output_rx) = mpsc::unbounded_channel::<String>();
        
        // Store output sender for WebSocket forwarding
        {
            let mut output_senders = self.output_senders.write().await;
            output_senders.insert(session_id.clone(), output_tx);
        }
        
        // Create input channel for command processing
        let (input_tx, mut input_rx) = mpsc::unbounded_channel::<String>();
        
        {
            let mut input_senders = self.input_senders.write().await;
            input_senders.insert(session_id.clone(), input_tx);
        }
        
        // Start the shell process
        let mut child = tokio::process::Command::new(shell)
            .current_dir(&working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TerminalError::ProcessExecution(e.to_string()))?;
        
        let stdin = child.stdin.take()
            .ok_or_else(|| TerminalError::ProcessExecution("Failed to get stdin handle".to_string()))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| TerminalError::ProcessExecution("Failed to get stdout handle".to_string()))?;
        let stderr = child.stderr.take()
            .ok_or_else(|| TerminalError::ProcessExecution("Failed to get stderr handle".to_string()))?;
        
        // Spawn task to handle stdout
        let output_tx_clone = output_tx.clone();
        let stdout_task = tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stdout);
            let mut buffer = String::new();
            
            while let Ok(n) = reader.read_line(&mut buffer).await {
                if n == 0 {
                    break; // EOF
                }
                
                let output = buffer.trim_end().to_string();
                if !output.is_empty() {
                    let _ = output_tx_clone.send(output);
                }
                buffer.clear();
            }
        });
        
        // Spawn task to handle stderr
        let stderr_task = tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stderr);
            let mut buffer = String::new();
            
            while let Ok(n) = reader.read_line(&mut buffer).await {
                if n == 0 {
                    break; // EOF
                }
                
                let output = format!("[stderr] {}", buffer.trim_end());
                if !output.trim().is_empty() {
                    let _ = output_tx.send(output);
                }
                buffer.clear();
            }
        });
        
        // Spawn task to handle input
        let stdin_clone = stdin;
        let input_task = tokio::spawn(async move {
            while let Some(input) = input_rx.recv().await {
                let _ = stdin_clone.write_all(input.as_bytes()).await;
                let _ = stdin_clone.write_all(b"\n").await;
            }
        });
        
        // Update session status
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                session.status = TerminalStatus::Running;
            }
        }
        
        Ok(TerminalSessionHandle {
            session_id,
            output_receiver: output_rx,
            _tasks: vec![stdout_task, stderr_task, input_task],
        })
    }
    
    /// Execute a command in a terminal session
    pub async fn execute_command(&self, session_id: &str, command: &str, _cwd: Option<&str>) -> IdeResult<()> {
        let input_senders = self.input_senders.read().await;
        let input_sender = input_senders.get(session_id)
            .ok_or_else(|| TerminalError::SessionNotFound(session_id.to_string()))?;
        
        let _ = input_sender.send(command.to_string());
        Ok(())
    }
    
    /// Resize terminal session
    pub async fn resize_session(&self, session_id: &str, _width: u16, _height: u16) -> IdeResult<()> {
        // For now, just validate the session exists
        let sessions = self.sessions.read().await;
        if !sessions.contains_key(session_id) {
            return Err(TerminalError::SessionNotFound(session_id.to_string()).into());
        }
        Ok(())
    }
    
    /// Close a terminal session
    pub async fn close_session(&self, session_id: &str) -> IdeResult<Option<i32>> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = TerminalStatus::Stopped;
            
            // Clean up channels
            {
                let mut output_senders = self.output_senders.write().await;
                output_senders.remove(session_id);
            }
            
            {
                let mut input_senders = self.input_senders.write().await;
                input_senders.remove(session_id);
            }
            
            Ok(None) // We don't track exit codes in this simple implementation
        } else {
            Err(TerminalError::SessionNotFound(session_id.to_string()).into())
        }
    }
    
    /// Get all active sessions
    pub async fn list_sessions(&self) -> Vec<TerminalSession> {
        self.sessions.read().await.values().cloned().collect()
    }
    
    /// Get session information
    pub async fn get_session(&self, session_id: &str) -> Option<TerminalSession> {
        self.sessions.read().await.get(session_id).cloned()
    }
    
    /// Send input to a terminal session
    pub async fn send_input(&self, session_id: &str, input: &str) -> IdeResult<()> {
        let input_senders = self.input_senders.read().await;
        let sender = input_senders.get(session_id)
            .ok_or_else(|| TerminalError::SessionNotFound(session_id.to_string()))?;
        
        sender.send(input.to_string()).map_err(|_| TerminalError::InvalidSession.into())
    }
}

/// Simple command execution interface
pub struct CommandExecutor {
    config: TerminalConfig,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new(config: TerminalConfig) -> Self {
        Self { config }
    }
    
    /// Execute a command and return the result
    pub async fn execute(&mut self, command: &str) -> IdeResult<ProcessResult> {
        let start_time = tokio::time::Instant::now();

        // Split command into program and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(TerminalError::ProcessExecution("Empty command".to_string()).into());
        }

        let program = parts[0];
        let args = &parts[1..];

        // Spawn the process asynchronously
        let child = tokio::process::Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TerminalError::ProcessExecution(e.to_string()))?;

        let (stdout, stderr) = (
            child.stdout.ok_or_else(|| TerminalError::ProcessExecution("Failed to get stdout".to_string()))?,
            child.stderr.ok_or_else(|| TerminalError::ProcessExecution("Failed to get stderr".to_string()))?,
        );

        // Read stdout and stderr concurrently
        let stdout_task = tokio::spawn(async move {
            let mut buffer = String::new();
            let _ = tokio::io::AsyncReadExt::read_to_string(&mut stdout, &mut buffer).await;
            buffer
        });

        let stderr_task = tokio::spawn(async move {
            let mut buffer = String::new();
            let _ = tokio::io::AsyncReadExt::read_to_string(&mut stderr, &mut buffer).await;
            buffer
        });

        let execution_time = start_time.elapsed();

        // Wait for the process to complete
        let output = child.wait_with_output()
            .await
            .map_err(|e| TerminalError::ProcessExecution(e.to_string()))?;

        let stdout = stdout_task.await.unwrap_or_default();
        let stderr = stderr_task.await.unwrap_or_default();

        Ok(ProcessResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout,
            stderr,
            execution_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_terminal_manager_creation() {
        let config = TerminalConfig::default();
        let manager = TerminalManager::new(config);
        
        let sessions = manager.list_sessions().await;
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_command_executor() {
        let config = TerminalConfig::default();
        let mut executor = CommandExecutor::new(config);
        
        let result = executor.execute("echo 'Hello, World!'").await.unwrap();
        assert!(result.exit_code == 0);
        assert!(result.stdout.contains("Hello, World!"));
    }
}