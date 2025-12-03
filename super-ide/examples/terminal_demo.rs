use super_ide::core::SuperIDE;
use super_ide::terminal::{TerminalManager, CommandExecutor};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Super IDE Terminal Demo - Testing Core Functionality");
    println!("====================================================");

    // Initialize the SuperIDE with terminal support
    let ide = SuperIDE::new().await?;
    println!("✅ SuperIDE initialized successfully");

    // Test 1: Basic Command Execution (echo)
    println!("\n📝 Test 1: Basic Command Execution - echo");
    println!("----------------------------------------");
    
    match ide.execute_command("echo 'Hello World from SuperIDE Terminal!'", None).await {
        Ok(result) => {
            println!("Command: echo 'Hello World from SuperIDE Terminal!'");
            println!("Exit Code: {}", result.exit_code);
            println!("Output:");
            println!("{}", result.output);
            if !result.stderr.is_empty() {
                println!("Error Output:");
                println!("{}", result.stderr);
            }
        },
        Err(e) => println!("❌ Failed to execute echo command: {}", e),
    }

    sleep(Duration::from_millis(500)).await;

    // Test 2: Directory Listing (ls -la)
    println!("\n📁 Test 2: Directory Listing - ls -la");
    println!("--------------------------------------");
    
    match ide.execute_command("ls -la", Some("/workspace/super-ide".to_string())).await {
        Ok(result) => {
            println!("Command: ls -la");
            println!("Exit Code: {}", result.exit_code);
            println!("Output:");
            println!("{}", result.output);
            if !result.stderr.is_empty() {
                println!("Error Output:");
                println!("{}", result.stderr);
            }
        },
        Err(e) => println!("❌ Failed to execute ls command: {}", e),
    }

    sleep(Duration::from_millis(500)).await;

    // Test 3: Cargo Version Check
    println!("\n🦀 Test 3: Cargo Version Check");
    println!("-----------------------------");
    
    match ide.execute_command("cargo --version", None).await {
        Ok(result) => {
            println!("Command: cargo --version");
            println!("Exit Code: {}", result.exit_code);
            println!("Output:");
            println!("{}", result.output);
            if !result.stderr.is_empty() {
                println!("Error Output:");
                println!("{}", result.stderr);
            }
        },
        Err(e) => println!("❌ Failed to execute cargo command: {}", e),
    }

    sleep(Duration::from_millis(500)).await;

    // Test 4: Terminal Session Management
    println!("\n🖥️  Test 4: Terminal Session Management");
    println!("--------------------------------------");
    
    // Create a new terminal session
    match ide.create_terminal(Some("Demo Terminal".to_string())).await {
        Ok(session_id) => {
            println!("✅ Created terminal session: {}", session_id);
            
            // Start the terminal session
            match ide.start_terminal(&session_id).await {
                Ok(_) => {
                    println!("✅ Started terminal session: {}", session_id);
                    
                    // Send some input to the terminal
                    match ide.send_terminal_input(&session_id, "pwd\n").await {
                        Ok(_) => {
                            println!("✅ Sent 'pwd' command to terminal");
                            sleep(Duration::from_millis(1000)).await;
                        },
                        Err(e) => println!("❌ Failed to send input: {}", e),
                    }
                    
                    // Stop the terminal session
                    match ide.stop_terminal(&session_id).await {
                        Ok(_) => println!("✅ Stopped terminal session: {}", session_id),
                        Err(e) => println!("❌ Failed to stop terminal: {}", e),
                    }
                },
                Err(e) => println!("❌ Failed to start terminal: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to create terminal: {}", e),
    }

    sleep(Duration::from_millis(500)).await;

    // Test 5: Multiple Commands in Sequence
    println!("\n🔄 Test 5: Multiple Commands in Sequence");
    println!("---------------------------------------");
    
    let commands = vec![
        "whoami",
        "date",
        "uname -a",
        "rustc --version",
    ];

    for cmd in commands {
        match ide.execute_command(cmd, None).await {
            Ok(result) => {
                println!("\n--- Command: {} ---", cmd);
                println!("Exit Code: {}", result.exit_code);
                println!("Output: {}", result.output.trim());
                if !result.stderr.is_empty() {
                    println!("Error: {}", result.stderr.trim());
                }
            },
            Err(e) => println!("❌ Failed to execute '{}': {}", cmd, e),
        }
        sleep(Duration::from_millis(300)).await;
    }

    // Test 6: Error Handling - Non-existent Command
    println!("\n❌ Test 6: Error Handling - Non-existent Command");
    println!("-----------------------------------------------");
    
    match ide.execute_command("this-command-does-not-exist-12345", None).await {
        Ok(result) => {
            println!("Command: this-command-does-not-exist-12345");
            println!("Exit Code: {}", result.exit_code);
            println!("Output: {}", result.output);
            if !result.stderr.is_empty() {
                println!("Error Output:");
                println!("{}", result.stderr);
            }
        },
        Err(e) => println!("❌ Command failed as expected: {}", e),
    }

    println!("\n🎉 Terminal Demo Completed!");
    println!("==========================");
    println!("All core terminal functionality has been tested:");
    println!("✅ Basic command execution");
    println!("✅ Directory listing");
    println!("✅ Tool version checks");
    println!("✅ Terminal session management");
    println!("✅ Multiple command sequences");
    println!("✅ Error handling");
    
    Ok(())
}