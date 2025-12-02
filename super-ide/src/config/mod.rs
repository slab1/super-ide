//! Configuration management for Super IDE

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use thiserror::Error;
use config::{Config, Environment, File};

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Configuration loading error: {0}")]
    Load(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub ide: IDESettings,
    pub ai: AISettings,
    pub editor: EditorSettings,
    pub theme: ThemeSettings,
    pub collaboration: CollaborationSettings,
    pub security: SecuritySettings,
    pub plugins: PluginSettings,
}

/// IDE general settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDESettings {
    pub workspace_path: String,
    pub auto_save_interval: u64, // seconds
    pub max_recent_files: usize,
    pub enable_telemetry: bool,
    pub crash_reporting: bool,
    pub update_check: bool,
    pub language: String,
    pub timezone: String,
}

/// AI settings and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISettings {
    pub provider: AIProvider,
    pub model_path: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub enable_local_inference: bool,
    pub cache_size: usize, // MB
    pub custom_instructions: Vec<String>,
    pub privacy_mode: bool,
    pub learning_enabled: bool,
}

/// AI providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    Local,
    OpenAI,
    Anthropic,
    Custom,
}

/// Editor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    pub font_family: String,
    pub font_size: u32,
    pub line_height: f32,
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub minimap_enabled: bool,
    pub line_numbers: bool,
    pub show_whitespace: bool,
    pub auto_close_brackets: bool,
    pub auto_indent: bool,
    pub format_on_save: bool,
    pub enable_live_linting: bool,
    pub spell_check: bool,
    pub code_folding: bool,
    pub bracket_matching: bool,
    pub highlight_selection: bool,
    pub show_code_actions: bool,
}

/// Theme settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub name: String,
    pub dark_mode: bool,
    pub primary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub foreground_color: String,
    pub syntax_highlighting: HashMap<String, String>,
    pub custom_css: Option<String>,
}

/// Collaboration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSettings {
    pub enable_real_time: bool,
    pub share_cursor_position: bool,
    pub share_selections: bool,
    pub max_collaborators: usize,
    pub server_url: Option<String>,
    pub enable_voice_chat: bool,
    pub share_screenshots: bool,
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub scan_for_secrets: bool,
    pub local_llm_only: bool,
    pub encrypt_local_data: bool,
    pub secure_delete: bool,
    pub certificate_validation: bool,
    pub trusted_domains: Vec<String>,
}

/// Plugin settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    pub enabled: bool,
    pub auto_update: bool,
    pub marketplace_url: String,
    pub trust_level: PluginTrustLevel,
    pub custom_plugins: Vec<String>,
}

/// Plugin trust levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginTrustLevel {
    Trusted,
    Verified,
    Community,
    Unknown,
}

/// User profile and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: UserPreferences,
    pub recent_projects: Vec<RecentProject>,
    pub shortcuts: KeyboardShortcuts,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub startup_behavior: StartupBehavior,
    pub file_associations: HashMap<String, String>,
    pub custom_commands: Vec<CustomCommand>,
    pub workspace_layout: WorkspaceLayout,
    pub notification_settings: NotificationSettings,
}

/// Startup behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StartupBehavior {
    LoadLastSession,
    ShowWelcomeScreen,
    NewEmptyProject,
    OpenSpecificProject(String),
}

/// Custom command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommand {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub hotkey: Option<String>,
    pub icon: Option<String>,
}

/// Workspace layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    pub layout_type: LayoutType,
    pub panels: Vec<PanelConfig>,
    pub window_state: WindowState,
}

/// Panel layout types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutType {
    Single,
    SplitHorizontal,
    SplitVertical,
    Grid,
    Custom(String),
}

/// Panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    pub panel_type: PanelType,
    pub position: PanelPosition,
    pub size: PanelSize,
    pub visible: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Panel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    Editor,
    Explorer,
    Terminal,
    Output,
    Problems,
    Debug,
    Git,
    Extensions,
    AIAssistant,
}

/// Panel position and sizing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
    pub fullscreen: bool,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub show_ai_suggestions: bool,
    pub show_build_results: bool,
    pub show_git_notifications: bool,
    pub show_error_alerts: bool,
    pub show_update_notifications: bool,
    pub sound_enabled: bool,
    pub position: NotificationPosition,
}

/// Notification positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
}

