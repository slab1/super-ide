//! AI Engine Module
//!
//! Provides AI-powered code analysis, completion, and assistance features.
//! Supports both local AI models and cloud providers like OpenAI.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use reqwest::{Client, header};
use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;

// Import Configuration types for conversion
use crate::config::{Configuration, AIProvider};

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub model_name: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub base_url: Option<String>,
}

/// OpenAI API request structures
#[derive(Debug, Clone, Serialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<OpenAIChoice>,
    pub usage: Option<OpenAIUsage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenAI error response
#[derive(Debug, Deserialize)]
pub struct OpenAIError {
    pub error: OpenAIErrorDetails,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIErrorDetails {
    pub message: String,
    pub r#type: String,
    pub code: Option<String>,
}

impl From<Configuration> for AiConfig {
    fn from(config: Configuration) -> Self {
        Self {
            provider: match config.ai.provider {
                AIProvider::Local => "local".to_string(),
                AIProvider::OpenAI => "openai".to_string(),
                AIProvider::Anthropic => "anthropic".to_string(),
                AIProvider::Custom => "custom".to_string(),
            },
            api_key: config.ai.api_key,
            model_name: "default".to_string(), // Could be enhanced to use model_path
            temperature: config.ai.temperature,
            max_tokens: config.ai.max_tokens,
        }
    }
}

impl From<&Configuration> for AiConfig {
    fn from(config: &Configuration) -> Self {
        Self::from(config.clone())
    }
}

/// Function information for code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub complexity: Option<f32>,
    pub signature: Option<String>,
    pub docstring: Option<String>,
}

/// Variable information for code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub name: String,
    pub variable_type: VariableType,
    pub line: usize,
    pub column: usize,
    pub scope: Option<String>,
    pub is_declared: Option<bool>,
    pub var_type: Option<String>,
    pub is_mutable: Option<bool>,
}

/// Variable types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    Local,
    Parameter,
    Field,
    Static,
}

/// Code complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeComplexity {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub lines_of_code: usize,
    pub maintainability_index: Option<f32>,
    pub nested_depth: Option<u32>,
    pub line_count: Option<u32>,
    pub function_count: Option<u32>,
    pub complexity_score: Option<f32>,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Info => write!(f, "Info"),
            IssueSeverity::Warning => write!(f, "Warning"),
            IssueSeverity::Error => write!(f, "Error"),
            IssueSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Code issue found during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub line: usize,
    pub column: usize,
}

/// Code analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<String>,
    pub complexity_score: f32,
}

/// Completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionSuggestion {
    pub text: String,
    pub confidence: f32,
    pub kind: String,
}

impl std::fmt::Display for CompletionSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// Code completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub confidence: f32,
    pub suggestions: Vec<CompletionSuggestion>,
}

/// Completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub language: String,
    pub context: String,
    pub position: Option<(usize, usize)>,
    pub prompt: String,
    pub max_tokens: Option<u32>,
}

/// User feedback for AI learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub suggestion_id: String,
    pub accepted: bool,
    pub rating: i32,
    pub context: String,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub feedback_type: Option<String>,
}

/// Code suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub id: String,
    pub title: String,
    pub description: String,
    pub message: String,
    pub code: String,
    pub confidence: f32,
}

/// Main AI Engine
#[derive(Debug, Clone)]
pub struct AiEngine {
    config: AiConfig,
    initialized: bool,
    http_client: Option<Client>,
    request_cache: Arc<RwLock<lru::LruCache<String, CompletionResponse>>>,
}

impl AiEngine {
    /// Create a new AI Engine
    pub fn new(config: AiConfig) -> Self {
        let http_client = if config.provider == "openai" || config.provider == "anthropic" {
            Some(Client::new())
        } else {
            None
        };

        Self {
            config,
            initialized: false,
            http_client,
            request_cache: Arc::new(RwLock::new(lru::LruCache::new(100))),
        }
    }

    /// Create a new AI Engine (async version for compatibility)
    pub async fn new_async(config: AiConfig) -> Result<Self> {
        let mut engine = Self::new(config);
        engine.initialize().await?;
        Ok(engine)
    }

