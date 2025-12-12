//! Git integration module for Super IDE
//! 
//! Provides Git operations including:
//! - Repository status and information
//! - File diff viewing
//! - Commit history and browsing
//! - Branch management
//! - Integration with GitHub/GitLab

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use chrono::{DateTime, Utc, TimeZone};

/// Git repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepository {
    pub path: PathBuf,
    pub current_branch: String,
    pub status: GitStatus,
    pub remote_url: Option<String>,
    pub last_commit: Option<GitCommit>,
}

/// Git repository status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub staged_files: Vec<GitFile>,
    pub unstaged_files: Vec<GitFile>,
    pub untracked_files: Vec<GitFile>,
    pub ahead_count: u32,
    pub behind_count: u32,
}

/// Git file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFile {
    pub path: String,
    pub status: FileStatus,
    pub added_lines: Option<u32>,
    pub removed_lines: Option<u32>,
}

/// File status in Git
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
}

/// Git commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub timestamp: DateTime<Utc>,
    pub files_changed: Vec<String>,
    pub insertions: u32,
    pub deletions: u32,
}

/// Git diff information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiff {
    pub file_path: String,
    pub old_content: String,
    pub new_content: String,
    pub hunks: Vec<GitHunk>,
}

/// Git diff hunk
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Clone)]
pub struct GitHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub content: String,
}

/// Git branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub ahead_count: u32,
    pub behind_count: u32,
}

/// Git configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub default_branch: String,
    pub auto_commit: bool,
}