/// Recent project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub id: String,
    pub name: String,
    pub path: String,
    pub language: String,
    pub last_opened: chrono::DateTime<chrono::Utc>,
    pub icon: Option<String>,
}

/// Keyboard shortcuts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardShortcuts {
    pub global: HashMap<String, Vec<String>>,
    pub editor: HashMap<String, Vec<String>>,
    pub terminal: HashMap<String, Vec<String>>,
    pub navigation: HashMap<String, Vec<String>>,
    pub ai: HashMap<String, Vec<String>>,
}

impl Configuration {
    /// Load configuration from default locations
    pub async fn load() -> Result<Self, ConfigError> {
        let mut config = Config::new();
        
        // Load from config file
        let config_paths = Self::get_config_paths();
        for path in config_paths {
            if path.exists() {
                config = config.merge(File::with_name(&path.to_string_lossy()))?;
            }
        }
        
        // Load from environment variables
        config = config.merge(Environment::with_prefix("SUPER_IDE"))?;
        
        // Load and validate
        let mut settings: Configuration = config.try_deserialize()
            .map_err(|e| ConfigError::Load(e.to_string()))?;
            
        settings.validate()?;
        settings.apply_defaults();
        
        Ok(settings)
    }
    
    /// Save configuration to file
    pub async fn save(&self) -> Result<(), ConfigError> {
        let config_dir = Self::get_config_dir()?;
        std::fs::create_dir_all(&config_dir)?;
        
        let config_file = config_dir.join("config.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(ConfigError::Serialization)?;
            
        std::fs::write(config_file, json)
            .map_err(ConfigError::Io)?;
            
        Ok(())
    }
    
    /// Get configuration directory
    fn get_config_dir() -> Result<PathBuf, ConfigError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| ConfigError::Load("Cannot find home directory".to_string()))?;
            
        #[cfg(target_os = "windows")]
        let config_dir = home_dir.join("AppData").join("Roaming").join("SuperIDE");
        
        #[cfg(target_os = "macos")]
        let config_dir = home_dir.join("Library").join("Application Support").join("SuperIDE");
        
        #[cfg(target_os = "linux")]
        let config_dir = home_dir.join(".config").join("super-ide");
        
