//! External integrations module
//!
//! This module provides integration with external Python-based components:
//! - MCP (Model Context Protocol) API for accessing various data sources
//! - Browser automation and error capture capabilities

pub mod api;
pub mod browser;

use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use log::{info, error};

/// Configuration for external integrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalConfig {
    /// Path to Python executable
    pub python_path: String,
    /// Path to external_api directory
    pub external_api_path: String,
    /// Path to browser directory
    pub browser_path: String,
    /// MCP server port
    pub mcp_server_port: u16,
    /// Browser debug port
    pub browser_debug_port: u16,
    /// Request timeout in seconds
    pub request_timeout: u64,
}

impl Default for ExternalConfig {
    fn default() -> Self {
        Self {
            python_path: "python".to_string(),
            external_api_path: "./external_api".to_string(),
            browser_path: "./browser".to_string(),
            mcp_server_port: 12306,
            browser_debug_port: 9222,
            request_timeout: 30,
        }
    }
}

/// Result type for external operations
pub type ExternalResult<T> = Result<T, ExternalError>;

/// Error type for external operations
#[derive(Debug, thiserror::Error)]
pub enum ExternalError {
    #[error("Python execution failed: {0}")]
    PythonError(String),

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("JSON parsing failed: {0}")]
    JsonError(String),

    #[error("Browser operation failed: {0}")]
    BrowserError(String),

    #[error("MCP operation failed: {0}")]
    McpError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Process spawn failed: {0}")]
    ProcessError(String),
}

/// Manager for external integrations
pub struct ExternalManager {
    config: ExternalConfig,
    mcp_server_process: Arc<Mutex<Option<std::process::Child>>>,
    browser_process: Arc<Mutex<Option<std::process::Child>>>,
}

impl ExternalManager {
    /// Create a new external manager
    pub fn new(config: ExternalConfig) -> Self {
        Self {
            config,
            mcp_server_process: Arc::new(Mutex::new(None)),
            browser_process: Arc::new(Mutex::new(None)),
        }
    }

    /// Start MCP server
    pub async fn start_mcp_server(&self) -> ExternalResult<()> {
        info!("Starting MCP server on port {}", self.config.mcp_server_port);

        // Set environment variables for MCP server
        let mut command = Command::new(&self.config.python_path);
        command
            .arg("-m")
            .arg("external_api")
            .env("FUNC_SERVER_PORT", self.config.mcp_server_port.to_string())
            .env("AGENT_NAME", "super-ide")
            .current_dir(&self.config.external_api_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        match command.spawn() {
            Ok(child) => {
                let mut process_lock = self.mcp_server_process.lock().await;
                *process_lock = Some(child);
                info!("MCP server started successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to start MCP server: {}", e);
                Err(ExternalError::ProcessError(e.to_string()))
            }
        }
    }

    /// Stop MCP server
    pub async fn stop_mcp_server(&self) -> ExternalResult<()> {
        let mut process_lock = self.mcp_server_process.lock().await;
        if let Some(mut child) = process_lock.take() {
            match child.kill() {
                Ok(_) => {
                    let _ = child.wait();
                    info!("MCP server stopped successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to stop MCP server: {}", e);
                    Err(ExternalError::ProcessError(e.to_string()))
                }
            }
        } else {
            Ok(())
        }
    }

    /// Start browser automation
    pub async fn start_browser(&self) -> ExternalResult<()> {
        info!("Starting browser automation on port {}", self.config.browser_debug_port);

        let mut command = Command::new(&self.config.python_path);
        command
            .arg("global_browser.py")
            .env("BEDROCK_PROJECT", "")
            .current_dir(&self.config.browser_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        match command.spawn() {
            Ok(child) => {
                let mut process_lock = self.browser_process.lock().await;
                *process_lock = Some(child);
                info!("Browser automation started successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to start browser automation: {}", e);
                Err(ExternalError::ProcessError(e.to_string()))
            }
        }
    }

    /// Stop browser automation
    pub async fn stop_browser(&self) -> ExternalResult<()> {
        let mut process_lock = self.browser_process.lock().await;
        if let Some(mut child) = process_lock.take() {
            match child.kill() {
                Ok(_) => {
                    let _ = child.wait();
                    info!("Browser automation stopped successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to stop browser automation: {}", e);
                    Err(ExternalError::ProcessError(e.to_string()))
                }
            }
        } else {
            Ok(())
        }
    }

    /// Check if MCP server is running
    pub async fn is_mcp_server_running(&self) -> bool {
        let mut process_lock = self.mcp_server_process.lock().await;
        if let Some(child) = process_lock.as_mut() {
            matches!(child.try_wait(), Ok(None))
        } else {
            false
        }
    }

    /// Check if browser is running
    pub async fn is_browser_running(&self) -> bool {
        let mut process_lock = self.browser_process.lock().await;
        if let Some(child) = process_lock.as_mut() {
            matches!(child.try_wait(), Ok(None))
        } else {
            false
        }
    }

    /// Get configuration
    pub fn config(&self) -> &ExternalConfig {
        &self.config
    }
}

impl Drop for ExternalManager {
    fn drop(&mut self) {
        // Note: We can't use async operations in Drop, so we use blocking operations
        // In a real implementation, you'd want to handle cleanup differently
        if let Ok(mut process_lock) = self.mcp_server_process.try_lock() {
            if let Some(mut child) = process_lock.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }

        if let Ok(mut process_lock) = self.browser_process.try_lock() {
            if let Some(mut child) = process_lock.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}
