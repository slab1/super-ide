//! Main entry point for Super IDE - AI-Powered Development Environment

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super_ide::{
    initialize, SuperIDE, Configuration,
    ui::WebUI,
    utils::performance::global_performance_monitor,
    config::AIProvider,
};

use clap::{Parser, Subcommand, CommandFactory};
use anyhow::Result;

/// Command line arguments
#[derive(Parser, Clone)]
#[command(name = "super-ide")]
#[command(about = "Super IDE - AI-Powered Development Environment")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    /// Workspace directory path
    #[arg(short, long)]
    workspace: Option<PathBuf>,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Port for web interface
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
    
    /// Enable AI features
    #[arg(short, long, default_value_t = true)]
    ai: bool,
    
    /// AI provider (local, openai, anthropic)
    #[arg(short = 't', long, default_value = "local")]
    ai_provider: String,
    
    /// API key for cloud AI providers
    #[arg(short = 'k', long)]
    api_key: Option<String>,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Subcommands
#[derive(Subcommand, Clone)]
enum Commands {
    /// Create a new project
    New {
        /// Project name
        name: String,
        /// Project template
        template: Option<String>,
    },
    
    /// Import existing project
    Import {
        /// Project path
        path: PathBuf,
    },
    
    /// Show configuration
    Config,
    
    /// Generate completion file for shell
    Completions {
        /// Shell type (bash, zsh, fish)
        shell: String,
    },
    
    /// Run as headless server
    Server {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
        
        /// Bind address
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Setup logging
    setup_logging(args.debug)?;
    
    // Handle subcommands
    if let Some(ref command) = args.command {
        return handle_subcommand(command.clone(), args.clone()).await;
    }
    
    // Initialize configuration
    let mut config = load_configuration(&args).await?;
    
    // Apply command line overrides
    if let Some(workspace) = args.workspace {
        config.ide.workspace_path = workspace.to_string_lossy().to_string();
    }
    
    if let Some(api_key) = args.api_key {
        config.ai.api_key = Some(api_key);
    }
    
    config.ai.enable_local_inference = args.ai;
    config.ai.provider = parse_ai_provider(&args.ai_provider)?;
    
    // Save updated configuration
    config.save().await?;
    
    println!("🚀 Starting Super IDE v{}", env!("CARGO_PKG_VERSION"));
    println!("📁 Workspace: {}", config.ide.workspace_path);
    println!("🤖 AI Provider: {:?}", config.ai.provider);
    println!("🌐 Web UI: http://localhost:{}", args.port);
    
    // Initialize Super IDE
    let ide = initialize().await?;
    
    // Performance monitoring is automatically started with global instance
    let _monitor = global_performance_monitor();
    
    // Start web UI
    let mut web_ui = WebUI::new(Arc::new(ide));
    if let Err(e) = web_ui.start(args.port).await {
        eprintln!("Error starting web UI: {}", e);
        return Ok(());
    }
    
    println!("✅ Server running. Press Ctrl+C to stop.");
    
    // Handle graceful shutdown
    tokio::signal::ctrl_c().await?;
    println!("\\n🛑 Shutting down Super IDE...");
    
    if let Err(e) = web_ui.stop().await {
        eprintln!("Error stopping web UI: {}", e);
    }
    println!("✅ Super IDE stopped gracefully");
    
    Ok(())
}

/// Handle subcommands
async fn handle_subcommand(command: Commands, args: Args) -> Result<()> {
    match command {
        Commands::New { name, template } => {
            create_new_project(&name, template.as_deref(), &args).await
        },
        Commands::Import { path } => {
            import_project(&path, &args).await
        },
        Commands::Config => {
            show_configuration(&args).await
        },
        Commands::Completions { shell } => {
            generate_completions(&shell)
        },
        Commands::Server { port, bind } => {
            run_server(&args, port, &bind).await
        },
    }
}

/// Setup logging configuration
fn setup_logging(debug: bool) -> Result<()> {
    use env_logger::{Builder, Env};
    
    let env = Env::default()
        .filter_or("SUPER_IDE_LOG", if debug { "debug" } else { "info" });
    
    Builder::from_env(env)
        .format_timestamp_millis()
        .init();
    
    Ok(())
}

/// Load configuration with fallbacks
async fn load_configuration(args: &Args) -> Result<Configuration> {
    let mut config = if let Some(config_path) = &args.config {
        // Load from specified file
        let config = config::Config::builder()
            .add_source(config::File::with_name(&config_path.to_string_lossy()))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
        let config: Configuration = config.try_deserialize()
            .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))?;
        config
    } else {
        // Load default configuration
        Configuration::default()
    };
    
    // Apply environment overrides
    if let Ok(workspace) = std::env::var("SUPER_IDE_WORKSPACE") {
        config.ide.workspace_path = workspace;
    }
    
    if let Ok(api_key) = std::env::var("SUPER_IDE_API_KEY") {
        config.ai.api_key = Some(api_key);
    }
    
    Ok(config)
}