    /// Initialize the AI Engine
    pub async fn initialize(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    /// Generate code completion
    pub async fn generate_completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        if !self.initialized {
            return Err(anyhow::anyhow!("AI Engine not initialized"));
        }

        // Check cache first
        let cache_key = format!("{}:{}:{}", request.language, request.cursor_position.unwrap_or((0, 0)).0, request.text_before_cursor);
        {
            let cache = self.request_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let result = match self.config.provider.as_str() {
            "openai" => self.generate_openai_completion(&request).await,
            "local" => self.generate_local_completion(&request).await,
            _ => Ok(CompletionResponse {
                text: "AI provider not supported".to_string(),
                confidence: 0.0,
                suggestions: vec![],
            })
        };

        // Cache the result
        if let Ok(ref response) = result {
            let mut cache = self.request_cache.write().await;
            cache.put(cache_key, response.clone());
        }

        result
    }

    /// Generate completion using OpenAI API
    async fn generate_openai_completion(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenAI API key not configured"))?;

        let client = self.http_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;

        let model = match self.config.model_name.as_str() {
            "default" => "gpt-3.5-turbo",
            other => other,
        };

        // Build the prompt for code completion
        let system_prompt = format!(
            "You are an expert {} developer. Provide helpful, accurate code completions and suggestions. Focus on clean, idiomatic code with proper error handling.",
            request.language
        );

        let user_prompt = format!(
            "Context: {}\n\nComplete this {} code:\n{}",
            request.context,
            request.language,
            request.text_before_cursor
        );

        let openai_request = OpenAIRequest {
            model: model.to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            temperature: self.config.temperature,
            max_tokens: Some(self.config.max_tokens),
            stream: false,
        };

        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.openai.com/v1");
        let url = format!("{}/chat/completions", base_url);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", api_key))?
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json")
        );

        let response = client
            .post(&url)
            .headers(headers)
            .json(&openai_request)
            .send()
            .await?;

        if response.status().is_success() {
            let openai_response: OpenAIResponse = response.json().await?;
            
            let completion_text = if let Some(choice) = openai_response.choices.first() {
                choice.message.content.clone()
            } else {
                "No completion generated".to_string()
            };

            Ok(CompletionResponse {
                text: completion_text,
                confidence: 0.8, // Default confidence for OpenAI responses
                suggestions: vec![],
            })
        } else {
            let error_response: Result<OpenAIError, _> = response.json().await;
            match error_response {
                Ok(error) => Err(anyhow::anyhow!("OpenAI API error: {}", error.error.message)),
                Err(_) => Err(anyhow::anyhow!("OpenAI API request failed with status: {}", response.status())),
            }
        }
    }

    /// Generate completion using local AI model
    async fn generate_local_completion(&self, request: &CompletionRequest) -> Result<CompletionResponse> {
        // For now, provide simple local completion suggestions
        // In a full implementation, this would use candle/transformers
        
        let suggestions = match request.language.to_lowercase().as_str() {
            "rust" => vec![
                "fn main() {".to_string(),
                "let mut".to_string(),
                "Result<(), Box<dyn std::error::Error>>".to_string(),
            ],
            "python" => vec![
                "def ".to_string(),
                "class ".to_string(),
                "import ".to_string(),
            ],
            "javascript" => vec![
                "function ".to_string(),
                "const ".to_string(),
                "let ".to_string(),
            ],
            _ => vec!["".to_string()],
        };

        Ok(CompletionResponse {
            text: suggestions.first().cloned().unwrap_or_default(),
            confidence: 0.5,
            suggestions: suggestions.into_iter().map(|text| CompletionSuggestion {
                text,
                confidence: 0.5,
                kind: "local".to_string(),
            }).collect(),
        })
    }

    /// Analyze code
    pub async fn analyze_code(&self, code: &str, language: &str) -> Result<AnalysisResult> {
        if !self.initialized {
            return Err(anyhow::anyhow!("AI Engine not initialized"));
        }

        match self.config.provider.as_str() {
            "openai" => self.analyze_code_with_openai(code, language).await,
            "local" => self.analyze_code_locally(code, language).await,
            _ => Ok(AnalysisResult {
                issues: vec![],
                suggestions: vec!["AI provider not supported".to_string()],
                complexity_score: 0.5,
            })
        }
    }

    /// Analyze code using OpenAI
    async fn analyze_code_with_openai(&self, code: &str, language: &str) -> Result<AnalysisResult> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenAI API key not configured"))?;

        let client = self.http_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;

        let prompt = format!(
            "Analyze this {} code for potential issues, bugs, and improvements. Provide specific suggestions with line numbers if possible.\n\nCode:\n{}",
            language,
            code
        );