        Ok(config_dir)
    }
    
    /// Get configuration file paths to search
    fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        if let Ok(config_dir) = Self::get_config_dir() {
            paths.push(config_dir.join("config.json"));
        }
        
        // Also check current directory
        paths.push(std::path::PathBuf::from(".super-ide.json"));
        
        // Check environment variable
        if let Ok(custom_path) = std::env::var("SUPER_IDE_CONFIG") {
            paths.push(PathBuf::from(custom_path));
        }
        
        paths
    }
    
    /// Validate configuration settings
    fn validate(&mut self) -> Result<(), ConfigError> {
        // Validate font sizes
        if self.editor.font_size < 8 || self.editor.font_size > 72 {
            return Err(ConfigError::Validation(
                "Font size must be between 8 and 72".to_string()
            ));
        }
        
        // Validate AI settings
        if matches!(self.ai.provider, AIProvider::Local) && self.ai.model_path.is_none() {
            return Err(ConfigError::Validation(
                "Local AI provider requires a model path".to_string()
            ));
        }
        
        if matches!(self.ai.provider, AIProvider::OpenAI | AIProvider::Anthropic) && self.ai.api_key.is_none() {
            return Err(ConfigError::Validation(
                "Cloud AI providers require an API key".to_string()
            ));
        }
        
        // Validate temperature range
        if self.ai.temperature < 0.0 || self.ai.temperature > 2.0 {
            return Err(ConfigError::Validation(
                "AI temperature must be between 0.0 and 2.0".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Apply default values where settings are missing
    fn apply_defaults(&mut self) {
        if self.ide.workspace_path.is_empty() {
            if let Ok(home) = std::env::var("HOME") {
                self.ide.workspace_path = format!("{}/SuperIDE/workspace", home);
            } else {
                self.ide.workspace_path = "./workspace".to_string();
            }
        }
        
        if self.ide.language.is_empty() {
            self.ide.language = "en-US".to_string();
        }
        
        if self.editor.font_family.is_empty() {
            self.editor.font_family = "Fira Code".to_string();
        }
        
        if self.theme.name.is_empty() {
            self.theme.name = "Dark".to_string();
        }
    }
    
    /// Update specific setting
    pub fn update_setting(&mut self, path: &str, value: serde_json::Value) -> Result<(), ConfigError> {
        // This would parse the path and update the nested setting
        // For now, implement basic path handling
        match path {
            "editor.font_size" => {
                let size: u32 = serde_json::from_value(value)
                    .map_err(|e| ConfigError::Validation(e.to_string()))?;
                if size >= 8 && size <= 72 {
                    self.editor.font_size = size;
                    Ok(())
                } else {
                    Err(ConfigError::Validation(
                        "Font size must be between 8 and 72".to_string()
                    ))
                }
            },
            "ai.provider" => {
                let provider: AIProvider = serde_json::from_value(value)
                    .map_err(|e| ConfigError::Validation(e.to_string()))?;
                self.ai.provider = provider;
                Ok(())
            },
            "theme.dark_mode" => {
                let dark_mode: bool = serde_json::from_value(value)
                    .map_err(|e| ConfigError::Validation(e.to_string()))?;
                self.theme.dark_mode = dark_mode;
                Ok(())
            },
            _ => Err(ConfigError::Validation(
                format!("Unknown setting path: {}", path)
            ))
        }
    }
    
    /// Get workspace directory
    pub fn workspace_dir(&self) -> PathBuf {
        PathBuf::from(&self.ide.workspace_path)
    }
    
    /// Check if AI assistance is enabled
    pub fn is_ai_enabled(&self) -> bool {
        self.ai.enable_local_inference || 
        (self.ai.api_key.is_some() && !self.ai.privacy_mode)
    }
    
    /// Get AI provider configuration
    pub fn ai_provider(&self) -> &AIProvider {
        &self.ai.provider
    }
    
    /// Check if real-time collaboration is enabled
    pub fn is_collaboration_enabled(&self) -> bool {
        self.collaboration.enable_real_time
    }
    
    /// Get custom CSS for theming
    pub fn get_custom_css(&self) -> Option<&str> {
        self.theme.custom_css.as_deref()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        let mut config = Self {
            ide: IDESettings {
                workspace_path: "./workspace".to_string(),
                auto_save_interval: 30,
                max_recent_files: 50,
                enable_telemetry: true,
                crash_reporting: true,
                update_check: true,
                language: "en-US".to_string(),
                timezone: "UTC".to_string(),
            },
            ai: AISettings {
                provider: AIProvider::Local,
                model_path: None,
                api_key: None,
                base_url: None,
                max_tokens: 2048,
                temperature: 0.7,
                enable_local_inference: true,
                cache_size: 512,
                custom_instructions: Vec::new(),
                privacy_mode: false,
                learning_enabled: true,
            },
            editor: EditorSettings {
                font_family: "Fira Code".to_string(),
                font_size: 14,
                line_height: 1.5,
                tab_size: 4,
                insert_spaces: true,
                word_wrap: false,
                minimap_enabled: true,
                line_numbers: true,
                show_whitespace: false,
                auto_close_brackets: true,
                auto_indent: true,
                format_on_save: true,
                enable_live_linting: true,
                spell_check: false,
                code_folding: true,
                bracket_matching: true,
                highlight_selection: true,
                show_code_actions: true,
            },
            theme: ThemeSettings {
                name: "Dark".to_string(),
                dark_mode: true,
                primary_color: "#007ACC".to_string(),
                accent_color: "#FF6B6B".to_string(),
                background_color: "#1E1E1E".to_string(),
                foreground_color: "#D4D4D4".to_string(),
                syntax_highlighting: HashMap::new(),
                custom_css: None,
            },
            collaboration: CollaborationSettings {
                enable_real_time: true,
                share_cursor_position: true,
                share_selections: true,
                max_collaborators: 10,
                server_url: None,
                enable_voice_chat: false,
                share_screenshots: false,
            },
            security: SecuritySettings {
                scan_for_secrets: true,
                local_llm_only: false,
                encrypt_local_data: true,
                secure_delete: true,
                certificate_validation: true,
                trusted_domains: Vec::new(),
            },
            plugins: PluginSettings {
                enabled: true,
                auto_update: true,
                marketplace_url: "https://plugins.super-ide.dev".to_string(),
                trust_level: PluginTrustLevel::Verified,
                custom_plugins: Vec::new(),
            },
        };
        
        config.apply_defaults();
        config
    }
}

// Import required dependencies
use dirs;