/// Parse AI provider from string
fn parse_ai_provider(provider: &str) -> Result<AIProvider> {
    match provider.to_lowercase().as_str() {
        "local" => Ok(AIProvider::Local),
        "openai" => Ok(AIProvider::OpenAI),
        "anthropic" => Ok(AIProvider::Anthropic),
        _ => anyhow::bail!("Invalid AI provider: {}. Use 'local', 'openai', or 'anthropic'", provider),
    }
}

/// Create a new project
async fn create_new_project(name: &str, template: Option<&str>, args: &Args) -> Result<()> {
    let workspace = args.workspace.as_ref().unwrap_or(&PathBuf::from(".")).clone();
    let project_path = workspace.join(name);
    
    println!("📁 Creating new project '{}' at {}", name, project_path.display());
    
    // Create project directory
    std::fs::create_dir_all(&project_path)?;
    
    // Determine template
    let template = template.unwrap_or("rust");
    
    match template {
        "rust" => create_rust_project(&project_path, name).await?,
        "python" => create_python_project(&project_path, name).await?,
        "javascript" => create_js_project(&project_path, name).await?,
        _ => anyhow::bail!("Unknown template: {}", template),
    }
    
    println!("✅ Project '{}' created successfully!", name);
    println!("🚀 Run 'cd {}' and then 'super-ide' to start coding with AI!", name);
    
    Ok(())
}

