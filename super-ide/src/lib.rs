//! Superior Intelligent IDE - AI-Powered Development Environment
//! 
//! A next-generation IDE built in Rust with advanced AI capabilities,
//! intelligent code analysis, real-time collaboration, and modern UX.

pub mod core;
pub mod ai;
pub mod editor;
pub mod ui;
pub mod api;
pub mod config;
pub mod utils;
pub mod terminal;
pub mod external;
pub mod learning;
pub mod git;
pub mod file_ops;

// Re-export main components
pub use core::{SuperIDE, IdeResult, IdeError};
pub use ai::{AiEngine, AiConfig};
pub use editor::{Editor};
pub use config::{Configuration};
pub use terminal::{TerminalManager, TerminalSession, CommandExecutor, TerminalConfig, TerminalError};
pub use git::{GitManager, GitRepository, GitStatus, GitCommit, GitBranch, GitError};
pub use file_ops::{FileManager, FileInfo, ProjectStructure, FileOperationResult, FileOperationError};
pub use learning::{
    LearningEngine, 
    AITutor, 
    StudentProfile, 
    LearningModule, 
    CodeTour, 
    LearningPath,
    LearningStyle,
    SkillLevel,
    ProgressMetrics
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize the Super IDE
pub async fn initialize() -> IdeResult<SuperIDE> {
    let config = Configuration::load().await?;
    let ide = SuperIDE::new(config).await?;
    Ok(ide)
}
