//! Core IDE functionality and main application state

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use anyhow::Result;
use thiserror::Error;

use crate::ai::AiEngine;
use crate::editor::Editor;
use crate::config::Configuration;
use crate::utils::event_bus::EventBus;

/// Main IDE result type
pub type IdeResult<T> = Result<T, IdeError>;

/// Main IDE errors
#[derive(Error, Debug)]
pub enum IdeError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("AI Engine error: {0}")]
    AIEngine(String),
    
    #[error("Editor error: {0}")]
    Editor(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(String),
}

/// Main SuperIDE application state
#[derive(Clone)]
pub struct SuperIDE {
    /// Core configuration
    config: Arc<RwLock<Configuration>>,
    
    /// AI engine for code intelligence
    ai_engine: Arc<AiEngine>,
    
    /// Code editor instance
    editor: Arc<Mutex<Editor>>,
    
    /// Event bus for inter-component communication
    event_bus: Arc<EventBus>,
    
    /// Application state
    state: Arc<RwLock<IdeState>>,
}

/// IDE application state
#[derive(Debug, Clone)]
pub struct IdeState {
    /// Currently open projects
    projects: Vec<ProjectInfo>,
    
    /// Active editor tabs
    active_tabs: Vec<EditorTab>,
    
    /// User preferences and settings
    preferences: UserPreferences,
    
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
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
#[derive(Debug, Clone)]
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
        let ai_engine = AIEngine::new(&config).await?;
        let editor = Editor::new(&config).await?;
        let event_bus = EventBus::new();
        
        let state = IdeState {
            projects: Vec::new(),
            active_tabs: Vec::new(),
            preferences: UserPreferences::default(),
            performance_metrics: PerformanceMetrics::default(),
        };
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            ai_engine: Arc::new(ai_engine),
            editor: Arc::new(Mutex::new(editor)),
            event_bus: Arc::new(event_bus),
            state: Arc::new(RwLock::new(state)),
        })
    }
    
    /// Get AI engine reference
    pub fn ai_engine(&self) -> &Arc<AIEngine> {
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
    
    /// Get current IDE state
    pub async fn get_state(&self) -> IdeState {
        self.state.read().await.clone()
    }
    
    /// Update IDE state
    pub async fn update_state(&self, update: impl FnOnce(&mut IdeState)) {
        let mut state = self.state.write().await;
        update(&mut state);
    }
    
    /// Load a project
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
            path: project_path,
            language: self.detect_language(&project_path).await,
            created_at: chrono::Utc::now(),
            last_opened: chrono::Utc::now(),
            ai_context: AIContext::default(),
        };
        
        self.update_state(|state| {
            state.projects.push(project_info);
        }).await;
        
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
        let ai_engine = self.ai_engine.clone();
        let event_bus = self.event_bus.clone();
        tokio::spawn(async move {
            // This would handle AI suggestion events
            log::info!("AI Engine ready for suggestions");
        });
        
        // Subscribe to editor events
        let editor = self.editor.clone();
        let event_bus = self.event_bus.clone();
        tokio::spawn(async move {
            // This would handle editor events
            log::info!("Editor event processing started");
        });
        
        Ok(())
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_size: 14,
            auto_save: true,
            ai_assistance: true,
            collaboration: true,
            keyboard_shortcuts: KeyboardShortcuts {
                save: vec!["ctrl+s".to_string()],
                format: vec!["shift+alt+f".to_string()],
                run: vec!["ctrl+f5".to_string()],
                debug: vec!["f5".to_string()],
                ai_suggestion: vec!["ctrl+space".to_string()],
            },
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