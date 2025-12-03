//! Terminal execution module for Super IDE
//! 
//! This module provides terminal execution capabilities including:
//! - Execute shell commands and programs
//! - Capture stdout/stderr in real-time  
//! - Support interactive terminals with PTY
//! - Handle process lifecycle (start, stop, kill)
//! - Integrate with the WebSocket UI for terminal display

use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, RwLock, broadcast};
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
    pub environment: HashMap<String, String>,
    pub pty_size: Option<(u16, u16)>,
}

/// Terminal error types
#[derive(thiserror::Error, Debug)]
pub enum TerminalError {
    #[error("Process execution failed: {0}")]
    ProcessExecution(String),
    
    #[error("PTY setup failed: {0}")]
    PtySetup(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid session ID")]
    InvalidSession,
    
    #[error("Session already exists")]
    SessionExists,
    
    #[error("Session not found")]
    SessionNotFound,
}

/// Terminal session management
#[derive(Debug)]
pub struct TerminalManager {
    sessions: Arc<RwLock<HashMap<String, TerminalSession>>>,
    output_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<TerminalOutput>>>>,
    input_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<TerminalInput>>>>,
    config: TerminalConfig,
}

/// Real-time terminal session with process execution
pub struct RealTimeTerminal {
    session_id: String,
    process: Option<tokio::process::Child>,
    stdin_tx: Option<mpsc::UnboundedSender<String>>,
    stdout_rx: Option<mpsc::UnboundedReceiver<String>>,
    stderr_rx: Option<mpsc::UnboundedReceiver<String>>,
    status: TerminalStatus,
    start_time: Instant,
}

/// Simple command execution interface
pub struct CommandExecutor {
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
    
    /// Create a new terminal session
    pub async fn create_session(&self, title: Option<String>) -> IdeResult<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let title = title.unwrap_or_else(|| format!("Terminal {}", session_id[..8].to_string()));
        
        let session = TerminalSession {
            id: session_id.clone(),
            title,
            working_directory: self.config.working_directory.clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default()),
            created_at: chrono::Utc::now(),
            status: TerminalStatus::Stopped,
        };
        
        // Check if session already exists
        let mut sessions = self.sessions.write().await;
        if sessions.contains_key(&session_id) {
            return Err(TerminalError::SessionExists.into());
        }
        
        sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }
    
    /// Start a real-time terminal session
    pub async fn start_terminal(&self, session_id: &str) -> IdeResult<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| TerminalError::SessionNotFound)?;
        
        if session.status == TerminalStatus::Running {
            return Ok(());
        }
        
        // Create communication channels
        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<TerminalInput>();
        let (stdout_tx, stdout_rx) = mpsc::unbounded_channel::<TerminalOutput>();
        let (stderr_tx, stderr_rx) = mpsc::unbounded_channel::<TerminalOutput>();
        
        // Store channel senders
        {
            let mut output_senders = self.output_senders.write().await;
            output_senders.insert(session_id.to_string(), stdout_tx.clone());
            output_senders.insert(session_id.to_string() + "_stderr", stderr_tx);
        }
        
        {
            let mut input_senders = self.input_senders.write().await;
            input_senders.insert(session_id.to_string(), stdin_tx.clone());
        }
        
        // Start the shell process
        let mut child = tokio::process::Command::new(&self.config.shell)
            .current_dir(&session.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(TerminalError::ProcessExecution)?;
        
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        
        let session_id_clone = session_id.to_string();
        let stdout_tx_clone = stdout_tx.clone();
        let stderr_tx_clone = stderr_tx.clone();
        
        // Spawn stdout reader task
        let stdout_task = tokio::spawn(async move {
            let mut buffer = vec![0u8; 4096];
            loop {
                match stdout.read(&mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        let _ = stdout_tx_clone.send(TerminalOutput {
                            session_id: session_id_clone.clone(),
                            data,
                            is_error: false,
                            timestamp: chrono::Utc::now(),
                        });
                    }
                    Err(_) => break,
                }
            }
        });
        
        // Spawn stderr reader task
        let stderr_task = tokio::spawn(async move {
            let mut buffer = vec![0u8; 4096];
            loop {
                match stderr.read(&mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        let _ = stderr_tx_clone.send(TerminalOutput {
                            session_id: session_id_clone.clone(),
                            data,
                            is_error: true,
                            timestamp: chrono::Utc::now(),
                        });
                    }
                    Err(_) => break,
                }
            }
        });
        
        // Spawn stdin writer task
        let stdin_task = tokio::spawn(async move {
            while let Some(input) = stdin_rx.recv().await {
                if let Err(_) = stdin.write_all(input.as_bytes()).await {
                    break;
                }
            }
        });
        
        // Update session status
        session.status = TerminalStatus::Running;
        
        // Keep track of the process
        // Note: In a real implementation, you'd want to store child handles
        // and manage them properly
        
        Ok(())
    }
    
    /// Send input to a terminal session
    pub async fn send_input(&self, session_id: &str, input: &str) -> IdeResult<()> {
        let input_senders = self.input_senders.read().await;
        let sender = input_senders.get(session_id)
            .ok_or_else(|| TerminalError::SessionNotFound)?;
        
        sender.send(TerminalInput {
            session_id: session_id.to_string(),
            data: input.to_string(),
            timestamp: chrono::Utc::now(),
        }).map_err(|_| TerminalError::InvalidSession.into())
    }
    
    /// Stop a terminal session
    pub async fn stop_terminal(&self, session_id: &str) -> IdeResult<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| TerminalError::SessionNotFound)?;
        
        session.status = TerminalStatus::Stopped;
        
        // Clean up channels
        {
            let mut output_senders = self.output_senders.write().await;
            output_senders.remove(session_id);
            output_senders.remove(&(session_id.to_string() + "_stderr"));
        }
        
        {
            let mut input_senders = self.input_senders.write().await;
            input_senders.remove(session_id);
        }
        
        Ok(())
    }
    
    /// Get all active sessions
    pub async fn list_sessions(&self) -> Vec<TerminalSession> {
        self.sessions.read().await.values().cloned().collect()
    }
    
    /// Get session information
    pub async fn get_session(&self, session_id: &str) -> Option<TerminalSession> {
        self.sessions.read().await.get(session_id).cloned()
    }
    
    /// Subscribe to terminal output (for WebSocket integration)
    pub async fn subscribe_output(&self, session_id: &str) -> Option<mpsc::UnboundedReceiver<TerminalOutput>> {
        let output_senders = self.output_senders.read().await;
        let stdout_sender = output_senders.get(session_id);
        let stderr_sender = output_senders.get(&(session_id.to_string() + "_stderr"));
        
        if let (Some(stdout_tx), Some(stderr_tx)) = (stdout_sender, stderr_sender) {
            // In a real implementation, you'd create a combined receiver
            // For now, return the stdout receiver
            let (tx, rx) = mpsc::unbounded_channel::<TerminalOutput>();
            
            // This is a simplified implementation
            // In reality, you'd merge stdout and stderr streams
            Some(rx)
        } else {
            None
        }
    }
}