/// Git error types
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Repository not found at path: {0}")]
    RepositoryNotFound(PathBuf),
    
    #[error("Git command failed: {0}")]
    CommandFailed(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Git manager for handling Git operations
#[derive(Debug)]
pub struct GitManager {
    repository_path: PathBuf,
}

impl GitManager {
    /// Create a new Git manager for the given repository path
    pub fn new(repository_path: PathBuf) -> Self {
        Self { repository_path }
    }

    /// Check if the path is a Git repository
    pub async fn is_repository(&self) -> bool {
        let git_dir = self.repository_path.join(".git");
        git_dir.exists() && git_dir.is_dir()
    }

    /// Initialize a new Git repository
    pub async fn init_repository(&self) -> Result<()> {
        let output = Command::new("git")
            .arg("init")
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok(())
    }

    /// Get repository information
    pub async fn get_repository_info(&self) -> Result<GitRepository> {
        if !self.is_repository().await {
            return Err(GitError::RepositoryNotFound(self.repository_path.clone()).into());
        }

        let current_branch = self.get_current_branch().await?;
        let status = self.get_status().await?;
        let remote_url = self.get_remote_url().await.ok();
        let last_commit = self.get_last_commit().await.ok();

        Ok(GitRepository {
            path: self.repository_path.clone(),
            current_branch,
            status,
            remote_url,
            last_commit,
        })
    }

    /// Get current branch name
    pub async fn get_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .arg("branch")
            .arg("--show-current")
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get repository status
    pub async fn get_status(&self) -> Result<GitStatus> {
        let output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        let status_text = String::from_utf8_lossy(&output.stdout);
        let mut staged_files = Vec::new();
        let mut unstaged_files = Vec::new();
        let mut untracked_files = Vec::new();

        for line in status_text.lines() {
            if line.len() < 3 {
                continue;
            }

            let status_code = &line[..2];
            let file_path = line[3..].trim();

            let file = GitFile {
                path: file_path.to_string(),
                status: match status_code {
                    "A " => FileStatus::Added,
                    "M " => FileStatus::Modified,
                    "D " => FileStatus::Deleted,
                    "R " => FileStatus::Renamed,
                    "??" => FileStatus::Untracked,
                    _ => FileStatus::Modified,
                },
                added_lines: None,
                removed_lines: None,
            };

            match status_code {
                "A " | "R " | "C " => staged_files.push(file),
                "M " | "D " => unstaged_files.push(file),
                "??" => untracked_files.push(file),
                _ => unstaged_files.push(file),
            }
        }

        // Get ahead/behind counts
        let (ahead_count, behind_count) = self.get_ahead_behind_counts().await?;

        Ok(GitStatus {
            staged_files,
            unstaged_files,
            untracked_files,
            ahead_count,
            behind_count,
        })
    }

    /// Get ahead/behind counts for current branch
    async fn get_ahead_behind_counts(&self) -> Result<(u32, u32)> {
        // This is a simplified implementation
        // In a full version, you'd parse git status output with more detail
        Ok((0, 0))
    }

    /// Get remote URL
    pub async fn get_remote_url(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["remote", "get-url", "origin"])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get last commit information
    pub async fn get_last_commit(&self) -> Result<GitCommit> {
        let output = Command::new("git")
            .args(&[
                "log",
                "-1",
                "--pretty=format:%H|%an|%ae|%at|%s",
                "--numstat"
            ])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        let output_text = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_text.lines().collect();
        
        if lines.is_empty() {
            return Err(GitError::ParseError("No commit data found".to_string()).into());
        }

        // Parse commit header
        let header_parts: Vec<&str> = lines[0].split('|').collect();
        if header_parts.len() < 5 {
            return Err(GitError::ParseError("Invalid commit format".to_string()).into());
        }

        let hash = header_parts[0].to_string();
        let author = header_parts[1].to_string();
        let email = header_parts[2].to_string();
        let timestamp = header_parts[3].parse::<i64>().unwrap_or(0);
        let message = header_parts[4].to_string();

        // Parse numstat data
        let mut files_changed = Vec::new();
        let mut insertions = 0;
        let mut deletions = 0;

        for line in &lines[1..] {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                if let Ok(additions) = parts[0].parse::<u32>() {
                    insertions += additions;
                }
                if let Ok(deletions) = parts[1].parse::<u32>() {
                    deletions += deletions;
                }
                files_changed.push(parts[2].to_string());
            }
        }

        Ok(GitCommit {
            hash,
            message,
            author,
            email,
            timestamp: Utc.timestamp(timestamp, 0).unwrap(),
            files_changed,
            insertions,
            deletions,
        })
    }

    /// Get commit history
    pub async fn get_commit_history(&self, limit: u32) -> Result<Vec<GitCommit>> {
        let output = Command::new("git")
            .args(&[
                "log",
                "--pretty=format:%H|%an|%ae|%at|%s",
                "-n",
                &limit.to_string()
            ])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        let output_text = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for line in output_text.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                let hash = parts[0].to_string();
                let author = parts[1].to_string();
                let email = parts[2].to_string();
                let timestamp = parts[3].parse::<i64>().unwrap_or(0);
                let message = parts[4].to_string();

                commits.push(GitCommit {
                    hash,
                    message,
                    author,
                    email,
                    timestamp: Utc.timestamp(timestamp, 0).unwrap(),
                    files_changed: Vec::new(),
                    insertions: 0,
                    deletions: 0,
                });
            }
        }

        Ok(commits)
    }

    /// Get diff for a specific file
    pub async fn get_file_diff(&self, file_path: &str) -> Result<GitDiff> {
        let output = Command::new("git")
            .args(&["diff", file_path])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        let diff_text = String::from_utf8_lossy(&output.stdout);
        
        // This is a simplified diff parser
        // A full implementation would parse unified diff format properly
        Ok(GitDiff {
            file_path: file_path.to_string(),
            old_content: String::new(),
            new_content: String::new(),
            hunks: vec![GitHunk {
                old_start: 0,
                old_lines: 0,
                new_start: 0,
                new_lines: 0,
                content: diff_text.to_string(),
            }],
        })
    }

    /// Get all branches
    pub async fn get_branches(&self) -> Result<Vec<GitBranch>> {
        let output = Command::new("git")
            .args(&["branch", "-a", "--format=%(refname:short)|%(upstream:short)|%(aheadbehind)"])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        let output_text = String::from_utf8_lossy(&output.stdout);
        let current_branch = self.get_current_branch().await?;
        let mut branches = Vec::new();

        for line in output_text.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            let branch_name = parts[0].to_string();
            let is_current = branch_name == current_branch;
            let is_remote = branch_name.starts_with("remotes/");
            let ahead_behind = if parts.len() > 2 { parts[2] } else { "" };

            let (ahead_count, behind_count) = if !ahead_behind.is_empty() {
                // Parse ahead/behind counts
                let counts: Vec<&str> = ahead_behind.split(',').collect();
                let ahead = counts.get(0).and_then(|s| s.trim().strip_prefix("+")).unwrap_or("0");
                let behind = counts.get(1).and_then(|s| s.trim().strip_prefix("-")).unwrap_or("0");
                
                (
                    ahead.parse::<u32>().unwrap_or(0),
                    behind.parse::<u32>().unwrap_or(0)
                )
            } else {
                (0, 0)
            };

            branches.push(GitBranch {
                name: branch_name,
                is_current,
                is_remote,
                ahead_count,
                behind_count,
            });
        }

        Ok(branches)
    }

    /// Stage files for commit
    pub async fn stage_files(&self, files: &[String]) -> Result<()> {
        let mut command = Command::new("git");
        command.arg("add").current_dir(&self.repository_path);

        for file in files {
            command.arg(file);
        }

        let output = command.output().await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok(())
    }

    /// Create a commit
    pub async fn commit(&self, message: &str) -> Result<String> {
        let output = Command::new("git")
            .args(&["commit", "-m", message])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        // Get the commit hash
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok(String::new())
        }
    }

    /// Checkout a branch
    pub async fn checkout_branch(&self, branch_name: &str) -> Result<()> {
        let output = Command::new("git")
            .args(&["checkout", branch_name])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok(())
    }

    /// Create and checkout a new branch
    pub async fn create_branch(&self, branch_name: &str) -> Result<()> {
        let output = Command::new("git")
            .args(&["checkout", "-b", branch_name])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok(())
    }

    /// Push changes to remote repository
    pub async fn push(&self, remote: Option<&str>, branch: Option<&str>) -> Result<String> {
        let remote_arg = remote.unwrap_or("origin");
        let branch_arg = branch.unwrap_or("HEAD");
        
        let output = Command::new("git")
            .args(&["push", remote_arg, branch_arg])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok("Push completed successfully".to_string())
    }

    /// Pull changes from remote repository
    pub async fn pull(&self, remote: Option<&str>, branch: Option<&str>) -> Result<String> {
        let remote_arg = remote.unwrap_or("origin");
        let branch_arg = branch.unwrap_or("HEAD");
        
        let output = Command::new("git")
            .args(&["pull", remote_arg, branch_arg])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok("Pull completed successfully".to_string())
    }

    /// Get file diff
    pub async fn get_diff(&self, file_path: Option<&str>, staged: bool) -> Result<GitDiff> {
        let mut args = vec!["diff"];
        if staged {
            args.push("--staged");
        }
        if let Some(file) = file_path {
            args.push(file);
        }

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        let diff_content = String::from_utf8_lossy(&output.stdout).to_string();
        
        // Parse diff into hunks (simplified parsing)
        let hunks = self.parse_diff_hunks(&diff_content);
        
        Ok(GitDiff {
            file_path: file_path.unwrap_or("").to_string(),
            old_content: String::new(), // Would need additional git command to get old content
            new_content: diff_content,
            hunks,
        })
    }

    /// Get commit history
    pub async fn get_log(&self, limit: u32) -> Result<Vec<GitCommit>> {
        let output = Command::new("git")
            .args(&["log", "--pretty=format:%H%n%an%n%ae%n%ad%n%s", &format!("-{}", limit)])
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        let log_content = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();
        let lines: Vec<&str> = log_content.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            if i + 4 < lines.len() {
                let hash = lines[i].to_string();
                let author = lines[i + 1].to_string();
                let email = lines[i + 2].to_string();
                let date_str = lines[i + 3].to_string();
                let message = lines[i + 4].to_string();
                
                // Parse date (simplified)
                let timestamp = Utc::now();
                
                commits.push(GitCommit {
                    hash,
                    message,
                    author,
                    email,
                    timestamp,
                    files_changed: Vec::new(), // Would need additional parsing
                    insertions: 0,
                    deletions: 0,
                });
                
                i += 5; // Move to next commit
            } else {
                break;
            }
        }
        
        Ok(commits)
    }

    /// Stage specific files
    pub async fn stage_files_optimized(&self, files: &[String]) -> Result<()> {
        if files.is_empty() {
            return Ok(());
        }

        let mut args = vec!["add"];
        args.extend(files.iter().map(|f| f.as_str()));

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok(())
    }

    /// Unstage specific files
    pub async fn unstage_files(&self, files: &[String]) -> Result<()> {
        if files.is_empty() {
            return Ok(());
        }

        let mut args = vec!["reset", "HEAD"];
        args.extend(files.iter().map(|f| f.as_str()));

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok(())
    }

    /// Discard changes to specific files
    pub async fn discard_changes(&self, files: &[String]) -> Result<()> {
        if files.is_empty() {
            return Ok(());
        }

        let mut args = vec!["checkout", "--"];
        args.extend(files.iter().map(|f| f.as_str()));

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok(())
    }

    /// Initialize a new git repository
    pub async fn init(&self) -> Result<()> {
        let output = Command::new("git")
            .arg("init")
            .current_dir(&self.repository_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ).into());
        }

        Ok(())
    }

    /// Parse diff hunks (simplified implementation)
    fn parse_diff_hunks(&self, diff_content: &str) -> Vec<GitHunk> {
        let mut hunks = Vec::new();
        let lines: Vec<&str> = diff_content.lines().collect();
        
        let mut current_hunk = GitHunk {
            old_start: 0,
            old_lines: 0,
            new_start: 0,
            new_lines: 0,
            content: String::new(),
        };

        for line in lines {
            if line.starts_with("@@") {
                if !current_hunk.content.is_empty() {
                    hunks.push(current_hunk);
                }
                
                // Parse hunk header like @@ -1,3 +1,3 @@
                if let Some(parts) = line.split_whitespace().nth(1) {
                    if let Some(parts) = parts.split(',').next().and_then(|s| s.strip_prefix('-')) {
                        if let Some((old_start, old_lines)) = parts.split_once(',') {
                            current_hunk.old_start = old_start.parse().unwrap_or(1);
                            current_hunk.old_lines = old_lines.parse().unwrap_or(1);
                        }
                    }
                }
                
                if let Some(parts) = line.split_whitespace().nth(2) {
                    if let Some(parts) = parts.split(',').next().and_then(|s| s.strip_prefix('+')) {
                        if let Some((new_start, new_lines)) = parts.split_once(',') {
                            current_hunk.new_start = new_start.parse().unwrap_or(1);
                            current_hunk.new_lines = new_lines.parse().unwrap_or(1);
                        }
                    }
                }
                
                current_hunk.content = line.to_string();
            } else {
                current_hunk.content.push_str("\n");
                current_hunk.content.push_str(line);
            }
        }
        
        if !current_hunk.content.is_empty() {
            hunks.push(current_hunk.clone());
        }
        
        hunks
    }
}