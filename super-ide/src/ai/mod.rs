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
use std::num::NonZeroUsize;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            base_url: None,
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
    pub file_path: Option<String>,
    pub rule_id: Option<String>,
    pub fix_suggestion: Option<String>,
    pub documentation_url: Option<String>,
}

/// Bug prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugPrediction {
    pub line: usize,
    pub column: usize,
    pub bug_type: BugType,
    pub confidence: f32,
    pub description: String,
    pub fix_suggestion: String,
    pub severity: IssueSeverity,
}

/// Types of bugs that can be predicted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BugType {
    NullPointer,
    ArrayIndexOutOfBounds,
    DivideByZero,
    MemoryLeak,
    RaceCondition,
    Deadlock,
    PerformanceIssue,
    SecurityVulnerability,
    LogicError,
    ResourceLeak,
}

/// Code smell detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSmell {
    pub id: String,
    pub name: String,
    pub description: String,
    pub line: usize,
    pub column: usize,
    pub severity: IssueSeverity,
    pub refactoring_suggestion: String,
}

/// Security vulnerability finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub id: String,
    pub cwe_id: Option<String>,
    pub title: String,
    pub description: String,
    pub severity: IssueSeverity,
    pub line: usize,
    pub column: usize,
    pub recommendation: String,
    pub cve_references: Vec<String>,
}

/// Code analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<String>,
    pub complexity_score: f32,
    pub bug_predictions: Vec<BugPrediction>,
    pub code_smells: Vec<CodeSmell>,
    pub security_vulnerabilities: Vec<SecurityVulnerability>,
    pub performance_insights: Vec<String>,
    pub maintainability_score: f32,
}

/// Debug session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub session_id: String,
    pub file_path: String,
    pub breakpoints: Vec<Breakpoint>,
    pub current_line: Option<usize>,
    pub variables: Vec<DebugVariable>,
    pub call_stack: Vec<StackFrame>,
    pub is_active: bool,
}

/// Breakpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: String,
    pub line: usize,
    pub column: usize,
    pub enabled: bool,
    pub condition: Option<String>,
    pub hit_count: u32,
}

/// Variable information for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugVariable {
    pub name: String,
    pub value: String,
    pub type_name: String,
    pub scope: VariableScope,
    pub is_changed: bool,
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub local_variables: Vec<DebugVariable>,
}

/// Variable scope in debugging context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableScope {
    Local,
    Global,
    Parameter,
    Field,
}

/// Code explanation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExplanationRequest {
    pub code: String,
    pub language: String,
    pub context: Option<String>,
    pub explanation_level: ExplanationLevel,
}

/// Levels of code explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExplanationLevel {
    Basic,
    Detailed,
    Expert,
}

/// Code explanation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExplanation {
    pub summary: String,
    pub explanation: String,
    pub key_concepts: Vec<String>,
    pub complexity_analysis: String,
    pub suggestions: Vec<String>,
}

/// Performance analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub optimization_suggestions: Vec<String>,
    pub complexity_analysis: CodeComplexity,
    pub memory_usage: Option<MemoryAnalysis>,
}

/// Performance bottleneck information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub line: usize,
    pub issue_type: PerformanceIssue,
    pub description: String,
    pub impact_score: f32,
    pub fix_suggestion: String,
}

/// Types of performance issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceIssue {
    SlowLoop,
    MemoryIntensive,
    IOBound,
    CPUIntensive,
    InefficientAlgorithm,
    MemoryLeak,
}

/// Memory analysis information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAnalysis {
    pub estimated_usage: usize,
    pub peak_usage: usize,
    pub allocation_sites: Vec<AllocationSite>,
}