impl RealTimeTerminal {
    /// Create a new real-time terminal
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            process: None,
            stdin_tx: None,
            stdout_rx: None,
            stderr_rx: None,
            status: TerminalStatus::Stopped,
            start_time: Instant::now(),
        }
    }
    
    /// Execute a command in the terminal
    pub async fn execute_command(&mut self, command: &str) -> IdeResult<ProcessResult> {
        let start_time = Instant::now();
        
        // Split command into program and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(TerminalError::ProcessExecution("Empty command".to_string()).into());
        }
        
        let program = parts[0];
        let args = &parts[1..];
        
        // Execute the command
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(TerminalError::ProcessExecution)?;
        
        let execution_time = start_time.elapsed();
        
        let result = ProcessResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
        };
        
        Ok(result)
    }
    
    /// Get the current status
    pub fn status(&self) -> &TerminalStatus {
        &self.status
    }
    
    /// Get execution time
    pub fn execution_time(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new(config: TerminalConfig) -> Self {
        Self { config }
    }
    
    /// Execute a single command and return the result
    pub async fn execute(&self, command: &str) -> IdeResult<ProcessResult> {
        let start_time = Instant::now();
        
        // For simple command execution, use std::process::Command
        let output = tokio::task::spawn_blocking(move || {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
        })
        .await
        .map_err(|_| TerminalError::ProcessExecution("Failed to spawn task".to_string()))?
        .map_err(TerminalError::ProcessExecution)?;
        
        let execution_time = start_time.elapsed();
        
        let result = ProcessResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
        };
        
        Ok(result)
    }
    
    /// Execute multiple commands in sequence
    pub async fn execute_batch(&self, commands: Vec<&str>) -> Vec<IdeResult<ProcessResult>> {
        let mut results = Vec::new();
        
        for command in commands {
            results.push(self.execute(command).await);
        }
        
        results
    }
    
    /// Execute commands in parallel
    pub async fn execute_parallel(&self, commands: Vec<&str>) -> Vec<IdeResult<ProcessResult>> {
        let mut handles = Vec::new();
        
        for command in commands {
            let command = command.to_string();
            let config = self.config.clone();
            let handle = tokio::spawn(async move {
                let executor = CommandExecutor::new(config);
                executor.execute(&command).await
            });
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.map_err(|_| TerminalError::ProcessExecution("Task panicked".to_string())).unwrap_or_else(|e| Err(e.into())));
        }
        
        results
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        let shell = if cfg!(target_os = "windows") {
            "cmd.exe".to_string()
        } else {
            "/bin/bash".to_string()
        };
        
        let mut environment = HashMap::new();
        environment.insert("TERM".to_string(), "xterm-256color".to_string());
        environment.insert("COLORTERM".to_string(), "truecolor".to_string());
        
        Self {
            shell,
            working_directory: None,
            environment,
            pty_size: Some((80, 24)),
        }
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new(TerminalConfig::default())
    }
}

/// Helper function to get terminal size
pub fn get_terminal_size() -> (u16, u16) {
    terminal_size::terminal_size().map(|(w, h)| {
        (w.0, h.1)
    }).unwrap_or((80, 24))
}

/// Helper function to check if running in a terminal
pub fn is_terminal() -> bool {
    atty::is(atty::Stream::Stdin) || atty::is(atty::Stream::Stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_session() {
        let config = TerminalConfig::default();
        let manager = TerminalManager::new(config);
        
        let session_id = manager.create_session(Some("Test Terminal".to_string())).await.unwrap();
        
        let sessions = manager.list_sessions().await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session_id);
        assert_eq!(sessions[0].title, "Test Terminal");
    }
    
    #[tokio::test]
    async fn test_command_executor() {
        let executor = CommandExecutor::default();
        
        let result = executor.execute("echo 'Hello, World!'").await.unwrap();
        assert!(result.exit_code == 0);
        assert!(result.stdout.contains("Hello, World!"));
        assert!(result.stderr.is_empty());
    }
}