        let openai_request = OpenAIRequest {
            model: match self.config.model_name.as_str() {
                "default" => "gpt-3.5-turbo",
                other => other,
            }.to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a senior software engineer performing code review. Identify bugs, performance issues, security concerns, and suggest improvements.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.3, // Lower temperature for more deterministic analysis
            max_tokens: Some(1000),
            stream: false,
        };

        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.openai.com/v1");
        let url = format!("{}/chat/completions", base_url);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", api_key))?
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json")
        );

        let response = client
            .post(&url)
            .headers(headers)
            .json(&openai_request)
            .send()
            .await?;

        if response.status().is_success() {
            let openai_response: OpenAIResponse = response.json().await?;
            
            let analysis_text = if let Some(choice) = openai_response.choices.first() {
                choice.message.content.clone()
            } else {
                "No analysis generated".to_string()
            };

            // Parse the analysis text to extract issues and suggestions
            let suggestions = self.parse_analysis_text(&analysis_text);

            Ok(AnalysisResult {
                issues: vec![], // TODO: Parse specific issues from AI response
                suggestions,
                complexity_score: self.calculate_complexity_score(code),
            })
        } else {
            let error_response: Result<OpenAIError, _> = response.json().await;
            match error_response {
                Ok(error) => Err(anyhow::anyhow!("OpenAI API error: {}", error.error.message)),
                Err(_) => Err(anyhow::anyhow!("OpenAI API request failed with status: {}", response.status())),
            }
        }
    }

    /// Analyze code using local heuristics
    async fn analyze_code_locally(&self, code: &str, language: &str) -> Result<AnalysisResult> {
        let mut suggestions = Vec::new();
        let mut issues = Vec::new();

        // Simple heuristic-based analysis
        match language.to_lowercase().as_str() {
            "rust" => {
                if code.contains("unwrap()") {
                    suggestions.push("Consider using ? operator or proper error handling instead of unwrap()".to_string());
                    issues.push(CodeIssue {
                        id: "unwrap_usage".to_string(),
                        severity: IssueSeverity::Warning,
                        message: "Consider using ? operator instead of unwrap()".to_string(),
                        line: 1,
                        column: 1,
                    });
                }
                if code.contains("clone()") && code.contains("&") {
                    suggestions.push("Consider using references instead of cloning".to_string());
                }
            },
            "python" => {
                if code.contains("except:") && !code.contains("except") {
                    suggestions.push("Avoid bare except clauses, catch specific exceptions".to_string());
                }
                if code.contains("== None") {
                    suggestions.push("Use 'is None' instead of '== None' for None comparisons".to_string());
                }
            },
            "javascript" => {
                if code.contains("==") && !code.contains("===") {
                    suggestions.push("Use === instead of == for type-safe comparisons".to_string());
                }
                if code.contains("var ") {
                    suggestions.push("Consider using let/const instead of var".to_string());
                }
            },
            _ => {}
        }

        // Calculate complexity score based on simple metrics
        let complexity_score = self.calculate_complexity_score(code);

        Ok(AnalysisResult {
            issues,
            suggestions,
            complexity_score,
        })
    }

    /// Parse AI analysis text to extract suggestions
    fn parse_analysis_text(&self, analysis_text: &str) -> Vec<String> {
        analysis_text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect()
    }

    /// Calculate a simple complexity score based on code metrics
    fn calculate_complexity_score(&self, code: &str) -> f32 {
        let lines: Vec<&str> = code.lines().collect();
        let total_lines = lines.len();
        if total_lines == 0 {
            return 0.0;
        }

        // Simple heuristic: more lines and deeper nesting = higher complexity
        let nested_blocks = code.matches('{').count() + code.matches('(').count();
        let complexity_ratio = nested_blocks as f32 / total_lines as f32;
        
        // Normalize to 0.0-1.0 range
        (complexity_ratio * 2.0).min(1.0)
    }

    /// Check if AI provider is available
    pub async fn ai_provider(&self) -> Result<String> {
        Ok(self.config.provider.clone())
    }

    /// Generate code completion (alias for generate_completion)
    pub async fn complete_code(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.generate_completion(request).await
    }

    /// Check if AI is available
    pub async fn is_available(&self) -> bool {
        self.initialized
    }

    /// Get model information
    pub async fn get_model_info(&self) -> Result<String> {
        Ok(format!("AI Engine with model: {}", self.config.model_name))
    }

    /// Learn from user feedback
    pub async fn learn_from_feedback(&self, _pattern_id: String, _accepted: bool) -> Result<()> {
        // Simplified learning - no-op for now
        Ok(())
    }
}

// Additional engines that some parts of the codebase expect
#[derive(Debug)]
pub struct RefactoringEngine;

impl RefactoringEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn analyze_for_refactoring(&self, _code: &str, _language: &str) -> Result<AnalysisResult> {
        Ok(AnalysisResult {
            issues: vec![],
            suggestions: vec!["Consider extracting this function".to_string()],
            complexity_score: 0.7,
        })
    }
}

#[derive(Debug)]
pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn get_performance_insights(&self, _code: &str, _language: &str) -> Result<AnalysisResult> {
        Ok(AnalysisResult {
            issues: vec![],
            suggestions: vec!["Consider optimizing this loop".to_string()],
            complexity_score: 0.6,
        })
    }
}

#[derive(Debug)]
pub struct SecurityAnalyzer;

impl SecurityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_code_security(&self, _code: &str, _language: &str) -> Result<AnalysisResult> {
        Ok(AnalysisResult {
            issues: vec![],
            suggestions: vec!["Consider input validation".to_string()],
            complexity_score: 0.8,
        })
    }
}