/// Memory allocation site information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSite {
    pub line: usize,
    pub allocation_count: usize,
    pub total_size: usize,
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
    pub cursor_position: Option<(usize, usize)>,
    pub text_before_cursor: String,
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
    analysis_cache: Arc<RwLock<lru::LruCache<String, AnalysisResult>>>,
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
            request_cache: Arc::new(RwLock::new(lru::LruCache::new(NonZeroUsize::new(100).unwrap()))),
            analysis_cache: Arc::new(RwLock::new(lru::LruCache::new(NonZeroUsize::new(50).unwrap()))),
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
            let status = response.status();
            let error_response: Result<OpenAIError, _> = response.json().await;
            match error_response {
                Ok(error) => Err(anyhow::anyhow!("OpenAI API error: {}", error.error.message)),
                Err(_) => Err(anyhow::anyhow!("OpenAI API request failed with status: {}", status)),
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
                bug_predictions: vec![],
                code_smells: vec![],
                security_vulnerabilities: vec![],
                performance_insights: vec![],
                maintainability_score: 0.5,
            })
        }
    }

    /// Comprehensive analysis using OpenAI
    async fn comprehensive_analysis_with_openai(&self, code: &str, language: &str, file_path: Option<&str>) -> Result<AnalysisResult> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenAI API key not configured"))?;

        let client = self.http_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;

        let prompt = format!(
            r#"Perform comprehensive code analysis for this {} file{}.

Analyze for:
1. Code issues (bugs, errors, warnings)
2. Code smells and anti-patterns
3. Security vulnerabilities
4. Performance issues
5. Maintainability concerns
6. Best practice violations

Provide specific, actionable feedback with line numbers where possible.

Code:
{}"#,
            language,
            if let Some(path) = file_path { format!(" at {}", path) } else { "".to_string() },
            code
        );

        let openai_request = OpenAIRequest {
            model: match self.config.model_name.as_str() {
                "default" => "gpt-4",
                other => other,
            }.to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a senior software engineer and security expert performing comprehensive code review. Provide detailed analysis with specific line numbers and actionable recommendations.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.2,
            max_tokens: Some(2000),
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

            // Parse the comprehensive analysis
            let (issues, suggestions, bug_predictions, code_smells, security_vulns) = 
                self.parse_comprehensive_analysis(&analysis_text, language);

            Ok(AnalysisResult {
                issues,
                suggestions,
                complexity_score: self.calculate_complexity_score(code),
                bug_predictions,
                code_smells,
                security_vulnerabilities: security_vulns,
                performance_insights: vec!["Consider optimizing complex functions".to_string()],
                maintainability_score: self.calculate_maintainability_score(code),
            })
        } else {
            let status = response.status();
            let error_response: Result<OpenAIError, _> = response.json().await;
            match error_response {
                Ok(error) => Err(anyhow::anyhow!("OpenAI API error: {}", error.error.message)),
                Err(_) => Err(anyhow::anyhow!("OpenAI API request failed with status: {}", status)),
            }
        }
    }

    /// Comprehensive local analysis
    async fn comprehensive_local_analysis(&self, code: &str, language: &str, file_path: Option<&str>) -> Result<AnalysisResult> {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut bug_predictions = Vec::new();
        let mut code_smells = Vec::new();
        let mut security_vulnerabilities = Vec::new();

        let lines: Vec<&str> = code.lines().collect();
        
        // Language-specific analysis
        match language.to_lowercase().as_str() {
            "rust" => {
                for (line_num, line) in lines.iter().enumerate() {
                    let line_num = line_num + 1;
                    
                    // Bug prediction patterns
                    if line.contains("unwrap()") {
                        bug_predictions.push(BugPrediction {
                            line: line_num,
                            column: line.find("unwrap").unwrap_or(0) + 1,
                            bug_type: BugType::LogicError,
                            confidence: 0.8,
                            description: "Potential panic if unwrap() fails".to_string(),
                            fix_suggestion: "Use ? operator or proper error handling".to_string(),
                            severity: IssueSeverity::Warning,
                        });
                    }
                    
                    if line.contains("clone()") && line.contains("&") {
                        code_smells.push(CodeSmell {
                            id: format!("rust_clone_ref_{}", line_num),
                            name: "Unnecessary Clone".to_string(),
                            description: "Cloning a reference is usually unnecessary".to_string(),
                            line: line_num,
                            column: line.find("clone").unwrap_or(0) + 1,
                            severity: IssueSeverity::Info,
                            refactoring_suggestion: "Consider using references directly".to_string(),
                        });
                    }
                    
                    // Security patterns
                    if line.contains("expect()") {
                        security_vulnerabilities.push(SecurityVulnerability {
                            id: format!("rust_expect_{}", line_num),
                            cwe_id: Some("CWE-754".to_string()),
                            title: "Improper Check for Exceptional Conditions".to_string(),
                            description: "expect() can panic and should be used carefully".to_string(),
                            severity: IssueSeverity::Warning,
                            line: line_num,
                            column: line.find("expect").unwrap_or(0) + 1,
                            recommendation: "Use proper error handling instead of expect()".to_string(),
                            cve_references: vec![],
                        });
                    }
                }
                
                suggestions.push("Consider using Result types for better error handling".to_string());
                suggestions.push("Use references instead of cloning where possible".to_string());
            },
            "python" => {
                for (line_num, line) in lines.iter().enumerate() {
                    let line_num = line_num + 1;
                    
                    if line.contains("except:") && !line.contains("except") {
                        issues.push(CodeIssue {
                            id: format!("python_bare_except_{}", line_num),
                            severity: IssueSeverity::Warning,
                            message: "Avoid bare except clauses".to_string(),
                            line: line_num,
                            column: line.find("except").unwrap_or(0) + 1,
                            file_path: file_path.map(|s| s.to_string()),
                            rule_id: Some("B017".to_string()),
                            fix_suggestion: Some("Catch specific exceptions".to_string()),
                            documentation_url: Some("https://docs.python.org/3/tutorial/errors.html".to_string()),
                        });
                    }
                    
                    if line.contains("eval(") {
                        security_vulnerabilities.push(SecurityVulnerability {
                            id: format!("python_eval_{}", line_num),
                            cwe_id: Some("CWE-95".to_string()),
                            title: "Code Injection".to_string(),
                            description: "eval() can execute arbitrary code and is a security risk".to_string(),
                            severity: IssueSeverity::Critical,
                            line: line_num,
                            column: line.find("eval").unwrap_or(0) + 1,
                            recommendation: "Use ast.literal_eval() or avoid dynamic code execution".to_string(),
                            cve_references: vec![],
                        });
                    }
                }
            },
            "javascript" => {
                for (line_num, line) in lines.iter().enumerate() {
                    let line_num = line_num + 1;
                    
                    if line.contains("==") && !line.contains("===") {
                        issues.push(CodeIssue {
                            id: format!("js_eqeq_{}", line_num),
                            severity: IssueSeverity::Warning,
                            message: "Use === instead of ==".to_string(),
                            line: line_num,
                            column: line.find("==").unwrap_or(0) + 1,
                            file_path: file_path.map(|s| s.to_string()),
                            rule_id: Some("eqeqeq".to_string()),
                            fix_suggestion: Some("Use === for type-safe comparison".to_string()),
                            documentation_url: Some("https://developer.mozilla.org/en-US/docs/Web/JavaScript/Equality_comparisons_and_sameness".to_string()),
                        });
                    }
                    
                    if line.contains("var ") {
                        code_smells.push(CodeSmell {
                            id: format!("js_var_{}", line_num),
                            name: "Deprecated var Declaration".to_string(),
                            description: "var is deprecated, use let or const instead".to_string(),
                            line: line_num,
                            column: line.find("var").unwrap_or(0) + 1,
                            severity: IssueSeverity::Info,
                            refactoring_suggestion: "Replace var with let or const".to_string(),
                        });
                    }
                }
            },
            _ => {}
        }

        Ok(AnalysisResult {
            issues,
            suggestions,
            complexity_score: self.calculate_complexity_score(code),
            bug_predictions,
            code_smells,
            security_vulnerabilities,
            performance_insights: vec!["Consider code optimization opportunities".to_string()],
            maintainability_score: self.calculate_maintainability_score(code),
        })
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
                bug_predictions: vec![],
                code_smells: vec![],
                security_vulnerabilities: vec![],
                performance_insights: vec![],
                maintainability_score: self.calculate_maintainability_score(code),
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
                        file_path: todo!(),
                        rule_id: todo!(),
                        fix_suggestion: todo!(),
                        documentation_url: todo!(),
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
            bug_predictions: vec![],
            code_smells: vec![],
            security_vulnerabilities: vec![],
            performance_insights: vec![],
            maintainability_score: self.calculate_maintainability_score(code),
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

    /// Calculate maintainability score based on code metrics
    fn calculate_maintainability_score(&self, code: &str) -> f32 {
        let lines: Vec<&str> = code.lines().collect();
        let total_lines = lines.len();
        if total_lines == 0 {
            return 1.0;
        }

        // Factors that affect maintainability
        let avg_line_length = code.len() as f32 / total_lines as f32;
        let comment_ratio = code.matches("//").count() as f32 / total_lines as f32;
        let function_count = code.matches("fn ").count() + code.matches("function ").count() + code.matches("def ").count();
        let avg_function_length = total_lines as f32 / (function_count.max(1) as f32);
        
        // Calculate score (0.0-1.0, higher is better)
        let mut score = 1.0;
        
        // Penalize very long lines
        if avg_line_length > 100.0 {
            score -= 0.2;
        } else if avg_line_length > 80.0 {
            score -= 0.1;
        }
        
        // Reward good comment ratio
        if comment_ratio > 0.1 && comment_ratio < 0.3 {
            score += 0.1;
        }
        
        // Penalize very long functions
        if avg_function_length > 50.0 {
            score -= 0.2;
        } else if avg_function_length > 30.0 {
            score -= 0.1;
        }
        
        // Normalize score
        score.max(0.0).min(1.0)
    }

    /// Parse comprehensive analysis from AI response
    fn parse_comprehensive_analysis(&self, analysis_text: &str, language: &str) -> (
        Vec<CodeIssue>, 
        Vec<String>, 
        Vec<BugPrediction>, 
        Vec<CodeSmell>, 
        Vec<SecurityVulnerability>
    ) {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut bug_predictions = Vec::new();
        let mut code_smells = Vec::new();
        let mut security_vulnerabilities = Vec::new();

        let lines: Vec<&str> = analysis_text.lines().collect();
        let mut line_num = 1;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Parse different types of findings
            if trimmed.starts_with("Issue:") || trimmed.starts_with("Warning:") || trimmed.starts_with("Error:") {
                let severity = if trimmed.starts_with("Error:") {
                    IssueSeverity::Error
                } else if trimmed.starts_with("Warning:") {
                    IssueSeverity::Warning
                } else {
                    IssueSeverity::Info
                };

                issues.push(CodeIssue {
                    id: format!("ai_issue_{}", line_num),
                    severity,
                    message: trimmed.to_string(),
                    line: line_num,
                    column: 1,
                    file_path: None,
                    rule_id: None,
                    fix_suggestion: None,
                    documentation_url: None,
                });
            } else if trimmed.starts_with("Suggestion:") {
                suggestions.push(trimmed.to_string());
            } else if trimmed.starts_with("Bug:") {
                bug_predictions.push(BugPrediction {
                    line: line_num,
                    column: 1,
                    bug_type: BugType::LogicError,
                    confidence: 0.7,
                    description: trimmed.to_string(),
                    fix_suggestion: "Review code logic".to_string(),
                    severity: IssueSeverity::Warning,
                });
            } else if trimmed.starts_with("Smell:") {
                code_smells.push(CodeSmell {
                    id: format!("ai_smell_{}", line_num),
                    name: "Code Smell".to_string(),
                    description: trimmed.to_string(),
                    line: line_num,
                    column: 1,
                    severity: IssueSeverity::Info,
                    refactoring_suggestion: "Consider refactoring".to_string(),
                });
            } else if trimmed.starts_with("Security:") {
                security_vulnerabilities.push(SecurityVulnerability {
                    id: format!("ai_security_{}", line_num),
                    cwe_id: None,
                    title: "Security Issue".to_string(),
                    description: trimmed.to_string(),
                    severity: IssueSeverity::Warning,
                    line: line_num,
                    column: 1,
                    recommendation: "Review security implications".to_string(),
                    cve_references: vec![],
                });
            }

            line_num += 1;
        }

        (issues, suggestions, bug_predictions, code_smells, security_vulnerabilities)
    }

    /// Performance analysis using OpenAI
    async fn performance_analysis_with_openai(&self, code: &str, language: &str) -> Result<PerformanceAnalysis> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenAI API key not configured"))?;

        let client = self.http_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;

        let prompt = format!(
            "Analyze the performance of this {} code. Identify bottlenecks, inefficiencies, and optimization opportunities.\n\nCode:\n{}",
            language,
            code
        );

        let openai_request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a performance optimization expert. Analyze code for performance issues and provide specific optimization suggestions.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.2,
            max_tokens: Some(1500),
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

            // Parse performance analysis
            let lines: Vec<&str> = analysis_text.lines().collect();
            let mut bottlenecks = Vec::new();
            let mut optimization_suggestions = Vec::new();

            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("Bottleneck:") {
                    bottlenecks.push(PerformanceBottleneck {
                        line: i + 1,
                        issue_type: PerformanceIssue::CPUIntensive,
                        description: trimmed.to_string(),
                        impact_score: 0.7,
                        fix_suggestion: "Consider optimization".to_string(),
                    });
                } else if trimmed.starts_with("Optimization:") {
                    optimization_suggestions.push(trimmed.to_string());
                }
            }

            Ok(PerformanceAnalysis {
                bottlenecks,
                optimization_suggestions,
                complexity_analysis: CodeComplexity {
                    cyclomatic_complexity: self.calculate_complexity_score(code) as u32 * 10,
                    cognitive_complexity: (self.calculate_complexity_score(code) * 10.0) as u32,
                    lines_of_code: code.lines().count(),
                    maintainability_index: Some(self.calculate_maintainability_score(code)),
                    nested_depth: Some(code.matches('{').count() as u32),
                    line_count: Some(code.lines().count() as u32),
                    function_count: Some((code.matches("fn ").count() + code.matches("function ").count()) as u32),
                    complexity_score: Some(self.calculate_complexity_score(code)),
                },
                memory_usage: None,
            })
        } else {
            Err(anyhow::anyhow!("Performance analysis failed"))
        }
    }

    /// Local performance analysis
    async fn local_performance_analysis(&self, code: &str, language: &str) -> Result<PerformanceAnalysis> {
        let mut bottlenecks = Vec::new();
        let mut optimization_suggestions = Vec::new();

        let lines: Vec<&str> = code.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            
            // Simple heuristic-based performance analysis
            if line.contains("for ") && line.contains("range(") {
                bottlenecks.push(PerformanceBottleneck {
                    line: line_num,
                    issue_type: PerformanceIssue::SlowLoop,
                    description: "Nested loop detected".to_string(),
                    impact_score: 0.6,
                    fix_suggestion: "Consider using more efficient data structures".to_string(),
                });
            }
            
            if line.contains("clone()") {
                optimization_suggestions.push(format!("Line {}: Consider avoiding unnecessary cloning", line_num));
            }
        }

        optimization_suggestions.push("Use appropriate data structures for better performance".to_string());

        Ok(PerformanceAnalysis {
            bottlenecks,
            optimization_suggestions,
            complexity_analysis: CodeComplexity {
                cyclomatic_complexity: self.calculate_complexity_score(code) as u32 * 10,
                cognitive_complexity: (self.calculate_complexity_score(code) * 10.0) as u32,
                lines_of_code: code.lines().count(),
                maintainability_index: Some(self.calculate_maintainability_score(code)),
                nested_depth: Some(code.matches('{').count() as u32),
                line_count: Some(code.lines().count() as u32),
                function_count: Some((code.matches("fn ").count() + code.matches("function ").count()) as u32),
                complexity_score: Some(self.calculate_complexity_score(code)),
            },
            memory_usage: None,
        })
    }

    /// Explain code using OpenAI
    async fn explain_code_with_openai(&self, request: CodeExplanationRequest) -> Result<CodeExplanation> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenAI API key not configured"))?;

        let client = self.http_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;

        let level_instruction = match request.explanation_level {
            ExplanationLevel::Basic => "Provide a simple, beginner-friendly explanation",
            ExplanationLevel::Detailed => "Provide a comprehensive explanation with technical details",
            ExplanationLevel::Expert => "Provide an expert-level analysis with advanced concepts",
        };

        let prompt = format!(
            "{} for this {} code. Explain what it does, how it works, and any important concepts.\n\nContext: {}\n\nCode:\n{}",
            level_instruction,
            request.language,
            request.context.as_deref().unwrap_or(""),
            request.code
        );

        let openai_request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are an expert programming educator. Provide clear, helpful code explanations.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.3,
            max_tokens: Some(1500),
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
            
            let explanation = if let Some(choice) = openai_response.choices.first() {
                choice.message.content.clone()
            } else {
                "No explanation generated".to_string()
            };

            Ok(CodeExplanation {
                summary: "Code explanation generated".to_string(),
                explanation,
                key_concepts: vec!["Functionality".to_string(), "Implementation".to_string()],
                complexity_analysis: "Moderate complexity".to_string(),
                suggestions: vec!["Consider adding documentation".to_string()],
            })
        } else {
            Err(anyhow::anyhow!("Code explanation failed"))
        }
    }

    /// Explain code using local knowledge
    async fn explain_code_locally(&self, request: CodeExplanationRequest) -> Result<CodeExplanation> {
        let summary = format!("This {} code performs specific functionality", request.language);
        
        let explanation = match request.language.to_lowercase().as_str() {
            "rust" => "This Rust code uses ownership and borrowing for memory safety",
            "python" => "This Python code uses dynamic typing and duck typing",
            "javascript" => "This JavaScript code uses prototype-based inheritance",
            _ => "This code implements specific functionality"
        };

        Ok(CodeExplanation {
            summary,
            explanation: explanation.to_string(),
            key_concepts: vec!["Language Features".to_string(), "Logic Flow".to_string()],
            complexity_analysis: "Code complexity analysis based on structure".to_string(),
            suggestions: vec!["Add comments for clarity".to_string()],
        })
    }

    /// Generate unit tests using OpenAI
    async fn generate_tests_with_openai(&self, code: &str, language: &str) -> Result<String> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenAI API key not configured"))?;

        let client = self.http_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;

        let prompt = format!(
            "Generate comprehensive unit tests for this {} code. Include edge cases and error scenarios.\n\nCode:\n{}",
            language,
            code
        );

        let openai_request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a testing expert. Generate comprehensive, well-structured unit tests with proper assertions.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.3,
            max_tokens: Some(2000),
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
            
            let test_code = if let Some(choice) = openai_response.choices.first() {
                choice.message.content.clone()
            } else {
                "// No tests generated".to_string()
            };

            Ok(test_code)
        } else {
            Err(anyhow::anyhow!("Test generation failed"))
        }
    }

    /// Generate unit tests using local patterns
    async fn generate_tests_locally(&self, code: &str, language: &str) -> Result<String> {
        let test_template = match language.to_lowercase().as_str() {
            "rust" => r#"#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Add your test cases here
        assert!(true);
    }

    #[test]
    fn test_edge_cases() {
        // Test edge cases and error scenarios
        assert!(true);
    }
}"#,
            "python" => r#"import unittest

