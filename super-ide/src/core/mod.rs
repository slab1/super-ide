//! Core IDE functionality and main application state

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use anyhow::Result;
use thiserror::Error;

use crate::ai::{AiEngine, AiConfig};
use crate::editor::Editor;
use crate::config::Configuration;
use crate::utils::event_bus::EventBus;
use crate::terminal::{TerminalManager, TerminalConfig};

/// Main IDE result type
pub type IdeResult<T> = Result<T, IdeError>;

/// Main IDE errors
#[derive(Error, Debug)]
pub enum IdeError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("AI Engine error: {0}")]
    AiEngine(String),
    
    #[error("Editor error: {0}")]
    Editor(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Editor error: {0}")]
    EditorError(#[from] crate::editor::EditorError),
    
    #[error("Terminal error: {0}")]
    Terminal(#[from] crate::terminal::TerminalError),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] crate::config::ConfigError),
}

/// Main SuperIDE application state
#[derive(Clone, Debug)]
pub struct SuperIDE {
    /// Core configuration
    config: Arc<RwLock<Configuration>>,
    
    /// AI engine for code intelligence
    ai_engine: AiEngine,
    
    /// Code editor instance
    editor: Arc<Mutex<Editor>>,
    
    /// Event bus for inter-component communication
    event_bus: Arc<EventBus>,
    
    /// Terminal manager for command execution
    terminal_manager: Arc<TerminalManager>,
    
    /// Application state
    state: Arc<RwLock<IdeState>>,
}

/// IDE application state
#[derive(Debug, Clone)]
pub struct IdeState {
    /// Currently open projects
    pub projects: Vec<ProjectInfo>,
    
    /// Active editor tabs
    pub active_tabs: Vec<EditorTab>,
    
    /// User preferences and settings
    pub preferences: UserPreferences,
    
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Project information
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub path: std::path::PathBuf,
    pub language: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_opened: chrono::DateTime<chrono::Utc>,
    pub ai_context: AIContext,
}

/// Editor tab information
#[derive(Debug, Clone)]
pub struct EditorTab {
    pub id: String,
    pub title: String,
    pub file_path: std::path::PathBuf,
    pub is_modified: bool,
    pub language: String,
    pub cursor_position: (usize, usize),
}

/// User preferences
#[derive(Debug, Clone, Default)]
pub struct UserPreferences {
    pub theme: String,
    pub font_size: u32,
    pub auto_save: bool,
    pub ai_assistance: bool,
    pub collaboration: bool,
    pub keyboard_shortcuts: KeyboardShortcuts,
}

/// Keyboard shortcuts configuration
#[derive(Debug, Clone, Default)]
pub struct KeyboardShortcuts {
    pub save: Vec<String>,
    pub format: Vec<String>,
    pub run: Vec<String>,
    pub debug: Vec<String>,
    pub ai_suggestion: Vec<String>,
}

/// Performance metrics
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub active_files: usize,
    pub ai_requests: u64,
    pub collaboration_events: u64,
}

/// AI context for project
#[derive(Debug, Clone)]
pub struct AIContext {
    pub custom_instructions: Vec<String>,
    pub coding_style: CodingStyle,
    pub preferred_patterns: Vec<String>,
    pub recent_interactions: Vec<AIInteraction>,
}

/// Coding style preferences
#[derive(Debug, Clone)]
pub struct CodingStyle {
    pub naming_convention: String,
    pub indentation: String,
    pub line_length: usize,
    pub bracket_style: String,
}

/// AI interaction history
#[derive(Debug, Clone)]
pub struct AIInteraction {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub interaction_type: String,
    pub input: String,
    pub output: String,
    pub rating: Option<i32>,
}

impl SuperIDE {
    /// Create a new IDE instance
    pub async fn new(config: Configuration) -> IdeResult<Self> {
        let ai_engine = AiEngine::new(AiConfig::from(&config)).await.map_err(|e| IdeError::Editor(e.to_string()))?;
        let editor = Editor::new(&config, Arc::new(ai_engine.clone())).await.map_err(|e| IdeError::Editor(e.to_string()))?;
        let event_bus = EventBus::new();
        
        // Initialize terminal manager with default config
        let terminal_config = TerminalConfig {
            shell: if cfg!(target_os = "windows") {
                "cmd.exe".to_string()
            } else {
                "/bin/bash".to_string()
            },
            working_directory: None,
            environment: std::env::vars().collect(),
            pty_size: Some((80, 24)),
        };
        let terminal_manager = Arc::new(TerminalManager::new(terminal_config));
        
        let state = IdeState {
            projects: Vec::new(),
            active_tabs: Vec::new(),
            preferences: UserPreferences::default(),
            performance_metrics: PerformanceMetrics::default(),
        };
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            ai_engine,
            editor: Arc::new(Mutex::new(editor)),
            event_bus: Arc::new(event_bus),
            terminal_manager,
            state: Arc::new(RwLock::new(state)),
        })
    }
    
    /// Get AI engine reference
    pub fn ai_engine(&self) -> &AiEngine {
        &self.ai_engine
    }
    
    /// Get editor reference
    pub fn editor(&self) -> &Arc<Mutex<Editor>> {
        &self.editor
    }
    
    /// Get configuration reference
    pub fn config(&self) -> &Arc<RwLock<Configuration>> {
        &self.config
    }
    
    /// Get event bus reference
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }
    
    /// Get terminal manager reference
    pub fn terminal_manager(&self) -> &Arc<TerminalManager> {
        &self.terminal_manager
    }
    
    /// Get current IDE state
    pub async fn get_state(&self) -> IdeState {
        self.state.read().await.clone()
    }
    
    /// Update IDE state
    pub async fn update_state(&self, update: impl FnOnce(&mut IdeState)) {
        let mut state = self.state.write().await;
        update(&mut state);
    }
    
    /// Load a project and scan for files
    pub async fn load_project(&self, project_path: std::path::PathBuf) -> IdeResult<String> {
        let project_id = uuid::Uuid::new_v4().to_string();

        let project_info = ProjectInfo {
            id: project_id.clone(),
            name: project_path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("Unknown")
                .to_string(),
            path: project_path.clone(),
            language: self.detect_language(&project_path).await,
            created_at: chrono::Utc::now(),
            last_opened: chrono::Utc::now(),
            ai_context: AIContext::default(),
        };

        // Scan project directory for files
        let mut project_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&project_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            if let Some(file_name) = entry.file_name().to_str() {
                                project_files.push(file_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        self.update_state(|state| {
            state.projects.push(project_info);
        }).await;

        // Update config with workspace path
        {
            let mut config = self.config.write().await;
            config.ide.workspace_path = project_path.to_string_lossy().to_string();
        }

        log::info!("Loaded project '{}' with {} files", project_id, project_files.len());
        Ok(project_id)
    }
    
    /// Detect primary language of a project
    async fn detect_language(&self, project_path: &std::path::Path) -> String {
        // Simple language detection based on file extensions
        let extensions = [".rs", ".py", ".js", ".ts", ".java", ".cpp", ".c"];
        
        for ext in extensions {
            if project_path.to_string_lossy().contains(ext) {
                return match ext {
                    ".rs" => "Rust",
                    ".py" => "Python", 
                    ".js" => "JavaScript",
                    ".ts" => "TypeScript",
                    ".java" => "Java",
                    ".cpp" | ".c" => "C/C++",
                    _ => "Unknown",
                }.to_string();
            }
        }
        
        "Unknown".to_string()
    }
    
    /// Run the IDE (this would typically start the UI or web server)
    pub async fn run(&self) -> IdeResult<()> {
        // This would typically start the web server or desktop app
        log::info!("Starting Super IDE v{}", crate::VERSION);
        
        // Subscribe to AI suggestions
        let _ai_engine = self.ai_engine.clone();
        let _event_bus = self.event_bus.clone();
        tokio::spawn(async move {
            // This would handle AI suggestion events
            log::info!("AI Engine ready for suggestions");
        });
        
        // Subscribe to editor events
        let _editor = self.editor.clone();
        let _event_bus = self.event_bus.clone();
        tokio::spawn(async move {
            // This would handle editor events
            log::info!("Editor event processing started");
        });
        
        Ok(())
    }
    
    /// Create a new terminal session
    pub async fn create_terminal(&self, title: Option<String>) -> IdeResult<String> {
        self.terminal_manager.create_session(title).await.map_err(|e| e.into())
    }
    
    /// Start a terminal session
    pub async fn start_terminal(&self, session_id: &str) -> IdeResult<()> {
        self.terminal_manager.start_terminal(session_id).await.map_err(|e| e.into())
    }
    
    /// Stop a terminal session
    pub async fn stop_terminal(&self, session_id: &str) -> IdeResult<()> {
        self.terminal_manager.stop_terminal(session_id).await.map_err(|e| e.into())
    }
    
    /// Send input to a terminal session
    pub async fn send_terminal_input(&self, session_id: &str, input: &str) -> IdeResult<()> {
        self.terminal_manager.send_input(session_id, input).await.map_err(|e| e.into())
    }
    
    /// List all terminal sessions
    pub async fn list_terminals(&self) -> Vec<super::terminal::TerminalSession> {
        self.terminal_manager.list_sessions().await
    }
    
    /// Execute a command in a new terminal session
    pub async fn execute_command(&self, command: &str, title: Option<String>) -> IdeResult<super::terminal::ProcessResult> {
        let session_id = self.create_terminal(title).await?;
        self.start_terminal(&session_id).await?;

        let executor = super::terminal::CommandExecutor::default();
        let result = executor.execute(command).await?;

        // Clean up the session
        let _ = self.stop_terminal(&session_id).await;

        Ok(result)
    }

    /// Open a file in the editor
    pub async fn open_file(&self, file_path: std::path::PathBuf) -> IdeResult<String> {
        let editor = self.editor.lock().await;
        let document_id = editor.open_file(file_path.clone()).await?;

        // Get document info for state tracking
        let doc_info = {
            let doc_opt = editor.get_active_document().await;
            if let Some(doc) = doc_opt {
                let doc_read = doc.read().await;
                EditorTab {
                    id: document_id.clone(),
                    title: doc_read.title.clone(),
                    file_path: doc_read.path.clone(),
                    is_modified: doc_read.is_modified,
                    language: doc_read.language.clone(),
                    cursor_position: (doc_read.cursor_line, doc_read.cursor_column),
                }
            } else {
                return Err(IdeError::Editor("Failed to get document info".to_string()));
            }
        };

        // Add to active tabs
        self.update_state(|state| {
            // Remove if already exists
            state.active_tabs.retain(|tab| tab.id != document_id);
            state.active_tabs.push(doc_info);
        }).await;

        // Publish event
        let _ = self.event_bus.publish("editor", crate::utils::event_bus::IdeEvent::EditorOpened {
            document_id: document_id.clone(),
            file_path: file_path.to_string_lossy().to_string(),
        });

        Ok(document_id)
    }

    /// Save the active document
    pub async fn save_active_file(&self) -> IdeResult<()> {
        let editor = self.editor.lock().await;
        editor.save_active_document().await?;

        // Update state
        if let Some(active_doc) = editor.get_active_document().await {
            let doc_read = active_doc.read().await;
            let document_id = doc_read.id.clone();

            self.update_state(|state| {
                if let Some(tab) = state.active_tabs.iter_mut().find(|tab| tab.id == document_id) {
                    tab.is_modified = false;
                }
            }).await;
        }

        Ok(())
    }

    /// Close a file/document
    pub async fn close_file(&self, document_id: &str) -> IdeResult<bool> {
        let editor = self.editor.lock().await;
        let closed = editor.close_document(document_id).await?;

        if closed {
            // Remove from active tabs
            self.update_state(|state| {
                state.active_tabs.retain(|tab| tab.id != document_id);
            }).await;

            // Publish event
            let _ = self.event_bus.publish("editor", crate::utils::event_bus::IdeEvent::EditorClosed {
                document_id: document_id.to_string(),
            });
        }

        Ok(closed)
    }

    /// Get list of open files
    pub async fn get_open_files(&self) -> Vec<EditorTab> {
        let state = self.get_state().await;
        state.active_tabs
    }

    /// Get current workspace files
    pub async fn get_workspace_files(&self) -> IdeResult<Vec<String>> {
        let config = self.config.read().await;
        let workspace_path = std::path::PathBuf::from(&config.ide.workspace_path);

        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&workspace_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            if let Some(file_name) = entry.file_name().to_str() {
                                files.push(file_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Get code completions for the current context
    pub async fn get_code_completions(&self, _document_id: &str, cursor_position: (usize, usize), text_context: &str) -> IdeResult<Vec<crate::editor::CompletionItem>> {
        let editor = self.editor.lock().await;

        let context = crate::editor::CompletionContext {
            cursor_position: crate::editor::CursorPosition {
                line: cursor_position.0,
                column: cursor_position.1,
            },
            language: "Rust".to_string(), // Would detect from document
            text_before_cursor: text_context.to_string(),
            text_after_cursor: String::new(),
        };

        let completions = editor.get_completions(&context).await?;
        Ok(completions)
    }

    /// Analyze code for issues and suggestions
    pub async fn analyze_code(&self, code: &str, language: &str) -> IdeResult<crate::ai::AnalysisResult> {
        match self.ai_engine.analyze_code(code, language).await {
            Ok(analysis) => Ok(analysis),
            Err(e) => Err(IdeError::AiEngine(e.to_string())),
        }
    }

    /// Get AI code suggestions
    pub async fn get_ai_suggestions(&self, context: &str, language: &str) -> IdeResult<String> {
        let request = crate::ai::CompletionRequest {
            prompt: format!("Provide code suggestions for: {}", context),
            context: context.to_string(),
            language: language.to_string(),
            max_tokens: Some(100),
        };

        match self.ai_engine.generate_completion(request).await {
            Ok(response) => Ok(response.text),
            Err(e) => Err(IdeError::AiEngine(e.to_string())),
        }
    }
}



impl Default for AIContext {
    fn default() -> Self {
        Self {
            custom_instructions: Vec::new(),
            coding_style: CodingStyle {
                naming_convention: "snake_case".to_string(),
                indentation: "spaces".to_string(),
                line_length: 80,
                bracket_style: "allman".to_string(),
            },
            preferred_patterns: Vec::new(),
            recent_interactions: Vec::new(),
        }
    }
}