/// Create Rust project template
async fn create_rust_project(project_path: &PathBuf, name: &str) -> Result<()> {
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
# Add your dependencies here

[dev-dependencies]
# Add your test dependencies here
"#, name);
    
    let main_rs = format!(r#"//! {}

// This is your main.rs file
// Super IDE will provide AI assistance for this code

fn main() {{
    println!("Hello, {{}}!", "{}");
    
    // Your code goes here
    // The AI will help you with:
    // - Code completion and suggestions
    // - Bug detection and fixes
    // - Performance optimization
    // - Test generation
    // - Documentation
}}

/// Calculate the factorial of a number
/// 
/// # Examples
/// 
/// ```
/// assert_eq!(factorial(5), 120);
/// ```
fn factorial(n: u64) -> u64 {{
    if n <= 1 {{
        1
    }} else {{
        n * factorial(n - 1)
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_factorial() {{
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
    }}
}}
"#, name, name);
    
    let readme = format!(r#"# {}

A Rust project created with Super IDE.

## Features

- 🧠 AI-powered code assistance
- 🔍 Smart debugging and error detection
- 📝 Automatic documentation generation
- 🧪 Unit test generation
- ⚡ Performance optimization suggestions

## Getting Started

1. Install Super IDE: `cargo install super-ide`
2. Run the project: `cargo run`
3. Open Super IDE in your browser and start coding!

## AI Features

Super IDE provides:
- Real-time code analysis
- Intelligent code completion
- Bug prediction and prevention
- Code refactoring suggestions
- Test case generation
- Documentation automation

## Usage

```bash
# Run the application
cargo run

# Run tests
cargo test

# Build for release
cargo build --release
```

Happy coding with AI assistance! 🚀
"#, name);
    
    std::fs::write(project_path.join("Cargo.toml"), cargo_toml)?;
    std::fs::write(project_path.join("src").join("main.rs"), main_rs)?;
    std::fs::write(project_path.join("README.md"), readme)?;
    
    // Create .gitignore
    let gitignore = r#"# Generated by Cargo
/target/

# IDE files
.super-ide/
.vscode/
.idea/

# OS files
.DS_Store
Thumbs.db

# Logs
*.log

# Environment variables
.env
.env.local
"#;
    
    std::fs::write(project_path.join(".gitignore"), gitignore)?;
    
    Ok(())
}

/// Create Python project template
async fn create_python_project(project_path: &PathBuf, name: &str) -> Result<()> {
    let setup_py = format!(r#"from setuptools import setup, find_packages

setup(
    name="{}",
    version="0.1.0",
    description="A Python project created with Super IDE",
    author="",
    packages=find_packages(),
    install_requires=[],
    python_requires=">=3.8",
)
"#, name);
    
    let main_py = format!(r#"#!/usr/bin/env python3
\"\"\"
{}

Super IDE - AI-Powered Python Development
\"\"\"

def main():
    print("Hello, {}!")
    
    # Your code goes here
    # Super IDE AI features:
    # - Code completion and suggestions
    # - Bug detection and fixes
    # - Performance optimization
    # - Test generation
    # - Documentation

if __name__ == "__main__":
    main()
"#, name, name);
    
    let readme = format!(r#"# {}

A Python project created with Super IDE.

## Features

- 🧠 AI-powered code assistance
- 🔍 Smart debugging and error detection
- 📝 Automatic documentation generation
- 🧪 Unit test generation
- ⚡ Performance optimization suggestions

## Installation

1. Install Super IDE
2. Open this directory in Super IDE
3. Start coding with AI assistance!

## Usage

```bash
python {}.py
```

Happy coding with AI assistance! 🚀
"#, name, name);
    
    std::fs::create_dir_all(project_path.join(name))?;
    std::fs::write(project_path.join("setup.py"), setup_py)?;
    std::fs::write(project_path.join("__init__.py"), "")?;
    std::fs::write(project_path.join(name).join("__init__.py"), "")?;
    std::fs::write(project_path.join(name).join("main.py"), main_py)?;
    std::fs::write(project_path.join("README.md"), readme)?;
    
    Ok(())
}

/// Create JavaScript project template
async fn create_js_project(project_path: &PathBuf, name: &str) -> Result<()> {
    let package_json = format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "description": "A JavaScript project created with Super IDE",
  "main": "index.js",
  "scripts": {{
    "start": "node index.js",
    "dev": "node index.js",
    "test": "echo \\"Error: no test specified\\" && exit 1"
  }},
  "keywords": ["super-ide", "ai-assisted"],
  "author": "",
  "license": "MIT",
  "dependencies": {{}}
}}"#, name);
    
    let index_js = format!(r#"#!/usr/bin/env node

/**
 * {}
 * 
 * Super IDE - AI-Powered JavaScript Development
 */

function main() {{
    console.log("Hello, {}!");
    
    // Your code goes here
    // Super IDE AI features:
    // - Code completion and suggestions
    // - Bug detection and fixes
    // - Performance optimization
    // - Test generation
    // - Documentation
}}

// ES6 Module support
export default main;

// CommonJS support
if (typeof module !== 'undefined' && module.exports) {{
    module.exports = main;
}}

// Run if executed directly
if (typeof require !== 'undefined' && require.main === module) {{
    main();
}}
"#, name, name);
    
    let readme = format!(r#"# {}

A JavaScript project created with Super IDE.

## Features

- 🧠 AI-powered code assistance
- 🔍 Smart debugging and error detection
- 📝 Automatic documentation generation
- 🧪 Unit test generation
- ⚡ Performance optimization suggestions

## Usage

```bash
npm start
# or
node index.js
```

Happy coding with AI assistance! 🚀
"#, name);
    
    std::fs::write(project_path.join("package.json"), package_json)?;
    std::fs::write(project_path.join("index.js"), index_js)?;
    std::fs::write(project_path.join("README.md"), readme)?;
    
    Ok(())
}

/// Import existing project
async fn import_project(path: &PathBuf, args: &Args) -> Result<()> {
    println!("📂 Importing project from {}", path.display());
    
    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }
    
    // Detect project type and create configuration
    let project_type = detect_project_type(path)?;
    
    println!("✅ Detected project type: {}", project_type);
    println!("📁 Project imported successfully!");
    
    Ok(())
}

/// Detect project type from directory structure
fn detect_project_type(path: &PathBuf) -> Result<String> {
    let entries = std::fs::read_dir(path)?;
    
    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            
            match name_str.as_ref() {
                "Cargo.toml" => return Ok("Rust".to_string()),
                "package.json" => return Ok("JavaScript".to_string()),
                "setup.py" | "pyproject.toml" => return Ok("Python".to_string()),
                "pom.xml" => return Ok("Java (Maven)".to_string()),
                "build.gradle" => return Ok("Java (Gradle)".to_string()),
                "go.mod" => return Ok("Go".to_string()),
                _ => {}
            }
        }
    }
    
    Ok("Unknown".to_string())
}

/// Show current configuration
async fn show_configuration(args: &Args) -> Result<()> {
    let config = load_configuration(args).await?;
    
    println!("📋 Super IDE Configuration:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Workspace: {}", config.ide.workspace_path);
    println!("AI Provider: {:?}", config.ai.provider);
    println!("Auto-save: {} seconds", config.ide.auto_save_interval);
    println!("Font size: {}px", config.editor.font_size);
    println!("Theme: {}", config.theme.name);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}

/// Generate shell completions
fn generate_completions(shell: &str) -> Result<()> {
    use clap_complete::{generate, shells::*, Generator};
    
    let mut cmd = Args::command();
    match shell.to_lowercase().as_str() {
        "bash" => {
            let generator = Bash;
            generate(generator, &mut cmd, "super-ide", &mut std::io::stdout());
        },
        "zsh" => {
            let generator = Zsh;
            generate(generator, &mut cmd, "super-ide", &mut std::io::stdout());
        },
        "fish" => {
            let generator = Fish;
            generate(generator, &mut cmd, "super-ide", &mut std::io::stdout());
        },
        "powershell" => {
            let generator = PowerShell;
            generate(generator, &mut cmd, "super-ide", &mut std::io::stdout());
        },
        _ => {
            anyhow::bail!("Unsupported shell: {}. Use bash, zsh, fish, or powershell", shell);
        }
    };
    
    Ok(())
}

/// Run as headless server
async fn run_server(args: &Args, port: u16, bind: &str) -> Result<()> {
    println!("🖥️ Starting Super IDE server on {}:{}", bind, port);
    
    // Initialize Super IDE in server mode
    let config = load_configuration(args).await?;
    let ide = initialize().await?;
    
    // Start web UI
    let mut web_ui = WebUI::new(Arc::new(ide));
    if let Err(e) = web_ui.start(port).await {
        eprintln!("Error starting web UI: {}", e);
        return Ok(());
    }
    
    println!("✅ Server running. Press Ctrl+C to stop.");
    
    // Keep server running
    tokio::signal::ctrl_c().await?;
    
    Ok(())
}