class TestCode(unittest.TestCase):
    def test_basic_functionality(self):
        # Add your test cases here
        self.assertTrue(True)

    def test_edge_cases(self):
        # Test edge cases and error scenarios
        self.assertTrue(True)

if __name__ == '__main__':
    unittest.main()"#,
            "javascript" => r#"describe('Test Suite', () => {
    test('basic functionality', () => {
        expect(true).toBe(true);
    });

    test('edge cases', () => {
        expect(true).toBe(true);
    });
});"#,
            _ => "// Tests not supported for this language"
        };

        Ok(test_template.to_string())
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

    /// Advanced code analysis with enhanced features
    pub async fn advanced_analyze_code(&self, code: &str, language: &str, file_path: Option<&str>) -> Result<AnalysisResult> {
        if !self.initialized {
            return Err(anyhow::anyhow!("AI Engine not initialized"));
        }

        // Check analysis cache first
        let cache_key = format!("{}:{}:{}", language, file_path.unwrap_or(""), code.len());
        {
            let cache = self.analysis_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Perform comprehensive analysis
        let mut analysis = match self.config.provider.as_str() {
            "openai" => self.comprehensive_analysis_with_openai(code, language, file_path).await?,
            "local" => self.comprehensive_local_analysis(code, language, file_path).await?,
            _ => AnalysisResult {
                issues: vec![],
                suggestions: vec!["AI provider not supported".to_string()],
                complexity_score: 0.5,
                bug_predictions: vec![],
                code_smells: vec![],
                security_vulnerabilities: vec![],
                performance_insights: vec![],
                maintainability_score: 0.5,
            }
        };

        // Cache the result
        {
            let mut cache = self.analysis_cache.write().await;
            cache.put(cache_key, analysis.clone());
        }

        Ok(analysis)
    }

    /// Predict potential bugs in code
    pub async fn predict_bugs(&self, code: &str, language: &str) -> Result<Vec<BugPrediction>> {
        let analysis = self.advanced_analyze_code(code, language, None).await?;
        Ok(analysis.bug_predictions)
    }

    /// Detect security vulnerabilities
    pub async fn analyze_security(&self, code: &str, language: &str) -> Result<Vec<SecurityVulnerability>> {
        let analysis = self.advanced_analyze_code(code, language, None).await?;
        Ok(analysis.security_vulnerabilities)
    }

    /// Analyze code performance
    pub async fn analyze_performance(&self, code: &str, language: &str) -> Result<PerformanceAnalysis> {
        match self.config.provider.as_str() {
            "openai" => self.performance_analysis_with_openai(code, language).await,
            "local" => self.local_performance_analysis(code, language).await,
            _ => Err(anyhow::anyhow!("AI provider not supported"))
        }
    }

    /// Explain code functionality
    pub async fn explain_code(&self, request: CodeExplanationRequest) -> Result<CodeExplanation> {
        match self.config.provider.as_str() {
            "openai" => self.explain_code_with_openai(request).await,
            "local" => self.explain_code_locally(request).await,
            _ => Err(anyhow::anyhow!("AI provider not supported"))
        }
    }

    /// Generate unit tests for code
    pub async fn generate_tests(&self, code: &str, language: &str) -> Result<String> {
        match self.config.provider.as_str() {
            "openai" => self.generate_tests_with_openai(code, language).await,
            "local" => self.generate_tests_locally(code, language).await,
            _ => Err(anyhow::anyhow!("AI provider not supported"))
        }
    }

    /// Suggest code improvements
    pub async fn suggest_improvements(&self, code: &str, language: &str) -> Result<Vec<String>> {
        let analysis = self.advanced_analyze_code(code, language, None).await?;
        Ok(analysis.suggestions)
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
            bug_predictions: vec![],
            code_smells: vec![],
            security_vulnerabilities: vec![],
            performance_insights: vec![],
            maintainability_score: 0.8,
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
            bug_predictions: vec![],
            code_smells: vec![],
            security_vulnerabilities: vec![],
            performance_insights: vec![],
            maintainability_score: 0.7,
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
            bug_predictions: vec![],
            code_smells: vec![],
            security_vulnerabilities: vec![],
            performance_insights: vec![],
            maintainability_score: 0.9,
        })
    }
}
