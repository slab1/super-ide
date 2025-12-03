//! Test file to verify terminal functionality

use super_ide::terminal::{TerminalManager, TerminalConfig, CommandExecutor};

#[tokio::test]
async fn test_terminal_creation() {
    let config = TerminalConfig::default();
    let manager = TerminalManager::new(config);
    
    // Test creating a session
    let session_id = manager.create_session(Some("Test Terminal".to_string())).await;
    assert!(session_id.is_ok());
    
    // Test listing sessions
    let sessions = manager.list_sessions().await;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].title, "Test Terminal");
}

#[tokio::test]
async fn test_command_execution() {
    let executor = CommandExecutor::default();
    
    // Test simple echo command
    let result = executor.execute("echo 'Hello, World!'").await;
    assert!(result.is_ok());
    
    let output = result.unwrap();
    assert!(output.exit_code == 0);
    assert!(output.stdout.contains("Hello, World!"));
}