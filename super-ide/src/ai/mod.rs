//! AI Engine Module
//!
//! This module provides advanced AI-powered features for the IDE including:
//! - Enhanced semantic analysis with AST parsing
//! - Multi-token completion for entire code blocks
//! - Context-aware intelligence with project insights
//! - Advanced error detection for logic and performance issues
//! - Learning algorithms for user preference adaptation
//! - Security and performance analysis capabilities
//! - Intelligent refactoring with automated code improvements
//! - Multi-provider AI integration (OpenAI, Claude, etc.)
//! - Plugin architecture for extensible AI capabilities
//! - Custom model training for personalized AI assistants

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// Note: PathBuf, Path, OsStr imports reserved for future file operations
// use std::path::{PathBuf, Path};
// use std::ffi::OsStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tree_sitter::{Language, Parser, Tree, Node};

/// AI Provider abstraction for multiple AI services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiProvider {
    Local,
    OpenAI,
    Anthropic,
    Google,
    MiniMax,
    Custom(String),
}

/// AI model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: AiProvider,
    pub model_name: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub context_window: u32,
}

/// AI provider client trait
#[async_trait::async_trait]
pub trait AiProviderClient: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn analyze_code(&self, code: &str, language: &str) -> Result<AnalysisResult>;
    async fn get_embeddings(&self, text: &str) -> Result<Vec<f32>>;
    fn supports_language(&self, language: &str) -> bool;
    fn get_model_info(&self) -> ModelInfo;
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub provider: AiProvider,
    pub context_window: u32,
    pub supports_function_calling: bool,
    pub supports_embeddings: bool,
    pub pricing_per_token: Option<f64>,
}

/// OpenAI provider implementation
pub struct OpenAiProvider {
    config: ModelConfig,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Parse AI response JSON into code issues
    pub fn parse_analysis_response(&self, response: &serde_json::Value) -> Result<Vec<CodeIssue>> {
        let mut issues = Vec::new();

        // Extract issues from AI response
        if let Some(choices) = response.get("choices").and_then(|c| c.as_array()) {
            for choice in choices {
                if let Some(message) = choice.get("message").and_then(|m| m.get("content")) {
                    if let Some(content_str) = message.as_str() {
                        // Simple parsing of AI response for issues
                        if content_str.contains("warning") || content_str.contains("issue") {
                            issues.push(CodeIssue {
                                id: Uuid::new_v4().to_string(),
                                severity: IssueSeverity::Info,
                                message: format!("AI Analysis: {}", content_str.lines().next().unwrap_or("Issue detected")),
                                line: 1,
                                column: 1,
                            });
                        }
                    }
                }
            }
        }

        Ok(issues)
    }
}

#[async_trait::async_trait]
impl AiProviderClient for OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenAI API key not configured"))?;

        let request_body = serde_json::json!({
            "model": self.config.model_name,
            "messages": [
                {
                    "role": "user",
                    "content": format!("Complete this code in {}:\n\n{}", request.language, request.context)
                }
            ],
            "max_tokens": request.max_tokens.unwrap_or(self.config.max_tokens),
            "temperature": self.config.temperature,
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("OpenAI API error: {}", response.status()));
        }

        let response_json: serde_json::Value = response.json().await?;
        let text = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(CompletionResponse {
            text,
            confidence: 0.85,
            suggestions: vec![],
        })
    }

    async fn analyze_code(&self, code: &str, language: &str) -> Result<AnalysisResult> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenAI API key not configured"))?;

        let prompt = format!(
            "Analyze this {} code for issues, bugs, and improvements:\n\n{}\n\nProvide analysis in JSON format with issues and suggestions.",
            language, code
        );

        let request_body = serde_json::json!({
            "model": self.config.model_name,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": 1000,
            "temperature": 0.3,
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("OpenAI API error: {}", response.status()));
        }

        let response_json: serde_json::Value = response.json().await?;

        // Parse the analysis response from AI provider
        let issues = self.parse_analysis_response(&response_json).unwrap_or_else(|_| {
            vec![CodeIssue {
                id: Uuid::new_v4().to_string(),
                severity: IssueSeverity::Info,
                message: "AI-powered analysis completed".to_string(),
                line: 1,
                column: 1,
            }]
        });

        Ok(AnalysisResult {
            issues,
            suggestions: vec![],
            complexity_score: 0.5,
        })
    }

    async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("Embeddings not supported by OpenAI chat models"))
    }

    fn supports_language(&self, _language: &str) -> bool {
        true // OpenAI models are generally language-agnostic
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: self.config.model_name.clone(),
            provider: AiProvider::OpenAI,
            context_window: self.config.context_window,
            supports_function_calling: true,
            supports_embeddings: false,
            pricing_per_token: Some(0.002), // Approximate pricing
        }
    }
}

/// Anthropic/Claude provider implementation
pub struct AnthropicProvider {
    config: ModelConfig,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl AiProviderClient for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Anthropic API key not configured"))?;

        let request_body = serde_json::json!({
            "model": self.config.model_name,
            "max_tokens": request.max_tokens.unwrap_or(self.config.max_tokens),
            "temperature": self.config.temperature,
            "messages": [
                {
                    "role": "user",
                    "content": format!("Complete this code in {}:\n\n{}", request.language, request.context)
                }
            ]
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Anthropic API error: {}", response.status()));
        }

        let response_json: serde_json::Value = response.json().await?;
        let text = response_json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(CompletionResponse {
            text,
            confidence: 0.9,
            suggestions: vec![],
        })
    }

    async fn analyze_code(&self, code: &str, language: &str) -> Result<AnalysisResult> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Anthropic API key not configured"))?;

        let prompt = format!(
            "Analyze this {} code for issues, bugs, and improvements:\n\n{}\n\nProvide detailed analysis.",
            language, code
        );

        let request_body = serde_json::json!({
            "model": self.config.model_name,
            "max_tokens": 1000,
            "temperature": 0.3,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Anthropic API error: {}", response.status()));
        }

        let response_json: serde_json::Value = response.json().await?;
        let analysis_text = response_json["content"][0]["text"]
            .as_str()
            .unwrap_or("");

        // Parse analysis (simplified)
        let issues = vec![CodeIssue {
            id: Uuid::new_v4().to_string(),
            severity: IssueSeverity::Info,
            message: format!("Anthropic analysis: {}", analysis_text.chars().take(100).collect::<String>()),
            line: 1,
            column: 1,
        }];

        Ok(AnalysisResult {
            issues,
            suggestions: vec![],
            complexity_score: 0.5,
        })
    }

    async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("Embeddings not supported by Anthropic"))
    }

    fn supports_language(&self, _language: &str) -> bool {
        true
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: self.config.model_name.clone(),
            provider: AiProvider::Anthropic,
            context_window: self.config.context_window,
            supports_function_calling: false,
            supports_embeddings: false,
            pricing_per_token: Some(0.008), // Approximate pricing
        }
    }
}

/// Local AI provider (placeholder for local models)
pub struct LocalProvider {
    config: ModelConfig,
}

impl LocalProvider {
    pub fn new(config: ModelConfig) -> Self {
        Self { config }
    }

    /// Generate Rust-specific suggestions
    pub fn generate_rust_suggestions(&self, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if context.contains("fn ") {
            suggestions.push("-> Result<(), Box<dyn std::error::Error>>".to_string());
        }
        if context.contains("struct ") {
            suggestions.push("{".to_string());
        }
        if context.contains("impl ") {
            suggestions.push("{".to_string());
        }
        
        suggestions
    }

    /// Generate Python-specific suggestions
    pub fn generate_python_suggestions(&self, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if context.contains("def ") {
            suggestions.push(" -> None:".to_string());
        }
        if context.contains("class ") {
            suggestions.push("pass".to_string());
        }
        if context.contains("if ") {
            suggestions.push("pass".to_string());
        }
        
        suggestions
    }
}

#[async_trait::async_trait]
impl AiProviderClient for LocalProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Use request information to provide better local completions
        let language = &request.language;
        let context = &request.context;
        
        // Generate basic suggestions based on context
        let suggestions = match language.as_str() {
            "rust" => self.generate_rust_suggestions(context),
            "python" => self.generate_python_suggestions(context),
            _ => vec!["// Basic completion".to_string()],
        };

        Ok(CompletionResponse {
            text: format!("// Local completion for {} language", language),
            confidence: 0.6,
            suggestions,
        })
    }



    async fn analyze_code(&self, _code: &str, _language: &str) -> Result<AnalysisResult> {
        Ok(AnalysisResult {
            issues: vec![],
            suggestions: vec![],
            complexity_score: 0.0,
        })
    }

    async fn get_embeddings(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("Local embeddings not implemented"))
    }

    fn supports_language(&self, language: &str) -> bool {
        matches!(language, "rust" | "python") // Limited language support for local models
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: self.config.model_name.clone(),
            provider: AiProvider::Local,
            context_window: 2048, // Limited context for local models
            supports_function_calling: false,
            supports_embeddings: true,
            pricing_per_token: Some(0.0), // Free
        }
    }
}

/// AI Provider Manager - manages multiple AI providers
pub struct AiProviderManager {
    providers: HashMap<String, Box<dyn AiProviderClient>>,
    active_provider: String,
    fallback_provider: Option<String>,
}

impl std::fmt::Debug for AiProviderManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiProviderManager")
            .field("active_provider", &self.active_provider)
            .field("fallback_provider", &self.fallback_provider)
            .field("providers_count", &self.providers.len())
            .finish()
    }
}

impl AiProviderManager {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            active_provider: "local".to_string(),
            fallback_provider: None,
        }
    }

    pub fn register_provider(&mut self, name: String, provider: Box<dyn AiProviderClient>) {
        self.providers.insert(name, provider);
    }

    pub fn set_active_provider(&mut self, name: &str) -> Result<()> {
        if self.providers.contains_key(name) {
            self.active_provider = name.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Provider '{}' not found", name))
        }
    }

    pub fn set_fallback_provider(&mut self, name: Option<String>) -> Result<()> {
        if let Some(ref provider_name) = name {
            if !self.providers.contains_key(provider_name) {
                return Err(anyhow::anyhow!("Fallback provider '{}' not found", provider_name));
            }
        }
        self.fallback_provider = name;
        Ok(())
    }

    pub async fn complete_with_fallback(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Try active provider first
        if let Some(provider) = self.providers.get(&self.active_provider) {
            match provider.complete(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    eprintln!("Active provider failed: {}", e);
                    // Try fallback if available
                    if let Some(ref fallback_name) = self.fallback_provider {
                        if let Some(fallback_provider) = self.providers.get(fallback_name) {
                            if let Ok(response) = fallback_provider.complete(request).await {
                                return Ok(response);
                            }
                        }
                    }
                    return Err(e);
                }
            }
        }

        Err(anyhow::anyhow!("No active provider available"))
    }

    pub async fn analyze_with_fallback(&self, code: &str, language: &str) -> Result<AnalysisResult> {
        if let Some(provider) = self.providers.get(&self.active_provider) {
            match provider.analyze_code(code, language).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Try fallback
                    if let Some(ref fallback_name) = self.fallback_provider {
                        if let Some(fallback_provider) = self.providers.get(fallback_name) {
                            if let Ok(result) = fallback_provider.analyze_code(code, language).await {
                                return Ok(result);
                            }
                        }
                    }
                    return Err(e);
                }
            }
        }

        Err(anyhow::anyhow!("No active provider available"))
    }

    pub fn get_available_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn get_provider_info(&self, name: &str) -> Option<ModelInfo> {
        self.providers.get(name).map(|p| p.get_model_info())
    }

    pub fn get_active_provider(&self) -> &str {
        &self.active_provider
    }
}

/// MiniMax M2 provider implementation (both local and API)
pub struct MiniMaxProvider {
    config: ModelConfig,
    client: reqwest::Client,
    is_local: bool,
}

impl MiniMaxProvider {
    pub fn new(config: ModelConfig) -> Self {
        let is_local = config.api_key.is_none() && config.model_name.contains("local");
        Self {
            config,
            client: reqwest::Client::new(),
            is_local,
        }
    }

    /// Check if this is a local MiniMax model
    pub fn is_local(&self) -> bool {
        self.is_local
    }

    /// Get MiniMax API endpoint
    fn get_api_endpoint(&self) -> String {
        if self.is_local {
            // Local MiniMax model endpoint (would be configurable)
            "http://localhost:8000/v1/chat/completions".to_string()
        } else {
            // MiniMax cloud API endpoint
            "https://api.minimax.chat/v1/text/chatcompletion_v2".to_string()
        }
    }
}

#[async_trait::async_trait]
impl AiProviderClient for MiniMaxProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        if self.is_local {
            // Local MiniMax model completion
            self.complete_local(request).await
        } else {
            // Cloud MiniMax API completion
            self.complete_api(request).await
        }
    }

    async fn analyze_code(&self, code: &str, language: &str) -> Result<AnalysisResult> {
        let prompt = format!(
            "作为专业的代码分析助手，请分析以下{}代码中的问题、漏洞和改进建议：\n\n{}\n\n请提供详细的分析结果，包括发现的问题和改进建议。",
            language, code
        );

        let completion_request = CompletionRequest {
            prompt: "代码分析任务".to_string(),
            context: prompt,
            language: language.to_string(),
            max_tokens: Some(1500),
        };

        let response = self.complete(completion_request).await?;

        // Parse the analysis response (simplified for MiniMax)
        let issues = if response.text.contains("错误") || response.text.contains("问题") {
            vec![CodeIssue {
                id: Uuid::new_v4().to_string(),
                severity: IssueSeverity::Info,
                message: format!("MiniMax M2 分析: {}", response.text.chars().take(100).collect::<String>()),
                line: 1,
                column: 1,
            }]
        } else {
            vec![]
        };

        Ok(AnalysisResult {
            issues,
            suggestions: vec![],
            complexity_score: 0.5,
        })
    }

    async fn get_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        if self.is_local {
            // Local embeddings (would need local model support)
            Err(anyhow::anyhow!("Local MiniMax embeddings not implemented"))
        } else {
            // MiniMax API embeddings
            self.get_api_embeddings(text).await
        }
    }

    fn supports_language(&self, language: &str) -> bool {
        // MiniMax M2 supports multiple languages including Chinese
        matches!(language, "rust" | "python" | "javascript" | "typescript" | "go" | "java" | "cpp" | "c")
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: format!("MiniMax-{}", self.config.model_name),
            provider: AiProvider::MiniMax,
            context_window: 8192, // MiniMax M2 context window
            supports_function_calling: true,
            supports_embeddings: !self.is_local,
            pricing_per_token: if self.is_local { Some(0.0) } else { Some(0.001) }, // Approximate pricing
        }
    }
}

impl MiniMaxProvider {
    /// Complete using local MiniMax model
    async fn complete_local(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // This would integrate with a local MiniMax M2 model deployment
        // For now, return a placeholder response
        Ok(CompletionResponse {
            text: format!("// 本地 MiniMax M2 模型补全 - {}\n// 代码: {}", request.language, request.context),
            confidence: 0.7,
            suggestions: vec![
                format!("// MiniMax M2 建议的{}代码结构", request.language),
            ],
        })
    }

    /// Complete using MiniMax API
    async fn complete_api(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("MiniMax API key not configured"))?;

        // MiniMax API request format (based on their documentation)
        let request_body = serde_json::json!({
            "model": self.config.model_name,
            "messages": [
                {
                    "role": "user",
                    "content": format!("请作为专业的{}代码助手，补全以下代码：\n\n{}", request.language, request.context)
                }
            ],
            "max_tokens": request.max_tokens.unwrap_or(self.config.max_tokens),
            "temperature": self.config.temperature,
            "stream": false
        });

        let response = self.client
            .post(self.get_api_endpoint())
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("MiniMax API error: {}", response.status()));
        }

        let response_json: serde_json::Value = response.json().await?;

        // Parse MiniMax response format
        let text = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(CompletionResponse {
            text,
            confidence: 0.85,
            suggestions: vec![],
        })
    }

    /// Get embeddings from MiniMax API
    async fn get_api_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("MiniMax API key not configured"))?;

        let request_body = serde_json::json!({
            "model": "embo-01", // MiniMax embedding model
            "texts": [text],
            "type": "query"
        });

        let response = self.client
            .post("https://api.minimax.chat/v1/embeddings")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("MiniMax embeddings API error: {}", response.status()));
        }

        let response_json: serde_json::Value = response.json().await?;

        // Parse embeddings response
        let embeddings = response_json["vectors"][0]["data"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid embeddings response format"))?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embeddings)
    }
}

/// Factory function to create AI providers
pub fn create_provider(config: ModelConfig) -> Result<Box<dyn AiProviderClient>> {
    match config.provider {
        AiProvider::OpenAI => {
            Ok(Box::new(OpenAiProvider::new(config)))
        }
        AiProvider::Anthropic => {
            Ok(Box::new(AnthropicProvider::new(config)))
        }
        AiProvider::MiniMax => {
            Ok(Box::new(MiniMaxProvider::new(config)))
        }
        AiProvider::Local => {
            Ok(Box::new(LocalProvider::new(config)))
        }
        AiProvider::Google | AiProvider::Custom(_) => {
            Err(anyhow::anyhow!("Provider not implemented yet"))
        }
    }
}

/// Configuration for AI features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub enabled: bool,
    pub provider: String,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl From<&crate::config::Configuration> for AiConfig {
    fn from(config: &crate::config::Configuration) -> Self {
        Self {
            enabled: config.ai.enable_local_inference,
            provider: format!("{:?}", config.ai.provider).to_lowercase(),
            api_key: config.ai.api_key.clone(),
            model_name: config.ai.model_path.clone(),
            max_tokens: config.ai.max_tokens,
            temperature: config.ai.temperature,
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: "mock".to_string(),
            api_key: None,
            model_name: Some("demo-model".to_string()),
            max_tokens: 2048,
            temperature: 0.7,
        }
    }
}

/// AI completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub context: String,
    pub language: String,
    pub max_tokens: Option<u32>,
}

/// AI completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub confidence: f32,
    pub suggestions: Vec<String>,
}

/// Code analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<CodeSuggestion>,
    pub complexity_score: f32,
}

/// Code issue found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub line: u32,
    pub column: u32,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Code suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub id: String,
    pub message: String,
    pub code: String,
    pub confidence: f32,
}

/// Function information for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub line_start: u32,
    pub line_end: u32,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub complexity: f32,
    pub signature: Option<String>,
    pub docstring: Option<String>,
}

/// Parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub type_hint: Option<String>,
    pub is_mutable: bool,
}

/// Variable information for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub name: String,
    pub line: u32,
    pub column: u32,
    pub scope: String,
    pub variable_type: VariableType,
    pub is_declared: bool,
    pub var_type: Option<String>,
    pub is_mutable: bool,
}

/// Variable types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    Local,
    Global,
    Parameter,
    Constant,
}

/// Import information for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub module_path: String,
    pub imported_items: Vec<String>,
    pub line: u32,
    pub import_type: ImportType,
}

/// Import types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportType {
    Direct,
    Wildcard,
    Aliased,
}

/// Code complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeComplexity {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub line_count: u32,
    pub function_count: u32,
    pub complexity_score: f32,
    pub maintainability_index: f32,
    pub lines_of_code: u32,
    pub nested_depth: u32,
}

/// Complete code analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub functions: Vec<FunctionInfo>,
    pub variables: Vec<VariableInfo>,
    pub imports: Vec<ImportInfo>,
    pub complexity: CodeComplexity,
    pub metrics: HashMap<String, f32>,
}

/// Advanced AI Engine with superior intelligence
#[derive(Debug, Clone)]
pub struct AiEngine {
    config: Arc<RwLock<AiConfig>>,
    provider_manager: Arc<RwLock<AiProviderManager>>,
    learning_data: Arc<RwLock<HashMap<String, LearningData>>>,
    semantic_analyzer: Arc<RwLock<SemanticAnalyzer>>,
    _pattern_recognizer: Arc<RwLock<PatternRecognizer>>,
    context_analyzer: Arc<RwLock<ContextAnalyzer>>,
    security_analyzer: Arc<RwLock<SecurityAnalyzer>>,
    performance_analyzer: Arc<RwLock<PerformanceAnalyzer>>,
    refactoring_engine: Arc<RwLock<RefactoringEngine>>,
    senior_engineer_knowledge: Arc<RwLock<SeniorEngineerKnowledge>>,
    terminal_intelligence: Arc<RwLock<TerminalIntelligence>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LearningData {
    patterns: HashMap<String, f32>,
    feedback_count: u32,
    success_rate: f32,
    user_preferences: HashMap<String, f32>,
    project_contexts: HashMap<String, Vec<String>>,
    learning_history: Vec<LearningEvent>,
    adaptation_metrics: AdaptationMetrics,
    behavioral_patterns: BehavioralPatterns,
}

/// Enhanced Semantic Analyzer for deep code understanding
#[derive(Debug)]
pub struct SemanticAnalyzer {
    ast_cache: HashMap<String, SyntaxTree>,
    symbol_table: HashMap<String, SymbolInfo>,
    dependency_graph: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SyntaxTree {
    pub root: SyntaxNode,
    pub symbols: HashMap<String, SymbolInfo>,
    pub imports: Vec<ImportInfo>,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
}

#[derive(Debug, Clone)]
pub struct SyntaxNode {
    pub node_type: String,
    pub text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub children: Vec<SyntaxNode>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub symbol_type: SymbolType,
    pub scope: String,
    pub definition_pos: usize,
    pub references: Vec<usize>,
    pub data_type: Option<String>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Variable,
    Function,
    Class,
    Method,
    Constant,
    Module,
    Type,
    Macro,
}


#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionStyle {
    Contextual,
    Verbose,
    Concise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningEventType {
    CompletionUsed,
    CompletionRejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningOutcome {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationMetrics {
    pub total_events: u32,
    pub positive_feedback_rate: f32,
    pub learning_efficiency: f32,
    pub adaptation_speed: f32,
    pub last_adaptation: chrono::DateTime<chrono::Utc>,
    pub confidence_trend: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralPatterns {
    pub preferred_completion_style: CompletionStyle,
    pub common_error_patterns: Vec<String>,
    pub favorite_languages: HashMap<String, f32>,
    pub coding_sessions: Vec<CodingSession>,
    pub productivity_metrics: ProductivityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityMetrics {
    pub average_session_length: f64,
    pub completion_acceptance_rate: f32,
    pub error_resolution_time: f64,
    pub code_quality_trend: Vec<f32>,
    pub learning_progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: LearningEventType,
    pub context: String,
    pub action: String,
    pub outcome: LearningOutcome,
    pub confidence: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingSession {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub completions_used: u32,
    pub errors_fixed: u32,
    pub productivity_score: f32,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<FunctionInfo>,
    pub fields: Vec<FieldInfo>,
    pub inheritance: Vec<String>,
    pub interfaces: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub default_value: Option<String>,
}

/// Advanced Pattern Recognizer for intelligent code analysis
#[derive(Debug)]
pub struct PatternRecognizer {
    _code_patterns: HashMap<String, CodePattern>,
    _anti_patterns: HashMap<String, AntiPattern>,
    _user_patterns: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct CodePattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub examples: Vec<String>,
    pub confidence: f32,
    pub language_specific: bool,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    DesignPattern,
    BestPractice,
    Performance,
    Security,
    Maintainability,
}

#[derive(Debug, Clone)]
pub struct AntiPattern {
    pub name: String,
    pub description: String,
    pub severity: IssueSeverity,
    pub fix_suggestion: String,
    pub examples: Vec<String>,
}

/// Coding style preferences
#[derive(Debug, Clone)]
pub struct CodingStyle {
    pub naming_convention: String,
    pub indentation: String,
    pub line_length: usize,
    pub bracket_style: String,
}

/// Performance metrics for benchmarking
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub duration_ms: f64,
    pub memory_usage_kb: usize,
    pub cpu_usage_percent: f32,
}

/// Context Analyzer for project and user understanding
#[derive(Debug)]
pub struct ContextAnalyzer {
    project_context: HashMap<String, ProjectContext>,
    user_profile: UserProfile,
    _coding_style: CodingStyle,
    preferences: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub language: String,
    pub framework: Option<String>,
    pub dependencies: Vec<String>,
    pub coding_standards: Vec<String>,
    pub common_patterns: Vec<String>,
    pub file_structure: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub preferred_style: CodingStyle,
    pub common_patterns: Vec<String>,
    pub favorite_libraries: Vec<String>,
    pub coding_habits: HashMap<String, f32>,
    pub error_patterns: Vec<String>,
}

/// Security Analyzer for vulnerability detection
#[derive(Debug)]
pub struct SecurityAnalyzer {
    vulnerability_patterns: HashMap<String, Vulnerability>,
    security_rules: Vec<SecurityRule>,
    _threat_models: HashMap<String, ThreatModel>,
}

#[derive(Debug, Clone)]
pub struct Vulnerability {
    pub vuln_type: VulnerabilityType,
    pub severity: IssueSeverity,
    pub description: String,
    pub cwe_id: Option<String>,
    pub fix_guidance: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum VulnerabilityType {
    Injection,
    Authentication,
    Authorization,
    Cryptography,
    InputValidation,
    ErrorHandling,
    Configuration,
}

#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub severity: IssueSeverity,
    pub language: String,
    pub pattern: String,
}

#[derive(Debug, Clone)]
pub struct ThreatModel {
    pub component: String,
    pub threats: Vec<String>,
    pub mitigations: Vec<String>,
    pub trust_boundaries: Vec<String>,
}

/// Performance Analyzer for optimization insights
#[derive(Debug)]
pub struct PerformanceAnalyzer {
    _performance_patterns: HashMap<String, PerformanceIssue>,
    _optimization_rules: Vec<OptimizationRule>,
    _benchmark_data: HashMap<String, PerformanceMetrics>,
}

#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub issue_type: PerformanceIssueType,
    pub description: String,
    pub impact: PerformanceImpact,
    pub optimization_suggestion: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PerformanceIssueType {
    AlgorithmComplexity,
    MemoryAllocation,
    IOOperations,
    Synchronization,
    ResourceLeaks,
    Caching,
}

#[derive(Debug, Clone)]
pub enum PerformanceImpact {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct OptimizationRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub pattern: String,
    pub replacement: String,
    pub performance_gain: f32,
}

/// Intelligent Refactoring Engine
#[derive(Debug)]
pub struct RefactoringEngine {
    refactoring_patterns: HashMap<String, RefactoringPattern>,
    code_smells: HashMap<String, CodeSmell>,
    transformation_rules: Vec<TransformationRule>,
}

#[derive(Debug, Clone)]
pub struct RefactoringPattern {
    pub name: String,
    pub description: String,
    pub before_pattern: String,
    pub after_pattern: String,
    pub benefits: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct CodeSmell {
    pub name: String,
    pub description: String,
    pub severity: IssueSeverity,
    pub detection_pattern: String,
    pub refactoring_suggestion: String,
}

#[derive(Debug, Clone)]
pub struct TransformationRule {
    pub from_pattern: String,
    pub to_pattern: String,
    pub conditions: Vec<String>,
    pub confidence: f32,
}

/// Senior Software Engineer Knowledge Base
#[derive(Debug)]
pub struct SeniorEngineerKnowledge {
    architecture_patterns: HashMap<String, ArchitecturePattern>,
    testing_strategies: HashMap<String, TestingStrategy>,
    devops_practices: HashMap<String, DevOpsPractice>,
    database_design: HashMap<String, DatabasePattern>,
    api_design: HashMap<String, ApiDesignPattern>,
    documentation_templates: HashMap<String, DocumentationTemplate>,
}

#[derive(Debug, Clone)]
pub struct ArchitecturePattern {
    pub name: String,
    pub description: String,
    pub use_case: String,
    pub components: Vec<String>,
    pub benefits: Vec<String>,
    pub tradeoffs: Vec<String>,
    pub example_code: String,
}

#[derive(Debug, Clone)]
pub struct TestingStrategy {
    pub strategy_type: TestingType,
    pub description: String,
    pub frameworks: Vec<String>,
    pub coverage_goals: f32,
    pub automation_level: AutomationLevel,
    pub ci_cd_integration: bool,
}

#[derive(Debug, Clone)]
pub enum TestingType {
    Unit,
    Integration,
    System,
    Performance,
    Security,
    E2E,
}

impl std::fmt::Display for TestingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestingType::Unit => write!(f, "Unit Testing"),
            TestingType::Integration => write!(f, "Integration Testing"),
            TestingType::System => write!(f, "System Testing"),
            TestingType::Performance => write!(f, "Performance Testing"),
            TestingType::Security => write!(f, "Security Testing"),
            TestingType::E2E => write!(f, "End-to-End Testing"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AutomationLevel {
    Manual,
    SemiAutomated,
    FullyAutomated,
}

#[derive(Debug, Clone)]
pub struct DevOpsPractice {
    pub practice: String,
    pub description: String,
    pub tools: Vec<String>,
    pub benefits: Vec<String>,
    pub implementation_steps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DatabasePattern {
    pub pattern: String,
    pub description: String,
    pub use_cases: Vec<String>,
    pub sql_example: String,
    pub considerations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ApiDesignPattern {
    pub pattern: String,
    pub description: String,
    pub http_method: String,
    pub example: String,
    pub best_practices: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DocumentationTemplate {
    pub doc_type: String,
    pub template: String,
    pub sections: Vec<String>,
    pub examples: Vec<String>,
}

/// Terminal Intelligence for Command Execution
#[derive(Debug)]
pub struct TerminalIntelligence {
    command_patterns: HashMap<String, CommandPattern>,
    script_templates: HashMap<String, ScriptTemplate>,
    automation_workflows: HashMap<String, AutomationWorkflow>,
    package_managers: HashMap<String, PackageManager>,
    system_commands: HashMap<String, SystemCommand>,
}

#[derive(Debug, Clone)]
pub struct CommandPattern {
    pub command: String,
    pub description: String,
    pub category: CommandCategory,
    pub arguments: Vec<CommandArgument>,
    pub examples: Vec<String>,
    pub common_flags: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum CommandCategory {
    FileSystem,
    ProcessManagement,
    Network,
    System,
    Development,
    PackageManagement,
    TextProcessing,
    BuildTool,
}

#[derive(Debug, Clone)]
pub struct CommandArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScriptTemplate {
    pub name: String,
    pub description: String,
    pub language: String,
    pub template: String,
    pub variables: Vec<String>,
    pub use_cases: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AutomationWorkflow {
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub triggers: Vec<String>,
    pub error_handling: ErrorHandlingStrategy,
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub command: String,
    pub description: String,
    pub timeout_seconds: Option<u32>,
    pub continue_on_error: bool,
}

#[derive(Debug, Clone)]
pub enum ErrorHandlingStrategy {
    StopOnError,
    ContinueOnError,
    RetryOnError,
}

#[derive(Debug, Clone)]
pub struct PackageManager {
    pub name: String,
    pub description: String,
    pub commands: HashMap<String, String>,
    pub ecosystems: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SystemCommand {
    pub name: String,
    pub description: String,
    pub category: SystemCategory,
    pub syntax: String,
    pub examples: Vec<String>,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SystemCategory {
    Process,
    Network,
    FileSystem,
    SystemInfo,
    UserManagement,
    Security,
    Monitoring,
}

impl AiEngine {
    /// Create a new AI engine
    pub async fn new(config: AiConfig) -> Result<Self> {
        // Initialize provider manager with available providers
        let mut provider_manager = AiProviderManager::new();

        // Register default providers based on configuration
        match config.provider.as_str() {
            "openai" => {
                if let Some(api_key) = &config.api_key {
                    let openai_config = ModelConfig {
                        provider: AiProvider::OpenAI,
                        model_name: config.model_name.clone().unwrap_or_else(|| "gpt-4".to_string()),
                        api_key: Some(api_key.clone()),
                        base_url: None,
                        max_tokens: config.max_tokens,
                        temperature: config.temperature,
                        context_window: 8192,
                    };
                    let openai_provider = OpenAiProvider::new(openai_config);
                    provider_manager.register_provider("openai".to_string(), Box::new(openai_provider));
                    provider_manager.set_active_provider("openai")?;
                }
            }
            "anthropic" => {
                if let Some(api_key) = &config.api_key {
                    let anthropic_config = ModelConfig {
                        provider: AiProvider::Anthropic,
                        model_name: config.model_name.clone().unwrap_or_else(|| "claude-3-sonnet-20240229".to_string()),
                        api_key: Some(api_key.clone()),
                        base_url: None,
                        max_tokens: config.max_tokens,
                        temperature: config.temperature,
                        context_window: 200000,
                    };
                    let anthropic_provider = AnthropicProvider::new(anthropic_config);
                    provider_manager.register_provider("anthropic".to_string(), Box::new(anthropic_provider));
                    provider_manager.set_active_provider("anthropic")?;
                }
            }
            "minimax" => {
                if let Some(api_key) = &config.api_key {
                    let minimax_config = ModelConfig {
                        provider: AiProvider::MiniMax,
                        model_name: config.model_name.clone().unwrap_or_else(|| "abab5-chat".to_string()),
                        api_key: Some(api_key.clone()),
                        base_url: None,
                        max_tokens: config.max_tokens,
                        temperature: config.temperature,
                        context_window: 8192,
                    };
                    let minimax_provider = MiniMaxProvider::new(minimax_config);
                    provider_manager.register_provider("minimax".to_string(), Box::new(minimax_provider));
                    provider_manager.set_active_provider("minimax")?;
                }
            }
            _ => {
                // Default to local provider if no API key configured
                let local_config = ModelConfig {
                    provider: AiProvider::Local,
                    model_name: config.model_name.clone().unwrap_or_else(|| "local-model".to_string()),
                    api_key: None,
                    base_url: None,
                    max_tokens: config.max_tokens,
                    temperature: config.temperature,
                    context_window: 2048,
                };
                let local_provider = LocalProvider::new(local_config);
                provider_manager.register_provider("local".to_string(), Box::new(local_provider));
                provider_manager.set_active_provider("local")?;
            }
        }

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            provider_manager: Arc::new(RwLock::new(provider_manager)),
            learning_data: Arc::new(RwLock::new(HashMap::new())),
            semantic_analyzer: Arc::new(RwLock::new(SemanticAnalyzer::new())),
            _pattern_recognizer: Arc::new(RwLock::new(PatternRecognizer::new())),
            context_analyzer: Arc::new(RwLock::new(ContextAnalyzer::new())),
            security_analyzer: Arc::new(RwLock::new(SecurityAnalyzer::new())),
            performance_analyzer: Arc::new(RwLock::new(PerformanceAnalyzer::new())),
            refactoring_engine: Arc::new(RwLock::new(RefactoringEngine::new())),
            senior_engineer_knowledge: Arc::new(RwLock::new(SeniorEngineerKnowledge::new())),
            terminal_intelligence: Arc::new(RwLock::new(TerminalIntelligence::new())),
        })
    }

    /// Update AI configuration
    pub async fn update_config(&self, config: AiConfig) -> Result<()> {
        *self.config.write().await = config;
        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> Result<AiConfig> {
        Ok(self.config.read().await.clone())
    }

    /// Get AI provider information
    pub async fn ai_provider(&self) -> Result<String> {
        let config = self.config.read().await;
        Ok(config.provider.clone())
    }

    /// Generate completion for given request
    pub async fn generate_completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.complete_code(request).await
    }

    /// Generate advanced code completion with multi-token support
    pub async fn complete_code(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // First try AI provider for completion
        let provider_result = self.provider_manager.read().await
            .complete_with_fallback(request.clone()).await;

        match provider_result {
            Ok(response) => {
                // Enhance with semantic analysis if provider succeeded
                let mut analyzer = self.semantic_analyzer.write().await;
                if let Ok(context_tree) = analyzer.parse_code(&request.context, &request.language, "completion_context.rs").await {
                    // Generate additional context-aware suggestions
                    let context_suggestions = self.generate_multi_token_completions(&request, &context_tree).await;
                    let mut enhanced_suggestions = response.suggestions.clone();
                    enhanced_suggestions.extend(context_suggestions);

                    return Ok(CompletionResponse {
                        text: response.text,
                        confidence: response.confidence,
                        suggestions: enhanced_suggestions,
                    });
                } else {
                    // Return provider response as-is if semantic analysis fails
                    return Ok(response);
                }
            }
            Err(_) => {
                // Fallback to semantic analysis-based completion
                let mut analyzer = self.semantic_analyzer.write().await;
                let context_tree = analyzer.parse_code(&request.context, &request.language, "completion_context.rs").await
                    .unwrap_or_else(|_| SyntaxTree {
                        root: SyntaxNode {
                            node_type: "unknown".to_string(),
                            text: request.context.clone(),
                            start_pos: 0,
                            end_pos: request.context.len(),
                            children: Vec::new(),
                            attributes: HashMap::new(),
                        },
                        symbols: HashMap::new(),
                        imports: Vec::new(),
                        functions: Vec::new(),
                        classes: Vec::new(),
                    });

                // Generate multi-token completions based on context
                let suggestions = self.generate_multi_token_completions(&request, &context_tree).await;

                let completion_text = if !suggestions.is_empty() {
                    suggestions[0].clone()
                } else {
                    self.fallback_completion(&request).await
                };

                Ok(CompletionResponse {
                    text: completion_text,
                    confidence: self.calculate_completion_confidence(&request, &suggestions),
                    suggestions,
                })
            }
        }
    }

    /// Generate multi-token completions for entire code blocks
    async fn generate_multi_token_completions(&self, request: &CompletionRequest, context_tree: &SyntaxTree) -> Vec<String> {
        let mut completions = Vec::new();
        let context = request.context.to_lowercase();

        // Analyze context for completion patterns
        match request.language.as_str() {
            "rust" => {
                completions.extend(self.generate_rust_block_completions(&context, context_tree).await);
            }
            "typescript" | "javascript" => {
                completions.extend(self.generate_js_block_completions(&context, context_tree).await);
            }
            "python" => {
                completions.extend(self.generate_python_block_completions(&context, context_tree).await);
            }
            _ => {
                completions.push("// Complete code block here".to_string());
            }
        }

        // Add context-aware completions
        completions.extend(self.generate_context_aware_completions(request, context_tree).await);

        // Use pattern recognizer to enhance completions
        if let Ok(pattern_recognizer) = self._pattern_recognizer.try_read() {
            let pattern_suggestions = pattern_recognizer.recognize_patterns(&request.context, &request.language).await;
            completions.extend(pattern_suggestions);
        }

        // Filter and rank completions
        self.rank_and_filter_completions(completions, request)
    }

    /// Generate Rust code block completions
    async fn generate_rust_block_completions(&self, context: &str, tree: &SyntaxTree) -> Vec<String> {
        let mut completions = Vec::new();

        // Analyze syntax tree to provide better context-aware completions
        let existing_types: Vec<&String> = tree.symbols.values()
            .filter(|s| matches!(s.symbol_type, crate::ai::SymbolType::Type))
            .map(|s| &s.name)
            .collect();
        
        let existing_functions: Vec<&String> = tree.functions.iter().map(|f| &f.name).collect();

        if context.contains("fn ") && !context.contains("{") {
            // Function definition completion
            let type_suggestion = if !existing_types.is_empty() {
                format!(" -> {}", existing_types[0])
            } else {
                " -> ReturnType".to_string()
            };
            
            completions.push(format!("fn function_name(param: Type){} {{\n    // Implementation\n}}", type_suggestion));
            completions.push("fn function_name(&self, param: Type) -> Result<T, E> {\n    // Implementation\n    Ok(result)\n}".to_string());
        } else if context.contains("impl ") && context.contains("for ") {
            // Trait implementation completion
            let method_suggestions = if !existing_functions.is_empty() {
                format!("\n    // Available functions: {:?}\n    // Implementation", existing_functions)
            } else {
                "\n    // Implementation".to_string()
            };
            
            completions.push(format!("impl Trait for Struct {{{}}}", method_suggestions));
        } else if context.contains("struct ") && !context.contains("{") {
            // Struct definition completion
            completions.push("struct StructName {\n    field: Type,\n    another_field: AnotherType,\n}".to_string());
        } else if context.contains("enum ") && !context.contains("{") {
            // Enum definition completion
            completions.push("enum EnumName {\n    Variant1,\n    Variant2(Type),\n    Variant3 { field: Type },\n}".to_string());
        } else if context.contains("if ") && !context.contains("{") {
            // If statement completion
            completions.push("if condition {\n    // Code here\n} else {\n    // Alternative code\n}".to_string());
        } else if context.contains("match ") && !context.contains("{") {
            // Match expression completion
            completions.push("match value {\n    Pattern1 => result1,\n    Pattern2 => result2,\n    _ => default_result,\n}".to_string());
        } else if context.contains("for ") && !context.contains("{") {
            // For loop completion
            completions.push("for item in collection {\n    // Process item\n}".to_string());
        } else if context.contains("async fn ") {
            // Async function completion
            completions.push("async fn async_function() -> Result<T, E> {\n    // Async implementation\n    Ok(result)\n}".to_string());
        }

        completions
    }

    /// Generate JavaScript/TypeScript code block completions
    async fn generate_js_block_completions(&self, context: &str, tree: &SyntaxTree) -> Vec<String> {
        let mut completions = Vec::new();

        // Analyze syntax tree to understand context better
        let has_function_def = tree.functions.len() > 0;
        let _has_class_def = tree.classes.len() > 0;
        let import_count = tree.imports.len();

        if context.contains("function ") && !context.contains("{") {
            let mut completion = "function functionName(param) {\n    // Implementation\n    return result;\n}".to_string();
            
            // Add context-aware suggestions based on imports
            if import_count > 0 {
                completion = completion.replace("return result;", 
                    &format!("// Use imported modules: {:?}\n    return result;", tree.imports.iter().map(|i| &i.module_path).collect::<Vec<_>>()));
            }
            completions.push(completion);
        } else if context.contains("const ") && context.contains("= ") && !context.contains("{") {
            completions.push("const functionName = (param) => {\n    // Implementation\n    return result;\n};".to_string());
        } else if context.contains("class ") && !context.contains("{") {
            let mut completion = "class ClassName {\n    constructor(param) {\n        this.param = param;\n    }\n\n    method() {\n        // Implementation\n    }\n}".to_string();
            
            // Suggest methods based on existing functions in the file
            if has_function_def {
                let function_names: Vec<String> = tree.functions.iter().map(|f| format!("self.{}(), ", f.name)).collect();
                completion = completion.replace("// Implementation", 
                    &format!("// Available functions: {}\n        // Implementation", function_names.join("")));
            }
            completions.push(completion);
        } else if context.contains("if ") && !context.contains("{") {
            completions.push("if (condition) {\n    // Code here\n} else {\n    // Alternative code\n}".to_string());
        } else if context.contains("for ") && !context.contains("{") {
            completions.push("for (let i = 0; i < array.length; i++) {\n    // Process array[i]\n}".to_string());
        } else if context.contains("try ") && !context.contains("{") {
            completions.push("try {\n    // Code that might throw\n} catch (error) {\n    // Handle error\n} finally {\n    // Cleanup code\n}".to_string());
        } else if context.contains("async function") {
            completions.push("async function asyncFunction() {\n    try {\n        const result = await someAsyncOperation();\n        return result;\n    } catch (error) {\n        console.error(error);\n    }\n}".to_string());
        }

        completions
    }

    /// Generate Python code block completions
    async fn generate_python_block_completions(&self, context: &str, tree: &SyntaxTree) -> Vec<String> {
        let mut completions = Vec::new();

        // Analyze syntax tree for better context-aware completions
        let _has_class_def = tree.classes.len() > 0;
        let has_function_def = tree.functions.len() > 0;
        let imports: Vec<&String> = tree.imports.iter().map(|i| &i.module_path).collect();

        if context.contains("def ") && !context.contains(":") {
            let import_suggestion = if !imports.is_empty() {
                format!("\n    # Available imports: {:?}\n    # Implementation\n    return result", imports)
            } else {
                "\n    # Implementation\n    return result".to_string()
            };
            
            completions.push(format!("def function_name(param: Type) -> ReturnType:\n    \"\"\"\n    Function docstring\n    \"\"\"{}", import_suggestion));
        } else if context.contains("class ") && !context.contains(":") {
            let mut completion = "class ClassName:\n    \"\"\"\n    Class docstring\n    \"\"\"\n    \n    def __init__(self, param: Type):\n        self.param = param\n    \n    def method(self) -> ReturnType:\n        # Implementation\n        return result".to_string();
        
        // Add context based on existing functions
        if has_function_def {
            let function_names: Vec<String> = tree.functions.iter()
                .map(|f| format!("self.{}(), ", f.name))
                .collect();
            completion = completion.replace("# Implementation", 
                &format!("# Available functions: {}\n        # Implementation", function_names.join("")));
        }
        
        completions.push(completion);
        } else if context.contains("if ") && !context.contains(":") {
            completions.push("if condition:\n    # Code here\nelse:\n    # Alternative code".to_string());
        } else if context.contains("for ") && !context.contains(":") {
            completions.push("for item in iterable:\n    # Process item".to_string());
        } else if context.contains("try:") && !context.contains("except") {
            completions.push("try:\n    # Code that might raise exception\nexcept Exception as e:\n    # Handle exception\n    print(f\"Error: {e}\")\nfinally:\n    # Cleanup code".to_string());
        } else if context.contains("async def ") {
            completions.push("async def async_function() -> ReturnType:\n    \"\"\"\n    Async function docstring\n    \"\"\"\n    # Async implementation\n    return await some_coroutine()".to_string());
        }

        completions
    }

    /// Generate context-aware completions based on project structure
    async fn generate_context_aware_completions(&self, request: &CompletionRequest, tree: &SyntaxTree) -> Vec<String> {
        let mut completions = Vec::new();

        // Analyze existing imports and suggest related completions
        for import in &tree.imports {
            match request.language.as_str() {
                "rust" => {
                    if import.module_path.contains("std::fs") {
                        completions.push("let contents = fs::read_to_string(\"file.txt\")?;\n    println!(\"{}\", contents);".to_string());
                    } else if import.module_path.contains("std::collections") {
                        completions.push("let mut map = HashMap::new();\n    map.insert(key, value);".to_string());
                    }
                }
                "python" => {
                    if import.module_path.contains("os") {
                        completions.push("files = os.listdir('.')\n    for file in files:\n        print(file)".to_string());
                    } else if import.module_path.contains("json") {
                        completions.push("with open('data.json', 'r') as f:\n    data = json.load(f)\n    print(data)".to_string());
                    }
                }
                _ => {}
            }
        }

        // Analyze existing functions and suggest related patterns
        for func in &tree.functions {
            if func.name.contains("test") {
                completions.push("#[test]\nfn test_functionality() {\n    // Test implementation\n    assert!(true);\n}".to_string());
            } else if func.name.contains("handler") {
                completions.push("async fn request_handler(req: Request) -> Result<Response, Error> {\n    // Handle request\n    Ok(response)\n}".to_string());
            }
        }

        completions
    }

    /// Calculate confidence score for completions
    fn calculate_completion_confidence(&self, request: &CompletionRequest, suggestions: &[String]) -> f32 {
        if suggestions.is_empty() {
            return 0.0;
        }

        // Base confidence on context analysis
        let mut confidence: f32 = 0.5;

        // Increase confidence based on code structure
        if request.context.contains("fn ") || request.context.contains("function ") || request.context.contains("def ") {
            confidence += 0.2;
        }

        if request.context.contains("{") || request.context.contains(":") {
            confidence += 0.1;
        }

        // Context-aware confidence
        if let Ok(context_analyzer) = self.context_analyzer.try_read() {
            if context_analyzer.project_context.contains_key(&request.language) {
                confidence += 0.1;
            }
        }

        confidence.min(1.0)
    }

    /// Fallback completion when no specific patterns match
    async fn fallback_completion(&self, request: &CompletionRequest) -> String {
        // Use mock completion methods for intelligent fallbacks
        match request.language.as_str() {
            "rust" => {
                let mock_completions = self.mock_rust_completion(&request.context);
                if !mock_completions.is_empty() {
                    mock_completions[0].clone()
                } else {
                    "// Complete your Rust code here".to_string()
                }
            }
            "javascript" | "typescript" => {
                let mock_completions = self.mock_js_completion(&request.context);
                if !mock_completions.is_empty() {
                    mock_completions[0].clone()
                } else {
                    "// Complete your JavaScript code here".to_string()
                }
            }
            "python" => {
                let mock_completions = self.mock_python_completion(&request.context);
                if !mock_completions.is_empty() {
                    mock_completions[0].clone()
                } else {
                    "# Complete your Python code here".to_string()
                }
            }
            _ => "// Complete your code here".to_string(),
        }
    }

    /// Rank and filter completions based on relevance
    fn rank_and_filter_completions(&self, completions: Vec<String>, request: &CompletionRequest) -> Vec<String> {
        // Remove duplicates and sort by relevance
        let mut unique_completions: Vec<String> = completions.into_iter().collect();
        unique_completions.sort_by(|a, b| {
            // Prefer completions that match the current indentation/context
            let a_score = self.score_completion(a, request);
            let b_score = self.score_completion(b, request);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        unique_completions.dedup();
        unique_completions.into_iter().take(10).collect() // Limit to top 10
    }

    /// Score completion based on context matching
    fn score_completion(&self, completion: &str, request: &CompletionRequest) -> f32 {
        let mut score = 0.0;

        // Prefer completions that continue the current statement
        if completion.starts_with(&request.context[request.context.len().saturating_sub(10)..]) {
            score += 0.5;
        }

        // Prefer language-specific patterns
        match request.language.as_str() {
            "rust" => {
                if completion.contains("Result<") || completion.contains("Option<") {
                    score += 0.2;
                }
            }
            "python" => {
                if completion.contains("def ") || completion.contains("class ") {
                    score += 0.2;
                }
            }
            _ => {}
        }

        // Prefer completions that introduce useful patterns
        if completion.contains("error") || completion.contains("Error") {
            score += 0.1;
        }

        score
    }

    /// Analyze code for issues and suggestions using advanced AI
    pub async fn analyze_code(&self, code: &str, language: &str) -> Result<AnalysisResult> {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut complexity_score: f32 = 0.0;

        // First try AI provider for code analysis
        let provider_result = self.provider_manager.read().await
            .analyze_with_fallback(code, language).await;

        match provider_result {
            Ok(provider_analysis) => {
                // Enhance with local analysis capabilities
                issues.extend(provider_analysis.issues);
                suggestions.extend(provider_analysis.suggestions);
                complexity_score = complexity_score.max(provider_analysis.complexity_score);
            }
            Err(_) => {
                // Fallback to local analysis if provider fails
                let semantic_result = self.analyze_semantically(code, language).await;
                issues.extend(semantic_result.issues);
                suggestions.extend(semantic_result.suggestions);
            }
        }

        // Always run local pattern recognition and security analysis
        // Use pattern recognizer for code quality analysis
        let pattern_issues = self.recognize_patterns(code, language).await;

        // Use security analyzer for vulnerability detection
        let security_issues = self.analyze_security(code, language).await;

        // Use performance analyzer for optimization suggestions
        let performance_issues = self.analyze_performance(code, language).await;

        // Combine all analysis results
        issues.extend(pattern_issues);
        issues.extend(security_issues);
        issues.extend(performance_issues);

        suggestions.extend(self.generate_refactoring_suggestions(code, language).await);

        // Calculate overall complexity score (combine with local calculation)
        let calculated_score = self.calculate_complexity_score(code, language);
        complexity_score = complexity_score.max(calculated_score);

        // Add basic TODO detection for demo purposes
        if code.contains("TODO") {
            issues.push(CodeIssue {
                id: Uuid::new_v4().to_string(),
                severity: IssueSeverity::Info,
                message: "TODO comment found".to_string(),
                line: 1,
                column: 1,
            });
        }

        Ok(AnalysisResult {
            issues,
            suggestions,
            complexity_score,
        })
    }

    /// Perform semantic analysis of code
    async fn analyze_semantically(&self, code: &str, language: &str) -> AnalysisResult {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // Enhanced semantic analysis based on language
        match language {
            "rust" => {
                if code.contains("unwrap()") {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Warning,
                        message: "Consider using ? operator instead of unwrap() for better error handling".to_string(),
                        line: 1,
                        column: 1,
                    });

                    suggestions.push(CodeSuggestion {
                        id: Uuid::new_v4().to_string(),
                        message: "Replace unwrap() with ? operator for better error handling".to_string(),
                        code: "fn example() -> Result<T, E> {\n    // Use ? instead of unwrap()\n}".to_string(),
                        confidence: 0.9,
                    });
                }

                if code.contains("pub fn ") && !code.contains("///") {
                    suggestions.push(CodeSuggestion {
                        id: Uuid::new_v4().to_string(),
                        message: "Consider adding documentation to public functions".to_string(),
                        code: "/// Function description\npub fn function_name() {\n    // Implementation\n}".to_string(),
                        confidence: 0.7,
                    });
                }
            }
            "python" => {
                if code.contains("print(") {
                    suggestions.push(CodeSuggestion {
                        id: Uuid::new_v4().to_string(),
                        message: "Consider using logging instead of print statements".to_string(),
                        code: "import logging\nlogging.info('message')".to_string(),
                        confidence: 0.6,
                    });
                }
            }
            "javascript" | "typescript" => {
                if code.contains("var ") {
                    suggestions.push(CodeSuggestion {
                        id: Uuid::new_v4().to_string(),
                        message: "Consider using let/const instead of var".to_string(),
                        code: "const variableName = value;".to_string(),
                        confidence: 0.8,
                    });
                }
            }
            _ => {}
        }

        AnalysisResult {
            issues,
            suggestions,
            complexity_score: 0.3,
        }
    }

    /// Recognize patterns and anti-patterns in code
    async fn recognize_patterns(&self, code: &str, language: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Pattern recognition for common issues
        match language {
            "rust" => {
                if code.contains("Rc<RefCell<") {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Warning,
                        message: "Consider using Arc<Mutex<>> for thread-safe interior mutability".to_string(),
                        line: 1,
                        column: 1,
                    });
                }

                if code.lines().count() > 50 && code.contains("fn main()") {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Info,
                        message: "Consider breaking large main function into smaller functions".to_string(),
                        line: 1,
                        column: 1,
                    });
                }
            }
            _ => {}
        }

        issues
    }

    /// Analyze code for security vulnerabilities
    async fn analyze_security(&self, code: &str, language: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        match language {
            "rust" => {
                if code.contains("std::env::var(") && !code.contains("expect(") {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Warning,
                        message: "Environment variable access should handle missing variables gracefully".to_string(),
                        line: 1,
                        column: 1,
                    });
                }
            }
            "javascript" | "typescript" => {
                if code.contains("eval(") {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Critical,
                        message: "Use of eval() can lead to code injection vulnerabilities".to_string(),
                        line: 1,
                        column: 1,
                    });
                }

                if code.contains("innerHTML") && code.contains("=") {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Warning,
                        message: "Direct innerHTML assignment can lead to XSS vulnerabilities".to_string(),
                        line: 1,
                        column: 1,
                    });
                }
            }
            _ => {}
        }

        issues
    }

    /// Analyze code for performance issues
    async fn analyze_performance(&self, code: &str, language: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        match language {
            "rust" => {
                if code.contains("String::from(") && code.contains("push_str(") {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Info,
                        message: "Consider using format!() for string concatenation".to_string(),
                        line: 1,
                        column: 1,
                    });
                }

                if code.contains("vec![") && code.lines().any(|line| line.contains("push(")) {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Info,
                        message: "Consider pre-allocating Vec capacity if size is known".to_string(),
                        line: 1,
                        column: 1,
                    });
                }
            }
            "python" => {
                if code.contains("for ") && code.contains("range(len(") {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Info,
                        message: "Use enumerate() instead of range(len()) for better performance".to_string(),
                        line: 1,
                        column: 1,
                    });
                }
            }
            _ => {}
        }

        issues
    }

    /// Generate intelligent refactoring suggestions
    async fn generate_refactoring_suggestions(&self, code: &str, language: &str) -> Vec<CodeSuggestion> {
        let mut suggestions = Vec::new();

        match language {
            "rust" => {
                if code.lines().count() > 30 && code.contains("fn ") && !code.contains("struct ") {
                    suggestions.push(CodeSuggestion {
                        id: Uuid::new_v4().to_string(),
                        message: "Consider extracting this function into a separate module".to_string(),
                        code: "mod utils {\n    pub fn extracted_function() {\n        // Implementation\n    }\n}".to_string(),
                        confidence: 0.6,
                    });
                }

                if code.contains("if let Some") && code.contains("else") {
                    suggestions.push(CodeSuggestion {
                        id: Uuid::new_v4().to_string(),
                        message: "Consider using match instead of if let with else".to_string(),
                        code: "match value {\n    Some(v) => { /* handle some */ }\n    None => { /* handle none */ }\n}".to_string(),
                        confidence: 0.7,
                    });
                }
            }
            _ => {}
        }

        suggestions
    }

    /// Calculate code complexity score
    fn calculate_complexity_score(&self, code: &str, _language: &str) -> f32 {
        let lines = code.lines().count();
        let functions = code.matches("fn ").count();
        let conditionals = code.matches("if ").count() + code.matches("match ").count();
        let loops = code.matches("for ").count() + code.matches("while ").count();

        // Simple complexity calculation
        let base_complexity = (functions as f32 * 2.0) + (conditionals as f32 * 1.5) + (loops as f32 * 1.0);
        let size_factor = (lines as f32 / 100.0).min(1.0);

        (base_complexity * size_factor).min(1.0)
    }

    /// Predict potential bugs
    pub async fn predict_bugs(&self, code: &str) -> Result<Vec<CodeIssue>> {
        let mut bugs = Vec::new();

        // Mock bug prediction patterns
        if code.contains("== null") || code.contains("== undefined") {
            bugs.push(CodeIssue {
                id: Uuid::new_v4().to_string(),
                severity: IssueSeverity::Warning,
                message: "Potential null/undefined comparison".to_string(),
                line: 1,
                column: 1,
            });
        }

        Ok(bugs)
    }

    /// Learn from user feedback with advanced adaptation
    pub async fn learn_from_feedback(&self, pattern: String, success: bool) -> Result<()> {
        let mut learning_data = self.learning_data.write().await;

        // Create enhanced learning data with behavioral tracking
        let entry = learning_data.entry(pattern.clone()).or_insert_with(|| LearningData {
            patterns: HashMap::new(),
            feedback_count: 0,
            success_rate: 0.0,
            user_preferences: HashMap::new(),
            project_contexts: HashMap::new(),
            learning_history: Vec::new(),
            adaptation_metrics: AdaptationMetrics {
                total_events: 0,
                positive_feedback_rate: 0.0,
                learning_efficiency: 1.0,
                adaptation_speed: 0.1,
                last_adaptation: chrono::Utc::now(),
                confidence_trend: Vec::new(),
            },
            behavioral_patterns: BehavioralPatterns {
                preferred_completion_style: CompletionStyle::Contextual,
                common_error_patterns: Vec::new(),
                favorite_languages: HashMap::new(),
                coding_sessions: Vec::new(),
                productivity_metrics: ProductivityMetrics {
                    average_session_length: 0.0,
                    completion_acceptance_rate: 0.0,
                    error_resolution_time: 0.0,
                    code_quality_trend: Vec::new(),
                    learning_progress: 0.0,
                },
            },
        });

        // Record learning event
        let learning_event = LearningEvent {
            timestamp: chrono::Utc::now(),
            event_type: if success { LearningEventType::CompletionUsed } else { LearningEventType::CompletionRejected },
            context: "code_completion".to_string(),
            action: pattern.clone(),
            outcome: if success { LearningOutcome::Positive } else { LearningOutcome::Negative },
            confidence: entry.success_rate,
            metadata: HashMap::from([
                ("pattern_type".to_string(), "completion".to_string()),
                ("language".to_string(), "unknown".to_string()),
            ]),
        };

        entry.learning_history.push(learning_event);
        entry.feedback_count += 1;

        // Update success rate with exponential moving average for stability
        let alpha = 0.1; // Learning rate
        let target = if success { 1.0 } else { 0.0 };
        entry.success_rate = alpha * target + (1.0 - alpha) * entry.success_rate;

        // Update adaptation metrics
        entry.adaptation_metrics.total_events += 1;
        entry.adaptation_metrics.confidence_trend.push(entry.success_rate);

        // Keep only recent trend data (last 100 events)
        if entry.adaptation_metrics.confidence_trend.len() > 100 {
            entry.adaptation_metrics.confidence_trend.remove(0);
        }

        // Calculate positive feedback rate
        let positive_events = entry.learning_history.iter()
            .filter(|e| matches!(e.outcome, LearningOutcome::Positive))
            .count() as f32;
        entry.adaptation_metrics.positive_feedback_rate = positive_events / entry.learning_history.len() as f32;

        // Update learning efficiency based on improvement rate
        if entry.adaptation_metrics.confidence_trend.len() >= 10 {
            let recent_avg = entry.adaptation_metrics.confidence_trend.iter().rev().take(10).sum::<f32>() / 10.0;
            let older_avg = entry.adaptation_metrics.confidence_trend.iter().rev().skip(10).take(10).sum::<f32>() / 10.0;
            entry.adaptation_metrics.learning_efficiency = (recent_avg - older_avg).max(0.0) + 0.5;
        }

        // Adapt behavioral patterns based on learning
        self.adapt_behavioral_patterns(entry, &pattern, success).await;

        Ok(())
    }

    /// Adapt behavioral patterns based on learning data
    async fn adapt_behavioral_patterns(&self, learning_data: &mut LearningData, pattern: &str, success: bool) {
        // Update completion style preference based on success patterns
        if success && pattern.contains("fn ") {
            learning_data.behavioral_patterns.preferred_completion_style = CompletionStyle::Verbose;
        } else if success && pattern.len() < 50 {
            learning_data.behavioral_patterns.preferred_completion_style = CompletionStyle::Concise;
        }

        // Update favorite languages based on successful patterns
        if success {
            if pattern.contains("fn ") || pattern.contains("impl ") {
                *learning_data.behavioral_patterns.favorite_languages.entry("rust".to_string()).or_insert(0.0) += 0.1;
            } else if pattern.contains("function ") || pattern.contains("=>") {
                *learning_data.behavioral_patterns.favorite_languages.entry("javascript".to_string()).or_insert(0.0) += 0.1;
            } else if pattern.contains("def ") || pattern.contains("class ") {
                *learning_data.behavioral_patterns.favorite_languages.entry("python".to_string()).or_insert(0.0) += 0.1;
            }
        }

        // Update productivity metrics
        let session_count = learning_data.behavioral_patterns.coding_sessions.len() as f32;
        if session_count > 0.0 {
            learning_data.behavioral_patterns.productivity_metrics.average_session_length =
                learning_data.behavioral_patterns.coding_sessions.iter()
                    .map(|s| (s.end_time - s.start_time).num_seconds() as f64)
                    .sum::<f64>() / session_count as f64;

            learning_data.behavioral_patterns.productivity_metrics.completion_acceptance_rate =
                learning_data.adaptation_metrics.positive_feedback_rate;
        }

        // Update learning progress based on adaptation metrics
        learning_data.behavioral_patterns.productivity_metrics.learning_progress =
            learning_data.adaptation_metrics.learning_efficiency * learning_data.success_rate;
    }

    /// Record coding session for behavioral analysis
    pub async fn record_coding_session(&self, session: CodingSession) -> Result<()> {
        let mut learning_data = self.learning_data.write().await;

        // Add session to global learning data (using a default key)
        let entry = learning_data.entry("global_session_data".to_string()).or_insert_with(|| LearningData {
            patterns: HashMap::new(),
            feedback_count: 0,
            success_rate: 0.0,
            user_preferences: HashMap::new(),
            project_contexts: HashMap::new(),
            learning_history: Vec::new(),
            adaptation_metrics: AdaptationMetrics {
                total_events: 0,
                positive_feedback_rate: 0.0,
                learning_efficiency: 1.0,
                adaptation_speed: 0.1,
                last_adaptation: chrono::Utc::now(),
                confidence_trend: Vec::new(),
            },
            behavioral_patterns: BehavioralPatterns {
                preferred_completion_style: CompletionStyle::Contextual,
                common_error_patterns: Vec::new(),
                favorite_languages: HashMap::new(),
                coding_sessions: Vec::new(),
                productivity_metrics: ProductivityMetrics {
                    average_session_length: 0.0,
                    completion_acceptance_rate: 0.0,
                    error_resolution_time: 0.0,
                    code_quality_trend: Vec::new(),
                    learning_progress: 0.0,
                },
            },
        });

        entry.behavioral_patterns.coding_sessions.push(session);

        // Keep only recent sessions (last 100)
        if entry.behavioral_patterns.coding_sessions.len() > 100 {
            entry.behavioral_patterns.coding_sessions.remove(0);
        }

        // Update productivity metrics
        self.update_productivity_metrics(&mut entry.behavioral_patterns);

        Ok(())
    }

    /// Update productivity metrics based on coding sessions
    fn update_productivity_metrics(&self, behavioral_patterns: &mut BehavioralPatterns) {
        let sessions = &behavioral_patterns.coding_sessions;
        if sessions.is_empty() {
            return;
        }

        // Calculate average session length
        behavioral_patterns.productivity_metrics.average_session_length =
            sessions.iter()
                .map(|s| (s.end_time - s.start_time).num_seconds() as f64)
                .sum::<f64>() / sessions.len() as f64;

        // Calculate completion acceptance rate across sessions
        let total_completions: u32 = sessions.iter().map(|s| s.completions_used).sum();
        let avg_completion_rate = if !sessions.is_empty() {
            total_completions as f32 / sessions.len() as f32
        } else {
            0.0
        };
        behavioral_patterns.productivity_metrics.completion_acceptance_rate = avg_completion_rate;

        // Calculate average error resolution time (simplified)
        let total_errors: u32 = sessions.iter().map(|s| s.errors_fixed).sum();
        behavioral_patterns.productivity_metrics.error_resolution_time =
            if total_errors > 0 {
                behavioral_patterns.productivity_metrics.average_session_length / total_errors as f64
            } else {
                0.0
            };

        // Update code quality trend (based on productivity scores)
        let recent_scores: Vec<f32> = sessions.iter().rev().take(10).map(|s| s.productivity_score).collect();
        behavioral_patterns.productivity_metrics.code_quality_trend = recent_scores;
    }

    /// Get personalized suggestions based on learning data
    pub async fn get_personalized_suggestions(&self, context: &str, language: &str) -> Result<Vec<String>> {
        let learning_data = self.learning_data.read().await;
        let mut suggestions = Vec::new();

        // Find the most relevant learning data for this context
        let relevant_patterns: Vec<_> = learning_data.iter()
            .filter(|(_, data)| data.success_rate > 0.7) // Only highly successful patterns
            .filter(|(_, data)| data.feedback_count > 5) // Require minimum feedback count
            .collect();

        for (pattern_key, data) in relevant_patterns {
            // Check if pattern is relevant to current context
            if self.is_pattern_relevant(pattern_key, context, language) {
                // Generate suggestions based on successful patterns
                if let Some(suggestion) = self.generate_suggestion_from_pattern(pattern_key, data, context) {
                    suggestions.push(suggestion);
                }
            }
        }

        // Sort by success rate and recency
        suggestions.sort_by(|a, b| {
            // Prioritize more successful patterns
            let a_score = self.score_suggestion(a);
            let b_score = self.score_suggestion(b);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(suggestions.into_iter().take(5).collect())
    }

    /// Check if a learned pattern is relevant to current context
    fn is_pattern_relevant(&self, pattern: &str, context: &str, language: &str) -> bool {
        // Check language relevance
        if pattern.contains("fn ") && language != "rust" {
            return false;
        }
        if pattern.contains("function ") && !["javascript", "typescript"].contains(&language) {
            return false;
        }
        if pattern.contains("def ") && language != "python" {
            return false;
        }

        // Check contextual relevance
        let pattern_lower = pattern.to_lowercase();
        let context_lower = context.to_lowercase();

        // Simple keyword matching for relevance
        let keywords = ["async", "await", "result", "option", "error", "try", "catch", "class", "struct"];
        keywords.iter().any(|&keyword| {
            pattern_lower.contains(keyword) && context_lower.contains(keyword)
        }) || pattern_lower.split_whitespace().any(|word| context_lower.contains(word))
    }

    /// Generate suggestion from learned pattern
    fn generate_suggestion_from_pattern(&self, pattern: &str, data: &LearningData, context: &str) -> Option<String> {
        // Use successful patterns to generate contextually relevant suggestions
        if data.success_rate > 0.8 {
            // For high-confidence patterns, return them directly if relevant
            if self.is_pattern_relevant(pattern, context, "any") {
                return Some(pattern.to_string());
            }
        }

        // Generate variations based on successful patterns
        if pattern.contains("Result<") && context.contains("fn ") {
            Some("-> Result<T, Box<dyn std::error::Error>>".to_string())
        } else if pattern.contains("async fn") && context.contains("fn ") {
            Some("async fn function_name() -> Result<T, E>".to_string())
        } else {
            None
        }
    }

    /// Score suggestion based on various factors
    fn score_suggestion(&self, suggestion: &str) -> f32 {
        let mut score = 0.5; // Base score

        // Prefer suggestions with common successful patterns
        if suggestion.contains("Result<") || suggestion.contains("Option<") {
            score += 0.2;
        }

        if suggestion.contains("async ") || suggestion.contains("await ") {
            score += 0.1;
        }

        // Prefer concise but complete suggestions
        let length_score = 1.0 / (suggestion.len() as f32 / 50.0 + 1.0);
        score += length_score * 0.1;

        score.min(1.0)
    }

    /// Adapt AI behavior based on accumulated learning data
    pub async fn adapt_ai_behavior(&self) -> Result<()> {
        let learning_data = self.learning_data.read().await;

        // Analyze overall learning trends
        let total_patterns = learning_data.len();
        let successful_patterns = learning_data.values()
            .filter(|data| data.success_rate > 0.7)
            .count();

        let success_rate = if total_patterns > 0 {
            successful_patterns as f32 / total_patterns as f32
        } else {
            0.0
        };

        // Update AI configuration based on learning insights
        let mut config = self.config.write().await;

        // Adjust temperature based on success rate
        if success_rate > 0.8 {
            // High success rate - be more conservative
            config.temperature = (config.temperature * 0.9).max(0.1);
        } else if success_rate < 0.5 {
            // Low success rate - be more creative
            config.temperature = (config.temperature * 1.1).min(1.0);
        }

        // Adjust max tokens based on user preferences for completion length
        let avg_completion_length = learning_data.values()
            .filter_map(|data| {
                if data.feedback_count > 0 {
                    Some(data.patterns.values().sum::<f32>() / data.feedback_count as f32)
                } else {
                    None
                }
            })
            .sum::<f32>() / learning_data.len().max(1) as f32;

        if avg_completion_length > 100.0 {
            config.max_tokens = (config.max_tokens as f32 * 1.2) as u32;
        } else if avg_completion_length < 50.0 {
            config.max_tokens = (config.max_tokens as f32 * 0.8) as u32;
        }

        Ok(())
    }

    /// Export learning data for analysis or backup
    pub async fn export_learning_data(&self) -> Result<String> {
        let learning_data = self.learning_data.read().await;
        let export_data = serde_json::to_string_pretty(&*learning_data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize learning data: {}", e))?;
        Ok(export_data)
    }

    /// Import learning data from backup
    pub async fn import_learning_data(&self, data: &str) -> Result<()> {
        let imported_data: HashMap<String, LearningData> = serde_json::from_str(data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize learning data: {}", e))?;

        let mut learning_data = self.learning_data.write().await;
        *learning_data = imported_data;

        Ok(())
    }

    /// Reset learning data (useful for troubleshooting)
    pub async fn reset_learning_data(&self) -> Result<()> {
        let mut learning_data = self.learning_data.write().await;
        learning_data.clear();
        Ok(())
    }

    /// Get learning statistics
    pub async fn get_learning_statistics(&self) -> Result<HashMap<String, serde_json::Value>> {
        let learning_data = self.learning_data.read().await;
        let mut stats = HashMap::new();

        stats.insert("total_patterns".to_string(), learning_data.len().into());
        stats.insert("successful_patterns".to_string(),
            learning_data.values().filter(|d| d.success_rate > 0.7).count().into());

        let avg_success_rate = if !learning_data.is_empty() {
            learning_data.values().map(|d| d.success_rate).sum::<f32>() / learning_data.len() as f32
        } else {
            0.0
        };
        stats.insert("average_success_rate".to_string(), avg_success_rate.into());

        let total_feedback = learning_data.values().map(|d| d.feedback_count).sum::<u32>();
        stats.insert("total_feedback_events".to_string(), total_feedback.into());

        Ok(stats)
    }

    /// Generate mock Rust completions
    /// Generate mock Rust completions based on context
    pub fn mock_rust_completion(&self, context: &str) -> Vec<String> {
        if context.contains("fn ") {
            vec![
                "fn function_name() -> Result<T, E> {\n    // Implementation\n}".to_string(),
                "fn function_name(param: &str) -> i32 {\n    // Implementation\n}".to_string(),
            ]
        } else if context.contains("struct ") {
            vec![
                "struct MyStruct {\n    field: Type,\n}".to_string(),
            ]
        } else if context.contains("impl ") {
            vec![
                "impl MyStruct {\n    fn method(&self) -> T {\n        // Implementation\n    }\n}".to_string(),
            ]
        } else {
            vec!["// Continue your Rust code here".to_string()]
        }
    }

    /// Generate mock JavaScript/TypeScript completions based on context
    pub fn mock_js_completion(&self, context: &str) -> Vec<String> {
        if context.contains("function ") {
            vec![
                "function functionName(param) {\n    // Implementation\n}".to_string(),
            ]
        } else if context.contains("const ") {
            vec![
                "const variableName = value;".to_string(),
                "const functionName = (param) => {\n    // Implementation\n};".to_string(),
            ]
        } else {
            vec!["// Continue your JavaScript code here".to_string()]
        }
    }

    /// Generate mock Python completions based on context
    pub fn mock_python_completion(&self, context: &str) -> Vec<String> {
        if context.contains("def ") {
            vec![
                "def function_name(param):\n    # Implementation\n    pass".to_string(),
            ]
        } else if context.contains("class ") {
            vec![
                "class MyClass:\n    def __init__(self):\n        # Initialization\n        pass".to_string(),
            ]
        } else {
            vec!["# Continue your Python code here".to_string()]
        }
    }

    /// Get senior engineer architecture advice
    pub async fn get_architecture_advice(&self, context: &str) -> Result<String> {
        let knowledge = self.senior_engineer_knowledge.read().await;

        // Analyze context and provide architectural recommendations
        if context.contains("microservice") || context.contains("distributed") {
            let pattern = knowledge.architecture_patterns.get("Microservices")
                .ok_or_else(|| anyhow::anyhow!("Architecture pattern not found"))?;
            Ok(format!("Consider {}: {}\nBenefits: {}\nTradeoffs: {}",
                pattern.name, pattern.description,
                pattern.benefits.join(", "),
                pattern.tradeoffs.join(", ")))
        } else if context.contains("web") || context.contains("gui") {
            let pattern = knowledge.architecture_patterns.get("MVC")
                .ok_or_else(|| anyhow::anyhow!("Architecture pattern not found"))?;
            Ok(format!("For web applications, consider {}: {}\nExample: {}",
                pattern.name, pattern.description, pattern.example_code))
        } else {
            Ok("Consider using layered architecture with clear separation of concerns.".to_string())
        }
    }

    /// Get testing strategy recommendations
    pub async fn get_testing_strategy(&self, project_type: &str) -> Result<String> {
        let knowledge = self.senior_engineer_knowledge.read().await;

        match project_type {
            "api" | "backend" => {
                let strategy = knowledge.testing_strategies.get("Integration Testing")
                    .ok_or_else(|| anyhow::anyhow!("Testing strategy not found"))?;
                Ok(format!("For {} projects, use {}: {}\nFrameworks: {}\nCoverage goal: {:.0}%",
                    project_type, strategy.strategy_type, strategy.description,
                    strategy.frameworks.join(", "), strategy.coverage_goals * 100.0))
            }
            _ => {
                let strategy = knowledge.testing_strategies.get("Unit Testing")
                    .ok_or_else(|| anyhow::anyhow!("Testing strategy not found"))?;
                Ok(format!("Recommended testing strategy: {}\nFrameworks: {}\nAutomation: {}",
                    strategy.strategy_type, strategy.frameworks.join(", "),
                    match strategy.automation_level {
                        AutomationLevel::FullyAutomated => "Fully automated",
                        AutomationLevel::SemiAutomated => "Semi-automated",
                        AutomationLevel::Manual => "Manual",
                    }))
            }
        }
    }

    /// Generate DevOps automation scripts
    pub async fn generate_devops_script(&self, task: &str) -> Result<String> {
        let intelligence = self.terminal_intelligence.read().await;

        match task {
            "ci_cd" => {
                let workflow = intelligence.automation_workflows.get("ci_pipeline")
                    .ok_or_else(|| anyhow::anyhow!("Workflow not found"))?;
                let steps = workflow.steps.iter()
                    .map(|step| format!("  - {}: {}", step.description, step.command))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(format!("CI/CD Pipeline Steps:\n{}", steps))
            }
            "deployment" => {
                let template = intelligence.script_templates.get("deploy_script")
                    .ok_or_else(|| anyhow::anyhow!("Template not found"))?;
                Ok(format!("Deployment Script Template:\n{}", template.template))
            }
            "backup" => {
                let template = intelligence.script_templates.get("backup_script")
                    .ok_or_else(|| anyhow::anyhow!("Template not found"))?;
                Ok(format!("Backup Script Template:\n{}", template.template))
            }
            _ => Ok("#!/bin/bash\n# Custom automation script\n\necho 'Automation task completed'\n".to_string())
        }
    }

    /// Get package management commands
    pub async fn get_package_commands(&self, ecosystem: &str) -> Result<String> {
        let intelligence = self.terminal_intelligence.read().await;

        let manager_name = match ecosystem {
            "rust" => "cargo",
            "javascript" | "typescript" => "npm",
            "python" => "pip",
            _ => return Ok("Unsupported ecosystem".to_string()),
        };

        let manager = intelligence.package_managers.get(manager_name)
            .ok_or_else(|| anyhow::anyhow!("Package manager not found"))?;

        let commands = manager.commands.iter()
            .map(|(cmd, usage)| format!("  {}: {}", cmd, usage))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!("{} package manager commands:\n{}", manager.name, commands))
    }

    /// Generate documentation template
    pub async fn generate_documentation(&self, doc_type: &str, context: &str) -> Result<String> {
        let knowledge = self.senior_engineer_knowledge.read().await;

        let template = knowledge.documentation_templates.get(doc_type)
            .ok_or_else(|| anyhow::anyhow!("Documentation template not found"))?;

        let examples = template.examples.join("\n");
        let doc = template.template
            .replace("{description}", context)
            .replace("{method}", "GET")
            .replace("{path}", "/api/endpoint")
            .replace("{parameters}", "- id: Resource identifier")
            .replace("{response}", "{ \"data\": \"result\" }")
            .replace("{examples}", &examples);

        Ok(doc)
    }

    /// Analyze security vulnerabilities
    pub async fn analyze_security_threats(&self, code: &str, language: &str) -> Result<Vec<String>> {
        let _analyzer = self.security_analyzer.read().await;
        let mut threats = Vec::new();

        // Basic security analysis
        if language == "javascript" && code.contains("eval(") {
            threats.push("Code injection vulnerability: eval() usage".to_string());
        }

        if language == "rust" && code.contains("std::env::var(") && !code.contains("expect(") {
            threats.push("Information disclosure: Unsafe environment variable access".to_string());
        }

        if code.contains("password") && (code.contains("console.log") || code.contains("println!")) {
            threats.push("Information disclosure: Logging sensitive data".to_string());
        }

        if threats.is_empty() {
            threats.push("No obvious security vulnerabilities detected".to_string());
        }

        Ok(threats)
    }

    /// Provide performance optimization advice
    pub async fn get_performance_advice(&self, code: &str, language: &str) -> Result<String> {
        let _analyzer = self.performance_analyzer.read().await;
        let mut advice = Vec::new();

        if language == "rust" {
            if code.contains("String::from(") && code.contains("push_str(") {
                advice.push("Use format!() macro instead of String concatenation for better performance".to_string());
            }
            if code.contains("vec![") && code.lines().any(|line| line.contains("push(")) {
                advice.push("Pre-allocate Vec capacity if size is known: Vec::with_capacity(size)".to_string());
            }
        }

        if language == "python" {
            if code.contains("for ") && code.contains("range(len(") {
                advice.push("Use enumerate() instead of range(len()) for better performance and readability".to_string());
            }
        }

        if advice.is_empty() {
            advice.push("Code appears to follow good performance practices".to_string());
        }

        Ok(advice.join("\n"))
    }

    /// Generate refactoring suggestions
    pub async fn suggest_refactoring(&self, code: &str, language: &str) -> Result<Vec<String>> {
        let _refactoring = self.refactoring_engine.read().await;
        let mut suggestions = Vec::new();

        if language == "rust" {
            if code.lines().count() > 50 && code.contains("fn main()") {
                suggestions.push("Extract main function logic into smaller, focused functions".to_string());
            }
            if code.contains("if let Some") && code.contains("else") {
                suggestions.push("Consider using match instead of if let with else for clarity".to_string());
            }
        }

        if language == "javascript" {
            if code.contains("var ") {
                suggestions.push("Replace var with let/const for better scoping".to_string());
            }
        }

        if suggestions.is_empty() {
            suggestions.push("Code structure looks good, no major refactoring needed".to_string());
        }

        Ok(suggestions)
    }

    /// Get terminal command suggestions
    pub async fn suggest_terminal_commands(&self, context: &str) -> Result<Vec<String>> {
        let intelligence = self.terminal_intelligence.read().await;
        let mut suggestions = Vec::new();

        if context.contains("file") || context.contains("find") {
            if let Some(pattern) = intelligence.command_patterns.get("find") {
                suggestions.extend(pattern.examples.clone());
            }
        }

        if context.contains("process") {
            if let Some(pattern) = intelligence.command_patterns.get("ps") {
                suggestions.extend(pattern.examples.clone());
            }
        }

        if context.contains("network") || context.contains("api") {
            if let Some(pattern) = intelligence.command_patterns.get("curl") {
                suggestions.extend(pattern.examples.clone());
            }
        }

        if context.contains("monitor") || context.contains("system") {
            if let Some(cmd) = intelligence.system_commands.get("top") {
                suggestions.extend(cmd.examples.clone());
            }
            if let Some(cmd) = intelligence.system_commands.get("df") {
                suggestions.extend(cmd.examples.clone());
            }
        }

        if suggestions.is_empty() {
            suggestions.push("ls -la".to_string());
            suggestions.push("pwd".to_string());
            suggestions.push("ps aux".to_string());
        }

        Ok(suggestions)
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            ast_cache: HashMap::new(),
            symbol_table: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }

    /// Parse code using AST and extract semantic information
    pub async fn parse_code(&mut self, code: &str, language: &str, file_path: &str) -> Result<SyntaxTree> {
        let cache_key = format!("{}_{}", file_path, code.len());

        // Check cache first
        if let Some(tree) = self.ast_cache.get(&cache_key) {
            return Ok(tree.clone());
        }

        let mut parser = Parser::new();
        let language = Self::get_language(language)?;

        parser.set_language(language)?;

        let tree = parser.parse(code, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse code"))?;

        let syntax_tree = self.extract_syntax_tree(&tree, code);
        self.ast_cache.insert(cache_key, syntax_tree.clone());

        Ok(syntax_tree)
    }

    /// Extract comprehensive syntax tree information
    fn extract_syntax_tree(&mut self, tree: &Tree, source: &str) -> SyntaxTree {
        let root_node = tree.root_node();
        let root_syntax_node = self.node_to_syntax_node(&root_node, source);

        let mut symbols = HashMap::new();
        let mut imports = Vec::new();
        let mut functions = Vec::new();
        let mut classes = Vec::new();

        self.traverse_ast(&root_node, source, &mut symbols, &mut imports, &mut functions, &mut classes);

        SyntaxTree {
            root: root_syntax_node,
            symbols,
            imports,
            functions,
            classes,
        }
    }

    /// Traverse AST and extract semantic information
    fn traverse_ast(
        &mut self,
        node: &Node,
        source: &str,
        symbols: &mut HashMap<String, SymbolInfo>,
        imports: &mut Vec<ImportInfo>,
        functions: &mut Vec<FunctionInfo>,
        classes: &mut Vec<ClassInfo>,
    ) {
        match node.kind() {
            // Rust-specific patterns
            "function_item" | "function_definition" => {
                if let Some(func) = self.extract_function_info(node, source) {
                    functions.push(func);
                }
            }
            "struct_item" | "class_definition" => {
                if let Some(class) = self.extract_class_info(node, source) {
                    classes.push(class);
                }
            }
            "use_declaration" | "import_statement" => {
                if let Some(import) = self.extract_import_info(node, source) {
                    imports.push(import);
                }
            }
            "let_declaration" | "const_declaration" | "variable_declaration" => {
                if let Some(symbol) = self.extract_symbol_info(node, source, SymbolType::Variable) {
                    symbols.insert(symbol.name.clone(), symbol);
                }
            }
            _ => {}
        }

        // Recursively traverse children
        for child in node.children(&mut node.walk()) {
            self.traverse_ast(&child, source, symbols, imports, functions, classes);
        }
    }

    /// Extract function information from AST node
    fn extract_function_info(&self, node: &Node, source: &str) -> Option<FunctionInfo> {
        let mut parameters = Vec::new();
        let mut return_type = None;
        let mut docstring = None;

        // Extract function name
        let name = self.extract_node_text(node, source, "identifier")?;

        // Extract parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            for param in params_node.children(&mut params_node.walk()) {
                if param.kind() == "parameter" {
                    if let Some(param_info) = self.extract_parameter_info(&param, source) {
                        parameters.push(param_info);
                    }
                }
            }
        }

        // Extract return type
        if let Some(return_node) = node.child_by_field_name("return_type") {
            return_type = Some(return_node.utf8_text(source.as_bytes()).unwrap_or("").to_string());
        }

        // Extract docstring (look for preceding comments)
        if let Some(prev_sibling) = node.prev_sibling() {
            if prev_sibling.kind().contains("comment") {
                docstring = Some(prev_sibling.utf8_text(source.as_bytes()).unwrap_or("").to_string());
            }
        }

        let line_start = node.start_position().row as u32;
        let line_end = node.end_position().row as u32;

        // Create signature before moving parameters
        let signature = Some(format!("fn {}({})", name, parameters.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ")));

        Some(FunctionInfo {
            name,
            line_start,
            line_end,
            parameters,
            return_type,
            complexity: self.calculate_function_complexity(node),
            signature,
            docstring,
        })
    }

    /// Extract class/struct information from AST node
    fn extract_class_info(&self, node: &Node, source: &str) -> Option<ClassInfo> {
        let name = self.extract_node_text(node, source, "type_identifier")?;
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        let mut inheritance = Vec::new();
        let mut interfaces = Vec::new();

        // Extract fields, methods, inheritance, and interfaces
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "field_declaration" | "field_definition" => {
                    if let Some(field) = self.extract_field_info(&child, source) {
                        fields.push(field);
                    }
                }
                "function_item" | "method_definition" => {
                    if let Some(func) = self.extract_function_info(&child, source) {
                        methods.push(func);
                    }
                }
                "super_class" | "base_class" => {
                    if let Some(parent_class) = self.extract_node_text(&child, source, "type_identifier") {
                        inheritance.push(parent_class);
                    }
                }
                "super_interfaces" | "implements" => {
                    // Extract interface names
                    for interface_child in child.children(&mut child.walk()) {
                        if interface_child.kind() == "type_identifier" {
                            if let Some(interface_name) = self.extract_node_text(&interface_child, source, "type_identifier") {
                                interfaces.push(interface_name);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Some(ClassInfo {
            name,
            methods,
            fields,
            inheritance,
            interfaces,
        })
    }

    /// Extract import information from AST node
    fn extract_import_info(&self, node: &Node, source: &str) -> Option<ImportInfo> {
        let mut module_path = String::new();
        let mut imported_items = Vec::new();

        // Extract module path and imported items based on language
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "scoped_identifier" | "identifier" => {
                    if let Ok(text) = child.utf8_text(source.as_bytes()) {
                        module_path = text.to_string();
                    }
                }
                "import_clause" | "import_specifier" => {
                    if let Ok(text) = child.utf8_text(source.as_bytes()) {
                        imported_items.push(text.to_string());
                    }
                }
                _ => {}
            }
        }

        if module_path.is_empty() {
            return None;
        }

        let line = node.start_position().row as u32;
        let import_type = if imported_items.is_empty() {
            ImportType::Direct
        } else if imported_items.contains(&"*".to_string()) {
            ImportType::Wildcard
        } else {
            ImportType::Aliased
        };

        Some(ImportInfo {
            module_path,
            imported_items,
            line,
            import_type,
        })
    }

    /// Extract symbol information from AST node
    fn extract_symbol_info(&self, node: &Node, source: &str, symbol_type: SymbolType) -> Option<SymbolInfo> {
        let name = self.extract_node_text(node, source, "identifier")?;
        let line = node.start_position().row as u32;
        let column = node.start_position().column as u32;

        let data_type = self.extract_type_annotation(node, source);
        let visibility = Visibility::Private; // Default, TODO: Extract actual visibility

        // Use line and column information for better symbol tracking
        let position_info = format!("{}:{}", line, column);
        
        Some(SymbolInfo {
            name,
            symbol_type,
            scope: format!("local@{}", position_info), // Include position in scope for better tracking
            definition_pos: node.start_byte(),
            references: Vec::new(), // TODO: Find all references
            data_type,
            visibility,
        })
    }

    /// Extract parameter information
    fn extract_parameter_info(&self, node: &Node, source: &str) -> Option<ParameterInfo> {
        let name = self.extract_node_text(node, source, "identifier")?;
        let type_hint = self.extract_type_annotation(node, source);
        let is_mutable = node.utf8_text(source.as_bytes()).unwrap_or("").contains("mut");

        Some(ParameterInfo {
            name,
            type_hint,
            is_mutable,
        })
    }

    /// Extract field information
    fn extract_field_info(&self, node: &Node, source: &str) -> Option<FieldInfo> {
        let name = self.extract_node_text(node, source, "field_identifier")?;
        let field_type = self.extract_type_annotation(node, source).unwrap_or_else(|| "Unknown".to_string());
        
        // Detect visibility modifiers
        let mut visibility = Visibility::Private;
        let mut is_static = false;
        
        // Check for visibility modifiers and static keyword
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "public" | "visibility_modifier" => {
                    // Check the actual text to determine visibility
                    if let Some(text) = child.utf8_text(source.as_bytes()).ok() {
                        match text.trim() {
                            "public" | "pub" => visibility = Visibility::Public,
                            "private" | "priv" => visibility = Visibility::Private,
                            "protected" => visibility = Visibility::Protected,
                            _ => {}
                        }
                    }
                }
                "static" => {
                    is_static = true;
                }
                _ => {}
            }
        }
        
        // Extract default value if present
        let mut default_value = None;
        if let Some(value_child) = node.child_by_field_name("value") {
            if let Some(value_text) = value_child.utf8_text(source.as_bytes()).ok() {
                default_value = Some(value_text.to_string());
            }
        }

        Some(FieldInfo {
            name,
            field_type,
            visibility,
            is_static,
            default_value,
        })
    }

    /// Extract text from a named child node
    fn extract_node_text(&self, node: &Node, source: &str, child_name: &str) -> Option<String> {
        if let Some(child) = node.child_by_field_name(child_name) {
            child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Extract type annotation from node
    fn extract_type_annotation(&self, node: &Node, source: &str) -> Option<String> {
        if let Some(type_node) = node.child_by_field_name("type") {
            type_node.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Calculate function complexity based on AST structure
    fn calculate_function_complexity(&self, node: &Node) -> f32 {
        let mut complexity = 1.0; // Base complexity

        // Count control flow statements
        self.count_control_flow(node, &mut complexity);

        // Count operators and expressions
        self.count_operators(node, &mut complexity);

        complexity.min(10.0) // Cap at 10
    }

    /// Count control flow statements for complexity
    fn count_control_flow(&self, node: &Node, complexity: &mut f32) {
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "if_expression" | "if_statement" | "match_expression" | "match_statement" => {
                    *complexity += 1.0;
                }
                "for_loop" | "while_loop" | "loop_expression" => {
                    *complexity += 1.5;
                }
                "try_expression" | "catch_clause" => {
                    *complexity += 0.5;
                }
                _ => {}
            }
            self.count_control_flow(&child, complexity);
        }
    }

    /// Count operators for complexity
    fn count_operators(&self, node: &Node, complexity: &mut f32) {
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "binary_expression" | "unary_expression" => {
                    *complexity += 0.1;
                }
                "call_expression" | "method_call_expression" => {
                    *complexity += 0.2;
                }
                _ => {}
            }
            self.count_operators(&child, complexity);
        }
    }

    /// Convert tree-sitter node to our SyntaxNode
    fn node_to_syntax_node(&self, node: &Node, source: &str) -> SyntaxNode {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
        let mut children = Vec::new();

        for child in node.children(&mut node.walk()) {
            children.push(self.node_to_syntax_node(&child, source));
        }

        let mut attributes = HashMap::new();
        attributes.insert("kind".to_string(), node.kind().to_string());
        attributes.insert("is_named".to_string(), node.is_named().to_string());
        attributes.insert("is_missing".to_string(), node.is_missing().to_string());

        SyntaxNode {
            node_type: node.kind().to_string(),
            text,
            start_pos: node.start_byte(),
            end_pos: node.end_byte(),
            children,
            attributes,
        }
    }

    /// Get tree-sitter language for given language string
    fn get_language(language: &str) -> Result<Language> {
        match language {
            "rust" => Ok(tree_sitter_rust::language()),
            "javascript" => Ok(tree_sitter_javascript::language()),
            "typescript" => {
                // TypeScript uses the same parser as JavaScript for basic syntax
                Ok(tree_sitter_javascript::language())
            },
            "python" => Ok(tree_sitter_python::language()),
            "css" => Ok(tree_sitter_css::language()),
            _ => Err(anyhow::anyhow!("Unsupported language: {}", language)),
        }
    }

    /// Find symbols by type
    pub fn find_symbols_by_type(&self, symbol_type: SymbolType) -> Vec<&SymbolInfo> {
        self.symbol_table.values()
            .filter(|symbol| symbol.symbol_type == symbol_type)
            .collect()
    }

    /// Find symbol by name
    pub fn find_symbol(&self, name: &str) -> Option<&SymbolInfo> {
        self.symbol_table.get(name)
    }

    /// Get all symbols
    pub fn get_all_symbols(&self) -> &HashMap<String, SymbolInfo> {
        &self.symbol_table
    }

    /// Analyze dependencies between modules
    pub fn analyze_dependencies(&mut self, syntax_tree: &SyntaxTree) {
        for import in &syntax_tree.imports {
            self.dependency_graph.entry(import.module_path.clone())
                .or_insert_with(Vec::new);
        }
    }

    /// Get dependency graph
    pub fn get_dependency_graph(&self) -> &HashMap<String, Vec<String>> {
        &self.dependency_graph
    }

    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.ast_cache.clear();
        self.symbol_table.clear();
        self.dependency_graph.clear();
    }
}

impl PatternRecognizer {
    pub fn new() -> Self {
        let mut code_patterns = HashMap::new();
        let mut anti_patterns = HashMap::new();
        let mut user_patterns = HashMap::new();

        // Initialize common code patterns
        code_patterns.insert("trait_pattern".to_string(), CodePattern {
            pattern_type: PatternType::DesignPattern,
            description: "Trait-based abstraction for clean interfaces".to_string(),
            examples: vec!["pub trait Repository { ... }".to_string()],
            confidence: 0.8,
            language_specific: true,
        });

        code_patterns.insert("class_pattern".to_string(), CodePattern {
            pattern_type: PatternType::DesignPattern,
            description: "Class-based organization for object-oriented design".to_string(),
            examples: vec!["class DataProcessor: ...".to_string()],
            confidence: 0.7,
            language_specific: true,
        });

        // Initialize anti-patterns
        anti_patterns.insert("dangerous_construct".to_string(), AntiPattern {
            name: "Dangerous Construct".to_string(),
            description: "Use of potentially unsafe language constructs".to_string(),
            severity: IssueSeverity::Critical,
            fix_suggestion: "Consider safer alternatives or proper validation".to_string(),
            examples: vec!["eval(user_input)".to_string()],
        });

        anti_patterns.insert("infinite_loop".to_string(), AntiPattern {
            name: "Infinite Loop Pattern".to_string(),
            description: "Unbounded loop without clear exit condition".to_string(),
            severity: IssueSeverity::Warning,
            fix_suggestion: "Add explicit break conditions or use proper loop constructs".to_string(),
            examples: vec!["while(true) {}".to_string()],
        });

        // Initialize user patterns for common languages
        user_patterns.insert("rust".to_string(), vec![
            "vec!".to_string(),
            "Option::unwrap".to_string(),
            "Result::?".to_string(),
        ]);

        user_patterns.insert("python".to_string(), vec![
            "def __init__".to_string(),
            "with open".to_string(),
            "import typing".to_string(),
        ]);

        Self {
            _code_patterns: code_patterns,
            _anti_patterns: anti_patterns,
            _user_patterns: user_patterns,
        }
    }

    /// Recognize patterns in code context and return suggestions
    pub async fn recognize_patterns(&self, context: &str, language: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Recognize design patterns
        if let Some(pattern) = self.recognize_design_pattern(context, language) {
            suggestions.push(format!("Consider using {:?} pattern: {}", pattern.pattern_type, pattern.description));
        }

        // Detect anti-patterns and suggest fixes
        if let Some(anti_pattern) = self.detect_anti_pattern(context, language) {
            suggestions.push(format!("Anti-pattern detected: {} - {}", anti_pattern.name, anti_pattern.fix_suggestion));
        }

        // Generate user pattern-based suggestions
        if let Some(user_suggestions) = self.get_user_pattern_suggestions(language, context) {
            suggestions.extend(user_suggestions);
        }

        suggestions
    }

    /// Recognize design patterns in code
    fn recognize_design_pattern(&self, context: &str, language: &str) -> Option<&CodePattern> {
        // Simple pattern recognition based on common keywords
        if context.contains("interface") && language == "rust" {
            self._code_patterns.get("trait_pattern")
        } else if context.contains("class") && language == "python" {
            self._code_patterns.get("class_pattern")
        } else {
            None
        }
    }

    /// Detect anti-patterns
    fn detect_anti_pattern(&self, context: &str, language: &str) -> Option<&AntiPattern> {
        // Detect common anti-patterns
        if context.contains("goto") || context.contains("eval") {
            self._anti_patterns.get("dangerous_construct")
        } else if context.contains("while(true)") && language == "rust" {
            self._anti_patterns.get("infinite_loop")
        } else {
            None
        }
    }

    /// Get user-specific pattern suggestions
    fn get_user_pattern_suggestions(&self, language: &str, context: &str) -> Option<Vec<String>> {
        if let Some(patterns) = self._user_patterns.get(language) {
            let mut suggestions = Vec::new();
            for pattern in patterns {
                if context.contains(pattern) {
                    suggestions.push(format!("Based on your usage: {}", pattern));
                }
            }
            Some(suggestions)
        } else {
            None
        }
    }
}

impl ContextAnalyzer {
    pub fn new() -> Self {
        Self {
            project_context: HashMap::new(),
            user_profile: UserProfile {
                preferred_style: CodingStyle {
                    naming_convention: "snake_case".to_string(),
                    indentation: "spaces".to_string(),
                    line_length: 80,
                    bracket_style: "allman".to_string(),
                },
                common_patterns: Vec::new(),
                favorite_libraries: Vec::new(),
                coding_habits: HashMap::new(),
                error_patterns: Vec::new(),
            },
            _coding_style: CodingStyle {
                naming_convention: "snake_case".to_string(),
                indentation: "spaces".to_string(),
                line_length: 80,
                bracket_style: "allman".to_string(),
            },
            preferences: HashMap::new(),
        }
    }

    /// Analyze project structure and build context
    pub async fn analyze_project(&mut self, project_root: &str) -> Result<()> {
        // Analyze project structure
        let project_context = self.analyze_project_structure(project_root).await?;
        self.project_context.insert(project_root.to_string(), project_context);

        // Update user preferences based on project patterns
        self.update_user_preferences_from_project(project_root).await;

        Ok(())
    }

    /// Analyze project structure to understand framework, dependencies, etc.
    async fn analyze_project_structure(&self, project_root: &str) -> Result<ProjectContext> {
        let mut context = ProjectContext {
            language: self.detect_primary_language(project_root).await?,
            framework: None,
            dependencies: Vec::new(),
            coding_standards: Vec::new(),
            common_patterns: Vec::new(),
            file_structure: HashMap::new(),
        };

        // Detect framework
        context.framework = self.detect_framework(&context.language, project_root).await;

        // Analyze dependencies
        context.dependencies = self.analyze_dependencies(&context.language, project_root).await?;

        // Detect coding standards
        context.coding_standards = self.detect_coding_standards(project_root).await?;

        // Analyze common patterns
        context.common_patterns = self.analyze_common_patterns(&context, project_root).await?;

        // Build file structure map
        context.file_structure = self.build_file_structure_map(project_root).await?;

        Ok(context)
    }

    /// Detect primary programming language of the project
    async fn detect_primary_language(&self, project_root: &str) -> Result<String> {
        // Check for common language indicators
        let indicators = [
            ("rust", vec!["Cargo.toml", "src/main.rs", "src/lib.rs"]),
            ("python", vec!["requirements.txt", "setup.py", "main.py", "__init__.py"]),
            ("javascript", vec!["package.json", "index.js", "app.js"]),
            ("typescript", vec!["tsconfig.json", "index.ts"]),
            ("go", vec!["go.mod", "main.go"]),
            ("java", vec!["pom.xml", "build.gradle", "src/main/java"]),
        ];

        for (language, files) in indicators {
            for file in files {
                let path = format!("{}/{}", project_root, file);
                if std::path::Path::new(&path).exists() {
                    return Ok(language.to_string());
                }
            }
        }

        // Fallback: count file extensions
        self.detect_language_by_extensions(project_root).await
    }

    /// Detect language by analyzing file extensions
    async fn detect_language_by_extensions(&self, project_root: &str) -> Result<String> {
        let mut extension_counts = HashMap::new();

        // Walk directory and count extensions
        for entry in walkdir::WalkDir::new(project_root).max_depth(3) {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    let ext_str = extension.to_string_lossy().to_string();
                    *extension_counts.entry(ext_str).or_insert(0) += 1;
                }
            }
        }

        // Map extensions to languages
        let extension_map: HashMap<&str, &str> = [
            ("rs", "rust"),
            ("py", "python"),
            ("js", "javascript"),
            ("ts", "typescript"),
            ("go", "go"),
            ("java", "java"),
            ("cpp", "cpp"),
            ("c", "c"),
            ("php", "php"),
            ("rb", "ruby"),
        ].into_iter().collect();

        // Find most common language
        let mut max_count = 0;
        let mut detected_language = "unknown".to_string();

        for (ext, count) in extension_counts {
            if let Some(language) = extension_map.get(ext.as_str()) {
                if count > max_count {
                    max_count = count;
                    detected_language = language.to_string();
                }
            }
        }

        Ok(detected_language)
    }

    /// Detect framework used in the project
    async fn detect_framework(&self, language: &str, project_root: &str) -> Option<String> {
        match language {
            "rust" => self.detect_rust_framework(project_root).await,
            "python" => self.detect_python_framework(project_root).await,
            "javascript" | "typescript" => self.detect_js_framework(project_root).await,
            _ => None,
        }
    }

    /// Detect Rust framework
    async fn detect_rust_framework(&self, project_root: &str) -> Option<String> {
        let cargo_toml = format!("{}/Cargo.toml", project_root);
        if let Ok(content) = tokio::fs::read_to_string(&cargo_toml).await {
            if content.contains("axum") || content.contains("warp") || content.contains("rocket") {
                Some("web".to_string())
            } else if content.contains("tokio") || content.contains("async-std") {
                Some("async".to_string())
            } else if content.contains("serde") {
                Some("serialization".to_string())
            } else {
                Some("general".to_string())
            }
        } else {
            None
        }
    }

    /// Detect Python framework
    async fn detect_python_framework(&self, project_root: &str) -> Option<String> {
        let requirements = format!("{}/requirements.txt", project_root);
        if let Ok(content) = tokio::fs::read_to_string(&requirements).await {
            if content.contains("django") {
                Some("django".to_string())
            } else if content.contains("flask") {
                Some("flask".to_string())
            } else if content.contains("fastapi") {
                Some("fastapi".to_string())
            } else {
                Some("general".to_string())
            }
        } else {
            None
        }
    }

    /// Detect JavaScript/TypeScript framework
    async fn detect_js_framework(&self, project_root: &str) -> Option<String> {
        let package_json = format!("{}/package.json", project_root);
        if let Ok(content) = tokio::fs::read_to_string(&package_json).await {
            if content.contains("react") {
                Some("react".to_string())
            } else if content.contains("vue") {
                Some("vue".to_string())
            } else if content.contains("angular") {
                Some("angular".to_string())
            } else if content.contains("express") {
                Some("express".to_string())
            } else {
                Some("general".to_string())
            }
        } else {
            None
        }
    }

    /// Analyze project dependencies
    async fn analyze_dependencies(&self, language: &str, project_root: &str) -> Result<Vec<String>> {
        match language {
            "rust" => self.analyze_rust_dependencies(project_root).await,
            "python" => self.analyze_python_dependencies(project_root).await,
            "javascript" | "typescript" => self.analyze_js_dependencies(project_root).await,
            _ => Ok(Vec::new()),
        }
    }

    /// Analyze Rust dependencies from Cargo.toml
    async fn analyze_rust_dependencies(&self, project_root: &str) -> Result<Vec<String>> {
        let cargo_toml = format!("{}/Cargo.toml", project_root);
        let mut dependencies = Vec::new();

        if let Ok(content) = tokio::fs::read_to_string(&cargo_toml).await {
            // Simple dependency extraction (could be enhanced with proper TOML parsing)
            for line in content.lines() {
                if line.contains("=") && line.contains("\"") {
                    if let Some(dep) = line.split('=').next().map(|s| s.trim()) {
                        if !dep.starts_with('[') && !dep.starts_with('#') {
                            dependencies.push(dep.to_string());
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Analyze Python dependencies from requirements.txt
    async fn analyze_python_dependencies(&self, project_root: &str) -> Result<Vec<String>> {
        let requirements = format!("{}/requirements.txt", project_root);
        let mut dependencies = Vec::new();

        if let Ok(content) = tokio::fs::read_to_string(&requirements).await {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    if let Some(dep) = line.split(&['=', '>', '<'][..]).next() {
                        dependencies.push(dep.trim().to_string());
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Analyze JavaScript dependencies from package.json
    async fn analyze_js_dependencies(&self, project_root: &str) -> Result<Vec<String>> {
        let package_json = format!("{}/package.json", project_root);
        let mut dependencies = Vec::new();

        if let Ok(content) = tokio::fs::read_to_string(&package_json).await {
            // Extract dependencies from package.json (simplified)
            if let Some(start) = content.find("\"dependencies\"") {
                if let Some(end) = content[start..].find('}') {
                    let deps_section = &content[start..start + end];
                    for line in deps_section.lines() {
                        if line.contains("\": \"") {
                            if let Some(dep) = line.split("\": \"").next() {
                                if let Some(dep) = dep.split('"').last() {
                                    dependencies.push(dep.trim_matches('"').to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Detect coding standards used in the project
    async fn detect_coding_standards(&self, project_root: &str) -> Result<Vec<String>> {
        let mut standards = Vec::new();

        // Check for common config files
        let config_files = [
            ".eslintrc", ".eslintrc.js", ".eslintrc.json",
            ".prettierrc", ".prettierrc.js", ".prettierrc.json",
            "eslint.config.js", "prettier.config.js",
            ".clang-format", "_clang-format",
            "rustfmt.toml", ".rustfmt.toml",
            "pyproject.toml", "setup.cfg", ".pylintrc",
            ".editorconfig",
        ];

        for config_file in config_files {
            let path = format!("{}/{}", project_root, config_file);
            if std::path::Path::new(&path).exists() {
                match config_file {
                    ".eslintrc" | ".eslintrc.js" | ".eslintrc.json" | "eslint.config.js" => {
                        standards.push("eslint".to_string());
                    }
                    ".prettierrc" | ".prettierrc.js" | ".prettierrc.json" | "prettier.config.js" => {
                        standards.push("prettier".to_string());
                    }
                    ".clang-format" | "_clang-format" => {
                        standards.push("clang-format".to_string());
                    }
                    "rustfmt.toml" | ".rustfmt.toml" => {
                        standards.push("rustfmt".to_string());
                    }
                    "pyproject.toml" | "setup.cfg" | ".pylintrc" => {
                        standards.push("python-linting".to_string());
                    }
                    ".editorconfig" => {
                        standards.push("editorconfig".to_string());
                    }
                    _ => {}
                }
            }
        }

        Ok(standards)
    }

    /// Analyze common patterns used in the project
    async fn analyze_common_patterns(&self, context: &ProjectContext, project_root: &str) -> Result<Vec<String>> {
        let mut patterns = Vec::new();

        // Analyze a sample of files to detect patterns
        let sample_files = self.get_sample_files(context, project_root, 5).await?;

        for file_path in sample_files {
            if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
                match context.language.as_str() {
                    "rust" => {
                        patterns.extend(self.detect_rust_patterns(&content));
                    }
                    "python" => {
                        patterns.extend(self.detect_python_patterns(&content));
                    }
                    "javascript" | "typescript" => {
                        patterns.extend(self.detect_js_patterns(&content));
                    }
                    _ => {}
                }
            }
        }

        // Remove duplicates and sort
        patterns.sort();
        patterns.dedup();
        Ok(patterns)
    }

    /// Get sample files for pattern analysis
    async fn get_sample_files(&self, context: &ProjectContext, project_root: &str, max_files: usize) -> Result<Vec<String>> {
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(project_root).max_depth(3) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    if let Some(extension) = entry.path().extension() {
                        let ext_str = extension.to_string_lossy();
                        if self.is_source_file(&context.language, &ext_str) {
                            files.push(entry.path().to_string_lossy().to_string());
                            if files.len() >= max_files {
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Check if file is a source file for the given language
    fn is_source_file(&self, language: &str, extension: &str) -> bool {
        match language {
            "rust" => extension == "rs",
            "python" => extension == "py",
            "javascript" => extension == "js",
            "typescript" => extension == "ts",
            "go" => extension == "go",
            "java" => extension == "java",
            "cpp" => extension == "cpp" || extension == "cc" || extension == "cxx",
            "c" => extension == "c" || extension == "h",
            _ => false,
        }
    }

    /// Detect common Rust patterns
    fn detect_rust_patterns(&self, content: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        if content.contains("#[derive(") {
            patterns.push("derive-macros".to_string());
        }
        if content.contains("impl ") {
            patterns.push("trait-implementations".to_string());
        }
        if content.contains("async fn ") {
            patterns.push("async-functions".to_string());
        }
        if content.contains("Result<") || content.contains("Option<") {
            patterns.push("error-handling".to_string());
        }
        if content.contains("tokio::") {
            patterns.push("tokio-runtime".to_string());
        }

        patterns
    }

    /// Detect common Python patterns
    fn detect_python_patterns(&self, content: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        if content.contains("def __init__(") {
            patterns.push("class-constructors".to_string());
        }
        if content.contains("async def ") {
            patterns.push("async-functions".to_string());
        }
        if content.contains("try:") && content.contains("except") {
            patterns.push("exception-handling".to_string());
        }
        if content.contains("import ") {
            patterns.push("imports".to_string());
        }
        if content.contains("@") && content.contains("def ") {
            patterns.push("decorators".to_string());
        }

        patterns
    }

    /// Detect common JavaScript/TypeScript patterns
    fn detect_js_patterns(&self, content: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        if content.contains("async ") && content.contains("await ") {
            patterns.push("async-await".to_string());
        }
        if content.contains("=>") {
            patterns.push("arrow-functions".to_string());
        }
        if content.contains("class ") {
            patterns.push("es6-classes".to_string());
        }
        if content.contains("import ") || content.contains("export ") {
            patterns.push("es6-modules".to_string());
        }
        if content.contains("try ") && content.contains("catch") {
            patterns.push("try-catch".to_string());
        }

        patterns
    }

    /// Build file structure map
    async fn build_file_structure_map(&self, project_root: &str) -> Result<HashMap<String, Vec<String>>> {
        let mut structure = HashMap::new();

        for entry in walkdir::WalkDir::new(project_root).max_depth(2) {
            if let Ok(entry) = entry {
                if entry.file_type().is_dir() {
                    let path = entry.path().strip_prefix(project_root).unwrap_or(entry.path());
                    let path_str = path.to_string_lossy().to_string();

                    if !path_str.is_empty() && !path_str.contains('.') {
                        let mut files = Vec::new();

                        // Get files in this directory
                        if let Ok(read_dir) = std::fs::read_dir(entry.path()) {
                            for file_entry in read_dir {
                                if let Ok(file_entry) = file_entry {
                                    if file_entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                                        files.push(file_entry.file_name().to_string_lossy().to_string());
                                    }
                                }
                            }
                        }

                        if !files.is_empty() {
                            structure.insert(path_str, files);
                        }
                    }
                }
            }
        }

        Ok(structure)
    }

    /// Update user preferences based on project analysis
    async fn update_user_preferences_from_project(&mut self, project_root: &str) {
        if let Some(context) = self.project_context.get(project_root).cloned() {
            // Update coding style preferences
            self.update_coding_style_from_project(&context);

            // Update favorite libraries
            self.update_favorite_libraries_from_project(&context);

            // Update coding habits
            self.update_coding_habits_from_project(&context);
        }
    }

    /// Update coding style from project analysis
    fn update_coding_style_from_project(&mut self, context: &ProjectContext) {
        // Detect indentation style
        if context.coding_standards.contains(&"editorconfig".to_string()) {
            // Could parse .editorconfig for specific settings
            // For now, keep defaults
        }

        // Update naming convention based on language
        match context.language.as_str() {
            "rust" | "python" => {
                self.user_profile.preferred_style.naming_convention = "snake_case".to_string();
                self._coding_style.naming_convention = "snake_case".to_string();
            }
            "javascript" | "typescript" => {
                self.user_profile.preferred_style.naming_convention = "camelCase".to_string();
                self._coding_style.naming_convention = "camelCase".to_string();
            }
            "java" | "c#" => {
                self._coding_style.naming_convention = "PascalCase".to_string();
                self.user_profile.preferred_style.naming_convention = "PascalCase".to_string();
            }
            _ => {}
        }
    }

    /// Update favorite libraries from project
    fn update_favorite_libraries_from_project(&mut self, context: &ProjectContext) {
        // Add common dependencies to favorites
        for dep in &context.dependencies {
            if !self.user_profile.favorite_libraries.contains(dep) {
                self.user_profile.favorite_libraries.push(dep.clone());
            }
        }
    }

    /// Update coding habits from project
    fn update_coding_habits_from_project(&mut self, context: &ProjectContext) {
        // Update habits based on detected patterns
        for pattern in &context.common_patterns {
            let habit_key = format!("uses_{}", pattern.replace('-', "_"));
            self.user_profile.coding_habits.insert(habit_key, 1.0);
        }
    }

    /// Get context-aware suggestions for code completion
    pub fn get_context_suggestions(&self, language: &str, current_context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Get language-specific suggestions
        match language {
            "rust" => {
                suggestions.extend(self.get_rust_context_suggestions(current_context));
            }
            "python" => {
                suggestions.extend(self.get_python_context_suggestions(current_context));
            }
            "javascript" | "typescript" => {
                suggestions.extend(self.get_js_context_suggestions(current_context));
            }
            _ => {}
        }

        // Add user preference-based suggestions
        suggestions.extend(self.get_user_preference_suggestions(language, current_context));

        suggestions
    }

    /// Get Rust-specific context suggestions
    fn get_rust_context_suggestions(&self, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        if context.contains("fn ") && !context.contains("->") {
            // Suggest return types based on common patterns
            if context.contains("parse") || context.contains("read") {
                suggestions.push("-> Result<T, E>".to_string());
            } else if context.contains("get") || context.contains("find") {
                suggestions.push("-> Option<T>".to_string());
            }
        }

        if context.contains("let ") && !context.contains(":") {
            // Suggest type annotations
            suggestions.push(": Type".to_string());
        }

        suggestions
    }

    /// Get Python-specific context suggestions
    fn get_python_context_suggestions(&self, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        if context.contains("def ") && !context.contains("->") {
            // Suggest return type annotations
            suggestions.push(" -> ReturnType:".to_string());
        }

        if context.contains("for ") && !context.contains(" in ") {
            // Suggest iteration patterns
            suggestions.push(" item in iterable:".to_string());
        }

        suggestions
    }

    /// Get JavaScript/TypeScript-specific context suggestions
    fn get_js_context_suggestions(&self, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        if context.contains("function ") && !context.contains("{") {
            // Suggest function body
            suggestions.push("(param) { // implementation }".to_string());
        }

        if context.contains("const ") && context.contains("= ") && !context.contains(";") {
            // Suggest common patterns
            suggestions.push("() => { // implementation };".to_string());
        }

        suggestions
    }

    /// Get suggestions based on user preferences
    fn get_user_preference_suggestions(&self, language: &str, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Use language-specific preferences and patterns
        match language {
            "rust" => {
                suggestions.extend(self.get_rust_specific_suggestions(context));
            }
            "python" => {
                suggestions.extend(self.get_python_specific_suggestions(context));
            }
            "javascript" | "typescript" => {
                suggestions.extend(self.get_js_specific_suggestions(context));
            }
            _ => {
                suggestions.extend(self.get_generic_suggestions(context));
            }
        }

        // Apply user coding style preferences
        if let Some(indent_pref) = self.preferences.get("preferred_indentation") {
            if indent_pref == &1.0 {
                suggestions.push("Use spaces for indentation".to_string());
            } else if indent_pref == &0.0 {
                suggestions.push("Use tabs for indentation".to_string());
            }
        }

        // Apply naming convention preferences based on language
        if self.should_suggest_naming(language, context) {
            match self.user_profile.preferred_style.naming_convention.as_str() {
                "snake_case" => {
                    suggestions.push("snake_case variable names".to_string());
                }
                "camelCase" => {
                    suggestions.push("camelCase variable names".to_string());
                }
                "PascalCase" => {
                    suggestions.push("PascalCase class/struct names".to_string());
                }
                _ => {}
            }
        }

        suggestions
    }

    /// Get Rust-specific suggestions
    fn get_rust_specific_suggestions(&self, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if context.contains("fn ") {
            suggestions.push("Consider using Result<T, E> for error handling".to_string());
            suggestions.push("Add #[test] functions for unit testing".to_string());
        }
        if context.contains("struct ") {
            suggestions.push("Implement Debug trait for debugging".to_string());
            suggestions.push("Consider deriving Clone, Copy if applicable".to_string());
        }
        if context.contains("impl ") {
            suggestions.push("Use trait implementations for better abstractions".to_string());
        }
        
        suggestions
    }

    /// Get Python-specific suggestions
    fn get_python_specific_suggestions(&self, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if context.contains("def ") {
            suggestions.push("Add type hints for better code documentation".to_string());
            suggestions.push("Include docstrings for function documentation".to_string());
        }
        if context.contains("class ") {
            suggestions.push("Implement __str__ or __repr__ for better string representation".to_string());
            suggestions.push("Consider inheriting from ABC if creating abstract base classes".to_string());
        }
        if context.contains("import ") {
            suggestions.push("Group imports according to PEP 8 (standard, third-party, local)".to_string());
        }
        
        suggestions
    }

    /// Get JavaScript/TypeScript-specific suggestions
    fn get_js_specific_suggestions(&self, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if context.contains("function ") {
            suggestions.push("Consider using arrow functions for cleaner syntax".to_string());
            suggestions.push("Add JSDoc comments for better documentation".to_string());
        }
        if context.contains("class ") {
            suggestions.push("Use constructor parameters with default values".to_string());
            suggestions.push("Implement private fields with # prefix".to_string());
        }
        if context.contains("const ") || context.contains("let ") {
            suggestions.push("Prefer const over let when variable won't be reassigned".to_string());
        }
        
        suggestions
    }

    /// Get generic suggestions for unknown languages
    fn get_generic_suggestions(&self, _context: &str) -> Vec<String> {
        vec!["Consider adding comments for code clarity".to_string()]
    }

    /// Determine if naming suggestions should be provided
    fn should_suggest_naming(&self, language: &str, context: &str) -> bool {
        match language {
            "rust" => context.contains("let ") || context.contains("fn "),
            "python" => context.contains("def ") || context.contains("class "),
            "javascript" | "typescript" => context.contains("const ") || context.contains("function "),
            _ => false,
        }
    }

    /// Learn from user behavior and update preferences
    pub async fn learn_from_user_action(&mut self, action: &str, context: &str, language: &str) {
        // Update coding habits
        let habit_key = format!("{}_in_{}", action, language);
        let count = self.user_profile.coding_habits.entry(habit_key).or_insert(0.0);
        *count += 1.0;

        // Update error patterns if action indicates error handling
        if action.contains("error") || action.contains("exception") {
            if !self.user_profile.error_patterns.contains(&context.to_string()) {
                self.user_profile.error_patterns.push(context.to_string());
            }
        }

        // Update common patterns
        if !self.user_profile.common_patterns.contains(&context.to_string()) {
            self.user_profile.common_patterns.push(context.to_string());
        }
    }

    /// Get project context for a given path
    pub fn get_project_context(&self, project_path: &str) -> Option<&ProjectContext> {
        self.project_context.get(project_path)
    }

    /// Get user profile
    pub fn get_user_profile(&self) -> &UserProfile {
        &self.user_profile
    }

    /// Update user preferences
    pub fn update_preference(&mut self, key: String, value: f32) {
        self.preferences.insert(key, value);
    }

    /// Get preference value
    pub fn get_preference(&self, key: &str) -> Option<f32> {
        self.preferences.get(key).copied()
    }

    /// Clear all context data
    pub fn clear_context(&mut self) {
        self.project_context.clear();
        // Keep user profile and preferences
    }
}

impl SecurityAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            vulnerability_patterns: HashMap::new(),
            security_rules: Vec::new(),
            _threat_models: HashMap::new(),
        };

        analyzer.initialize_vulnerability_patterns();
        analyzer.initialize_security_rules();
        analyzer.initialize_threat_models();
        analyzer
    }

    /// Initialize common vulnerability patterns
    fn initialize_vulnerability_patterns(&mut self) {
        // SQL Injection patterns
        self.vulnerability_patterns.insert("sql_injection".to_string(), Vulnerability {
            vuln_type: VulnerabilityType::Injection,
            severity: IssueSeverity::Critical,
            description: "Potential SQL injection vulnerability".to_string(),
            cwe_id: Some("CWE-89".to_string()),
            fix_guidance: "Use parameterized queries or prepared statements".to_string(),
            examples: vec![
                "SELECT * FROM users WHERE id = '".to_string(),
                "query(\"SELECT * FROM table WHERE id = \" + userInput)".to_string(),
            ],
        });

        // XSS patterns
        self.vulnerability_patterns.insert("xss_vulnerability".to_string(), Vulnerability {
            vuln_type: VulnerabilityType::InputValidation,
            severity: IssueSeverity::Critical,
            description: "Cross-site scripting vulnerability".to_string(),
            cwe_id: Some("CWE-79".to_string()),
            fix_guidance: "Sanitize user input and use safe encoding methods".to_string(),
            examples: vec![
                "element.innerHTML = userInput".to_string(),
                "document.write(userInput)".to_string(),
            ],
        });

        // Command injection
        self.vulnerability_patterns.insert("command_injection".to_string(), Vulnerability {
            vuln_type: VulnerabilityType::Injection,
            severity: IssueSeverity::Critical,
            description: "Command injection vulnerability".to_string(),
            cwe_id: Some("CWE-78".to_string()),
            fix_guidance: "Use safe APIs and validate/sanitize input".to_string(),
            examples: vec![
                "exec(\"ls \" + userInput)".to_string(),
                "system(userInput)".to_string(),
            ],
        });

        // Authentication bypass
        self.vulnerability_patterns.insert("auth_bypass".to_string(), Vulnerability {
            vuln_type: VulnerabilityType::Authentication,
            severity: IssueSeverity::Critical,
            description: "Potential authentication bypass".to_string(),
            cwe_id: Some("CWE-287".to_string()),
            fix_guidance: "Implement proper authentication and authorization checks".to_string(),
            examples: vec![
                "if (user == \"admin\")".to_string(),
                "password == \"password\"".to_string(),
            ],
        });
    }

    /// Initialize threat models for common attack vectors
    fn initialize_threat_models(&mut self) {
        // SQL Injection threat model
        self._threat_models.insert("sql_injection_model".to_string(), ThreatModel {
            component: "Database Layer".to_string(),
            threats: vec!["SQL Injection".to_string(), "Data breach".to_string()],
            mitigations: vec![
                "Use parameterized queries".to_string(),
                "Input validation".to_string(),
                "Least privilege database access".to_string(),
            ],
            trust_boundaries: vec!["User Input".to_string(), "Database".to_string()],
        });

        // XSS threat model
        self._threat_models.insert("xss_model".to_string(), ThreatModel {
            component: "Web Interface".to_string(),
            threats: vec!["Cross-Site Scripting".to_string(), "Session hijacking".to_string()],
            mitigations: vec![
                "Input sanitization".to_string(),
                "Output encoding".to_string(),
                "Content Security Policy".to_string(),
            ],
            trust_boundaries: vec!["Browser".to_string(), "Server".to_string()],
        });

        // Authentication bypass threat model
        self._threat_models.insert("auth_bypass_model".to_string(), ThreatModel {
            component: "Authentication System".to_string(),
            threats: vec!["Authentication Bypass".to_string(), "Unauthorized access".to_string()],
            mitigations: vec![
                "Multi-factor authentication".to_string(),
                "Strong password policies".to_string(),
                "Session management".to_string(),
            ],
            trust_boundaries: vec!["User".to_string(), "Application".to_string()],
        });
    }

    /// Initialize security rules
    fn initialize_security_rules(&mut self) {
        self.security_rules.push(SecurityRule {
            rule_id: "hardcoded_secrets".to_string(),
            name: "Hardcoded Secrets".to_string(),
            description: "Detection of hardcoded passwords, API keys, and other secrets".to_string(),
            severity: IssueSeverity::Critical,
            language: "any".to_string(),
            pattern: r"(?i)(password|secret|key|token)\s*[=:]\s*['\'][^\']{8,}['\']".to_string(),
        });

        self.security_rules.push(SecurityRule {
            rule_id: "unsafe_deserialization".to_string(),
            name: "Unsafe Deserialization".to_string(),
            description: "Detection of potentially unsafe deserialization operations".to_string(),
            severity: IssueSeverity::Warning,
            language: "any".to_string(),
            pattern: r"(?i)(pickle\.loads?|yaml\.load|json\.loads?)".to_string(),
        });

        self.security_rules.push(SecurityRule {
            rule_id: "weak_crypto".to_string(),
            name: "Weak Cryptography".to_string(),
            description: "Detection of weak cryptographic algorithms".to_string(),
            severity: IssueSeverity::Warning,
            language: "any".to_string(),
            pattern: r"(?i)(md5|sha1|des|rc4)".to_string(),
        });
    }

    /// Analyze Rust code for security vulnerabilities
    pub fn analyze_rust_security(&self, code: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // Check for unsafe environment variable access
            if line.contains("std::env::var(") && !line.contains("expect(") && !line.contains("unwrap_or") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    message: "Environment variable access should handle missing variables gracefully".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for potential command injection
            if line.contains("std::process::Command") && (line.contains("arg(") || line.contains("args(")) {
                if line.contains("user_input") || line.contains("input") {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Warning,
                        message: "Potential command injection: user input passed to Command".to_string(),
                        line: line_num as u32 + 1,
                        column: 1,
                    });
                }
            }

            // Check for unsafe code blocks
            if line.contains("unsafe {") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Info,
                    message: "Unsafe code block detected - review for safety".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for unwrap() usage
            if line.contains(".unwrap()") && !line.contains("expect(") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Info,
                    message: "Consider using expect() or proper error handling instead of unwrap()".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }
        }

        // Apply security rules
        for rule in &self.security_rules {
            if rule.language == "any" || rule.language == "rust" {
                if let Ok(regex) = regex::Regex::new(&rule.pattern) {
                    for (line_num, line) in lines.iter().enumerate() {
                        if regex.is_match(line) {
                            issues.push(CodeIssue {
                                id: Uuid::new_v4().to_string(),
                                severity: rule.severity.clone(),
                                message: format!("Security issue: {}", rule.description),
                                line: line_num as u32 + 1,
                                column: 1,
                            });
                        }
                    }
                }
            }
        }

        issues
    }

    /// Analyze JavaScript/TypeScript code for security vulnerabilities
    pub fn analyze_js_security(&self, code: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // Check for eval usage
            if line.contains("eval(") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Critical,
                    message: "Use of eval() can lead to code injection vulnerabilities".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for innerHTML assignments
            if line.contains("innerHTML") && line.contains("=") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    message: "Direct innerHTML assignment can lead to XSS vulnerabilities".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for document.write
            if line.contains("document.write(") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    message: "document.write() can lead to XSS and performance issues".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for localStorage/sessionStorage with sensitive data
            if (line.contains("localStorage.setItem(") || line.contains("sessionStorage.setItem("))
                && (line.to_lowercase().contains("password") || line.to_lowercase().contains("token")) {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    message: "Storing sensitive data in web storage - consider secure alternatives".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for HTTP URLs in production code
            if line.contains("http://") && !line.contains("localhost") && !line.contains("127.0.0.1") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Info,
                    message: "HTTP URLs detected - consider using HTTPS in production".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }
        }

        issues
    }

    /// Analyze Python code for security vulnerabilities
    pub fn analyze_python_security(&self, code: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // Check for exec/eval usage
            if line.contains("exec(") || line.contains("eval(") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Critical,
                    message: "Use of exec() or eval() can lead to code injection".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for subprocess with shell=True
            if line.contains("subprocess.") && line.contains("shell=True") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    message: "shell=True in subprocess can lead to command injection".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for pickle usage
            if line.contains("pickle.loads") || line.contains("pickle.load") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    message: "Pickle deserialization can lead to remote code execution".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for SQL injection patterns
            if line.contains("cursor.execute(") || line.contains("execute(") {
                if line.contains("%") && (line.contains("request.") || line.contains("input")) {
                    issues.push(CodeIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Critical,
                        message: "Potential SQL injection - use parameterized queries".to_string(),
                        line: line_num as u32 + 1,
                        column: 1,
                    });
                }
            }
        }

        issues
    }

    /// Analyze code for logic errors
    pub fn analyze_logic_errors(&self, code: &str, language: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        match language {
            "rust" => issues.extend(self.analyze_rust_logic(code)),
            "javascript" | "typescript" => issues.extend(self.analyze_js_logic(code)),
            "python" => issues.extend(self.analyze_python_logic(code)),
            _ => {}
        }

        issues
    }

    /// Get security threat analysis based on threat models
    pub fn get_threat_analysis(&self, code: &str, _language: &str) -> Vec<String> {
        let mut threats = Vec::new();

        // Check code against known threat models
        for (_model_id, threat_model) in &self._threat_models {
            let mut threat_detected = false;
            let mut relevant_lines = Vec::new();

            let lines: Vec<&str> = code.lines().collect();
            for (line_num, line) in lines.iter().enumerate() {
                // Simple heuristic: check if code patterns match threat model
                if threat_model.threats.iter().any(|t| {
                    match t.as_str() {
                        "SQL Injection" => line.contains("query(") || line.contains("SELECT") || line.contains("WHERE"),
                        "Cross-Site Scripting" => line.contains("innerHTML") || line.contains("document.write"),
                        "Authentication Bypass" => line.contains("==") || line.contains("password"),
                        _ => false,
                    }
                }) {
                    threat_detected = true;
                    relevant_lines.push(line_num + 1);
                }
            }

            if threat_detected {
                threats.push(format!(
                    "Threat Model '{}' detected in component: {}\nRelevant lines: {:?}\nMitigation: {}",
                    threat_model.threats.join(", "),
                    threat_model.component,
                    relevant_lines,
                    threat_model.mitigations.join(", ")
                ));
            }
        }

        threats
    }

    /// Analyze Rust code for logic errors
    fn analyze_rust_logic(&self, code: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // Check for potential null pointer dereferences
            if line.contains("as_ref()") && line.contains(".unwrap()") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    message: "Potential null pointer dereference after as_ref()".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for infinite loops
            if line.contains("loop {") && !code.lines().skip(line_num).any(|l| l.contains("break")) {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Info,
                    message: "Infinite loop detected - ensure break condition exists".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for unused variables (simple heuristic)
            if line.contains("let ") && line.contains("_") && !line.contains("mut") {
                let var_name = line.split("let ").nth(1)
                    .and_then(|s| s.split(":").next())
                    .map(|s| s.trim().trim_start_matches('_'));

                if let Some(var) = var_name {
                    // Check if variable is used later (simple check)
                    let remaining_code = &lines[line_num..];
                    let is_used = remaining_code.iter()
                        .any(|l| l.contains(var) && !l.contains("let ") && !l.contains("const "));

                    if !is_used {
                        issues.push(CodeIssue {
                            id: Uuid::new_v4().to_string(),
                            severity: IssueSeverity::Info,
                            message: format!("Unused variable '{}' detected", var),
                            line: line_num as u32 + 1,
                            column: 1,
                        });
                    }
                }
            }
        }

        issues
    }

    /// Analyze JavaScript/TypeScript code for logic errors
    fn analyze_js_logic(&self, code: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // Check for == instead of ===
            if line.contains(" == ") && !line.contains("===") && !line.contains("!=") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    message: "Consider using === instead of == for strict equality".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for potential infinite loops
            if line.contains("while (true)") || line.contains("for (;;)") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Info,
                    message: "Potential infinite loop - ensure break condition".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for console.log in production code
            if line.contains("console.log(") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Info,
                    message: "console.log statement found - remove for production".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }
        }

        issues
    }

    /// Analyze Python code for logic errors
    fn analyze_python_logic(&self, code: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // Check for bare except clauses
            if line.trim() == "except:" {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    message: "Bare 'except:' clause catches all exceptions including system exits".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for assert statements in production code
            if line.contains("assert ") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Info,
                    message: "Assert statements are removed with -O flag - use proper error handling".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }

            // Check for print statements (similar to console.log)
            if line.trim().starts_with("print(") {
                issues.push(CodeIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Info,
                    message: "Print statement found - consider using logging for production".to_string(),
                    line: line_num as u32 + 1,
                    column: 1,
                });
            }
        }

        issues
    }
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            _performance_patterns: HashMap::new(),
            _optimization_rules: Vec::new(),
            _benchmark_data: HashMap::new(),
        };

        analyzer.initialize_performance_patterns();
        analyzer.initialize_optimization_rules();
        analyzer.initialize_benchmark_data();
        analyzer
    }

    /// Initialize common performance patterns
    fn initialize_performance_patterns(&mut self) {
        // Memory allocation patterns
        self._performance_patterns.insert("excessive_allocation".to_string(), PerformanceIssue {
            issue_type: PerformanceIssueType::MemoryAllocation,
            description: "Excessive memory allocation detected".to_string(),
            impact: crate::ai::PerformanceImpact::Medium,
            optimization_suggestion: "Consider reusing objects or using more efficient data structures".to_string(),
            examples: vec!["vec![0; 1000000]".to_string()],
        });

        // CPU intensive patterns
        self._performance_patterns.insert("nested_loops".to_string(), PerformanceIssue {
            issue_type: PerformanceIssueType::AlgorithmComplexity,
            description: "Nested loops detected - potential O(n²) complexity".to_string(),
            impact: crate::ai::PerformanceImpact::High,
            optimization_suggestion: "Consider algorithm optimization or data structure improvements".to_string(),
            examples: vec!["for i in 0..n { for j in 0..n { ... } }".to_string()],
        });

        // I/O patterns
        self._performance_patterns.insert("frequent_io".to_string(), PerformanceIssue {
            issue_type: PerformanceIssueType::IOOperations,
            description: "Frequent I/O operations detected".to_string(),
            impact: crate::ai::PerformanceImpact::Low,
            optimization_suggestion: "Consider batching I/O operations or using caching".to_string(),
            examples: vec!["File::open() in loop".to_string()],
        });
    }

    /// Initialize optimization rules
    fn initialize_optimization_rules(&mut self) {
        self._optimization_rules.push(OptimizationRule {
            rule_id: "string_concatenation".to_string(),
            name: "Optimize String Concatenation".to_string(),
            description: "Use String::with_capacity or StringBuilder for multiple concatenations".to_string(),
            pattern: r"\+.*\+.*\+".to_string(),
            replacement: "String::with_capacity() or format!()".to_string(),
            performance_gain: 0.3,
            language: "rust".to_string(),
        });

        self._optimization_rules.push(OptimizationRule {
            rule_id: "hashmap_iteration".to_string(),
            name: "Optimize HashMap Iteration".to_string(),
            description: "Use iter() instead of into_iter() when possible".to_string(),
            pattern: r"\.into_iter\(\)".to_string(),
            replacement: ".iter()".to_string(),
            performance_gain: 0.2,
            language: "rust".to_string(),
        });
    }

    /// Initialize benchmark data
    fn initialize_benchmark_data(&mut self) {
        // Common operation benchmarks
        self._benchmark_data.insert("vector_push".to_string(), PerformanceMetrics {
            operation_name: "Vector Push".to_string(),
            duration_ms: 0.005,
            memory_usage_kb: 8,
            cpu_usage_percent: 1.0,
        });

        self._benchmark_data.insert("hashmap_lookup".to_string(), PerformanceMetrics {
            operation_name: "HashMap Lookup".to_string(),
            duration_ms: 0.013,
            memory_usage_kb: 16,
            cpu_usage_percent: 2.0,
        });

        self._benchmark_data.insert("file_read".to_string(), PerformanceMetrics {
            operation_name: "File Read (1KB)".to_string(),
            duration_ms: 1.5,
            memory_usage_kb: 1024,
            cpu_usage_percent: 5.0,
        });
    }

    /// Get performance insights based on patterns and benchmarks
    pub fn get_performance_insights(&self, code: &str, _language: &str) -> Vec<String> {
        let mut insights = Vec::new();

        // Check for performance patterns
        for (pattern_id, pattern) in &self._performance_patterns {
            if self.detect_performance_pattern(code, pattern) {
                insights.push(format!(
                    "Performance Pattern '{}': {}\nSuggestion: {}\nImpact: {:?}",
                    pattern_id,
                    pattern.description,
                    pattern.optimization_suggestion,
                    pattern.impact
                ));
            }
        }

        // Compare against benchmark data
        for (operation, metrics) in &self._benchmark_data {
            if code.contains(&operation.replace("_", " ")) {
                insights.push(format!(
                    "Benchmark Reference '{}':\nDuration: {:.3}ms, Memory: {}KB, CPU: {:.1}%",
                    metrics.operation_name,
                    metrics.duration_ms,
                    metrics.memory_usage_kb,
                    metrics.cpu_usage_percent
                ));
            }
        }

        insights
    }

    /// Detect if code contains a specific performance pattern
    fn detect_performance_pattern(&self, code: &str, pattern: &PerformanceIssue) -> bool {
        match pattern.issue_type {
            PerformanceIssueType::MemoryAllocation => {
                code.contains("clone()") || code.contains("String::new()")
            }
            PerformanceIssueType::AlgorithmComplexity => {
                code.contains("for ") && code.contains("for ")
            }
            PerformanceIssueType::IOOperations => {
                code.contains("read_to_string") || code.contains("read_line")
            }
            _ => false,
        }
    }
}

impl RefactoringEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            refactoring_patterns: HashMap::new(),
            code_smells: HashMap::new(),
            transformation_rules: Vec::new(),
        };

        engine.initialize_refactoring_patterns();
        engine.initialize_code_smells();
        engine.initialize_transformation_rules();

        engine
    }

    /// Initialize common refactoring patterns
    fn initialize_refactoring_patterns(&mut self) {
        // Extract Method
        self.refactoring_patterns.insert("extract_method".to_string(), RefactoringPattern {
            name: "Extract Method".to_string(),
            description: "Extract a code fragment into a separate method".to_string(),
            before_pattern: "function original() {\n    // setup code\n    // extracted code\n    // more code\n}".to_string(),
            after_pattern: "function extractedMethod() {\n    // extracted code\n}\n\nfunction original() {\n    // setup code\n    extractedMethod();\n    // more code\n}".to_string(),
            benefits: vec![
                "Improves readability".to_string(),
                "Reduces code duplication".to_string(),
                "Enhances testability".to_string(),
            ],
            risk_level: RiskLevel::Low,
        });

        // Rename Variable/Method
        self.refactoring_patterns.insert("rename_symbol".to_string(), RefactoringPattern {
            name: "Rename Symbol".to_string(),
            description: "Rename a variable, method, or class with proper updates throughout the codebase".to_string(),
            before_pattern: "let oldName = value;".to_string(),
            after_pattern: "let newName = value;".to_string(),
            benefits: vec![
                "Improves code clarity".to_string(),
                "Maintains consistency".to_string(),
                "Reduces confusion".to_string(),
            ],
            risk_level: RiskLevel::Low,
        });

        // Move Method
        self.refactoring_patterns.insert("move_method".to_string(), RefactoringPattern {
            name: "Move Method".to_string(),
            description: "Move a method from one class to another".to_string(),
            before_pattern: "class Source {\n    method() {}\n}\n\nclass Target {}".to_string(),
            after_pattern: "class Source {}\n\nclass Target {\n    method() {}\n}".to_string(),
            benefits: vec![
                "Improves cohesion".to_string(),
                "Reduces coupling".to_string(),
                "Better encapsulation".to_string(),
            ],
            risk_level: RiskLevel::Medium,
        });

        // Replace Conditional with Polymorphism
        self.refactoring_patterns.insert("replace_conditional_with_polymorphism".to_string(), RefactoringPattern {
            name: "Replace Conditional with Polymorphism".to_string(),
            description: "Replace type-checking conditional with polymorphism".to_string(),
            before_pattern: "if (type === 'A') {\n    methodA();\n} else if (type === 'B') {\n    methodB();\n}".to_string(),
            after_pattern: "class HandlerA {\n    handle() { methodA(); }\n}\n\nclass HandlerB {\n    handle() { methodB(); }\n}".to_string(),
            benefits: vec![
                "Eliminates type checking".to_string(),
                "Improves extensibility".to_string(),
                "Follows Open/Closed Principle".to_string(),
            ],
            risk_level: RiskLevel::High,
        });
    }

    /// Initialize code smell patterns
    fn initialize_code_smells(&mut self) {
        // Long Method
        self.code_smells.insert("long_method".to_string(), CodeSmell {
            name: "Long Method".to_string(),
            description: "A method that is too long and does too much".to_string(),
            severity: IssueSeverity::Warning,
            detection_pattern: r"(?s)function\s+\w+\([^)]*\)\s*\{.*?\}".to_string(),
            refactoring_suggestion: "Extract Method: Break down into smaller, focused methods".to_string(),
        });

        // Large Class
        self.code_smells.insert("large_class".to_string(), CodeSmell {
            name: "Large Class".to_string(),
            description: "A class that has grown too large and handles too many responsibilities".to_string(),
            severity: IssueSeverity::Warning,
            detection_pattern: r"(?s)class\s+\w+.*?\}".to_string(),
            refactoring_suggestion: "Extract Class: Split into smaller classes with single responsibilities".to_string(),
        });

        // Long Parameter List
        self.code_smells.insert("long_parameter_list".to_string(), CodeSmell {
            name: "Long Parameter List".to_string(),
            description: "A method with too many parameters".to_string(),
            severity: IssueSeverity::Info,
            detection_pattern: r"function\s+\w+\((?:\w+:\s*\w+,\s*){5,}".to_string(),
            refactoring_suggestion: "Introduce Parameter Object: Group related parameters into an object".to_string(),
        });

        // Duplicate Code
        self.code_smells.insert("duplicate_code".to_string(), CodeSmell {
            name: "Duplicate Code".to_string(),
            description: "Code that appears in multiple places".to_string(),
            severity: IssueSeverity::Warning,
            detection_pattern: r".*".to_string(), // Would need more sophisticated duplicate detection
            refactoring_suggestion: "Extract Method: Move duplicated code to a shared method".to_string(),
        });

        // Feature Envy
        self.code_smells.insert("feature_envy".to_string(), CodeSmell {
            name: "Feature Envy".to_string(),
            description: "A method that seems more interested in another class than its own".to_string(),
            severity: IssueSeverity::Info,
            detection_pattern: r".*".to_string(), // Complex pattern requiring AST analysis
            refactoring_suggestion: "Move Method: Move the method to the class it uses most".to_string(),
        });
    }

    /// Initialize transformation rules
    fn initialize_transformation_rules(&mut self) {
        // Arrow function conversion (JavaScript/TypeScript)
        self.transformation_rules.push(TransformationRule {
            from_pattern: r"function\s+(\w+)\s*\(([^)]*)\)\s*\{\s*return\s+([^;]+);\s*\}".to_string(),
            to_pattern: "const $1 = ($2) => $3;".to_string(),
            conditions: vec!["language:javascript".to_string(), "language:typescript".to_string()],
            confidence: 0.9,
        });

        // Optional chaining conversion (JavaScript/TypeScript)
        self.transformation_rules.push(TransformationRule {
            from_pattern: r"(\w+)\s*&&\s*\1\.(\w+)".to_string(),
            to_pattern: "$1?.$2".to_string(),
            conditions: vec!["language:javascript".to_string(), "language:typescript".to_string()],
            confidence: 0.8,
        });

        // Nullish coalescing conversion (JavaScript/TypeScript)
        self.transformation_rules.push(TransformationRule {
            from_pattern: r"(\w+)\s*\|\|\s*([^|]+)".to_string(),
            to_pattern: "$1 ?? $2".to_string(),
            conditions: vec!["language:javascript".to_string(), "language:typescript".to_string(), "null_check".to_string()],
            confidence: 0.7,
        });

        // String template conversion (JavaScript/TypeScript)
        self.transformation_rules.push(TransformationRule {
            from_pattern: r#"(\w+)\s*\+\s*"([^"]*)"\s*\+\s*(\w+)"#.to_string(),
            to_pattern: r#"`$2${$3}`"#.to_string(),
            conditions: vec!["javascript".to_string(), "typescript".to_string()],
            confidence: 0.8,
        });
    }

    /// Analyze code for refactoring opportunities
    pub async fn analyze_for_refactoring(&self, code: &str, language: &str) -> Result<Vec<RefactoringSuggestion>> {
        let mut suggestions = Vec::new();

        // Analyze for code smells
        for (smell_name, smell) in &self.code_smells {
            if self.detect_code_smell(code, language, smell).await {
                suggestions.push(RefactoringSuggestion {
                    smell_type: smell_name.clone(),
                    description: smell.description.clone(),
                    severity: smell.severity.clone(),
                    suggested_refactoring: smell.refactoring_suggestion.clone(),
                    confidence: 0.7,
                    affected_lines: self.find_affected_lines(code, smell),
                    risk_assessment: self.assess_refactoring_risk(smell_name),
                });
            }
        }

        // Analyze for transformation opportunities
        for rule in &self.transformation_rules {
            if self.should_apply_rule(code, language, rule) {
                if let Some(suggestion) = self.create_transformation_suggestion(code, rule) {
                    suggestions.push(suggestion);
                }
            }
        }

        // Analyze for structural improvements
        let structural_suggestions = self.analyze_structural_improvements(code, language).await;
        suggestions.extend(structural_suggestions);

        Ok(suggestions)
    }

    /// Detect if a code smell is present
    async fn detect_code_smell(&self, code: &str, language: &str, smell: &CodeSmell) -> bool {
        match smell.name.as_str() {
            "long_method" => {
                // Simple heuristic: count lines in functions
                let lines: Vec<&str> = code.lines().collect();
                let mut in_function = false;
                let mut brace_count = 0;
                let mut function_lines = 0;

                for line in lines {
                    if line.contains("function ") || line.contains("fn ") || line.contains("def ") {
                        in_function = true;
                        brace_count = 0;
                        function_lines = 0;
                    }

                    if in_function {
                        function_lines += 1;
                        brace_count += line.chars().filter(|&c| c == '{').count() as i32;
                        brace_count -= line.chars().filter(|&c| c == '}').count() as i32;

                        if brace_count <= 0 && function_lines > 20 {
                            return true;
                        }
                    }
                }
                false
            }
            "long_parameter_list" => {
                // Count parameters in function definitions
                let param_pattern = match language {
                    "rust" => r"fn\s+\w+\s*\(([^)]*)\)",
                    "javascript" | "typescript" => r"function\s+\w+\s*\(([^)]*)\)",
                    "python" => r"def\s+\w+\s*\(([^)]*)\)",
                    _ => return false,
                };

                if let Ok(regex) = regex::Regex::new(param_pattern) {
                    for cap in regex.captures_iter(code) {
                        if let Some(params) = cap.get(1) {
                            let param_count = params.as_str().split(',').filter(|p| !p.trim().is_empty()).count();
                            if param_count > 4 {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            "large_class" => {
                // Simple heuristic: count lines in class
                let lines: Vec<&str> = code.lines().collect();
                let mut in_class = false;
                let mut brace_count = 0;
                let mut class_lines = 0;

                for line in lines {
                    if line.contains("class ") || line.contains("struct ") {
                        in_class = true;
                        brace_count = 0;
                        class_lines = 0;
                    }

                    if in_class {
                        class_lines += 1;
                        brace_count += line.chars().filter(|&c| c == '{').count() as i32;
                        brace_count -= line.chars().filter(|&c| c == '}').count() as i32;

                        if brace_count <= 0 && class_lines > 50 {
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Find lines affected by a code smell
    fn find_affected_lines(&self, code: &str, smell: &CodeSmell) -> Vec<u32> {
        // Simple implementation - would need more sophisticated analysis
        let lines: Vec<&str> = code.lines().collect();
        let mut affected = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if self.line_matches_smell(line, smell) {
                affected.push(i as u32 + 1);
            }
        }

        affected
    }

    /// Check if a line matches a code smell pattern
    fn line_matches_smell(&self, line: &str, smell: &CodeSmell) -> bool {
        match smell.name.as_str() {
            "long_method" => line.contains("function ") || line.contains("fn ") || line.contains("def "),
            "long_parameter_list" => {
                let param_count = line.matches(',').count();
                param_count > 3
            }
            "large_class" => line.contains("class ") || line.contains("struct "),
            _ => false,
        }
    }

    /// Assess risk level for a refactoring
    fn assess_refactoring_risk(&self, smell_name: &str) -> f32 {
        match smell_name {
            "long_method" | "long_parameter_list" => 0.2, // Low risk
            "large_class" | "duplicate_code" => 0.5, // Medium risk
            "feature_envy" => 0.7, // Higher risk
            _ => 0.3,
        }
    }

    /// Check if a transformation rule should be applied
    fn should_apply_rule(&self, code: &str, language: &str, rule: &TransformationRule) -> bool {
        // Check language compatibility
        if !rule.conditions.iter().any(|c| c == &format!("language:{}", language)) {
            return false;
        }

        // Check if pattern matches
        if let Ok(regex) = regex::Regex::new(&rule.from_pattern) {
            return regex.is_match(code);
        }

        false
    }

    /// Create transformation suggestion
    fn create_transformation_suggestion(&self, code: &str, rule: &TransformationRule) -> Option<RefactoringSuggestion> {
        if let Ok(regex) = regex::Regex::new(&rule.from_pattern) {
            if let Some(captures) = regex.captures(code) {
                let mut transformed = rule.to_pattern.clone();

                // Replace capture groups
                for (i, cap) in captures.iter().enumerate() {
                    if let Some(m) = cap {
                        let placeholder = format!("${}", i);
                        transformed = transformed.replace(&placeholder, m.as_str());
                    }
                }

                return Some(RefactoringSuggestion {
                    smell_type: "transformation_opportunity".to_string(),
                    description: format!("Can apply transformation: {}", rule.from_pattern),
                    severity: IssueSeverity::Info,
                    suggested_refactoring: format!("Transform to: {}", transformed),
                    confidence: rule.confidence,
                    affected_lines: vec![], // Would need line analysis
                    risk_assessment: 0.1, // Transformations are usually safe
                });
            }
        }

        None
    }

    /// Analyze for structural improvements
    async fn analyze_structural_improvements(&self, code: &str, language: &str) -> Vec<RefactoringSuggestion> {
        let mut suggestions = Vec::new();

        // Check for missing error handling
        if language == "rust" && code.contains("unwrap()") && !code.contains("expect(") {
            suggestions.push(RefactoringSuggestion {
                smell_type: "error_handling".to_string(),
                description: "Replace unwrap() with proper error handling".to_string(),
                severity: IssueSeverity::Warning,
                suggested_refactoring: "Use Result<T, E> and proper error propagation".to_string(),
                confidence: 0.8,
                affected_lines: vec![],
                risk_assessment: 0.3,
            });
        }

        // Check for magic numbers
        let magic_number_pattern = r"\b\d{2,}\b"; // Numbers with 2+ digits
        if let Ok(regex) = regex::Regex::new(magic_number_pattern) {
            let has_magic_numbers = regex.is_match(code);
            if has_magic_numbers {
                suggestions.push(RefactoringSuggestion {
                    smell_type: "magic_numbers".to_string(),
                    description: "Replace magic numbers with named constants".to_string(),
                    severity: IssueSeverity::Info,
                    suggested_refactoring: "Extract magic numbers to const declarations".to_string(),
                    confidence: 0.6,
                    affected_lines: vec![],
                    risk_assessment: 0.1,
                });
            }
        }

        // Check for deep nesting
        let nesting_level = self.calculate_max_nesting(code);
        if nesting_level > 4 {
            suggestions.push(RefactoringSuggestion {
                smell_type: "deep_nesting".to_string(),
                description: format!("Deep nesting detected (level {})", nesting_level),
                severity: IssueSeverity::Warning,
                suggested_refactoring: "Extract nested logic into separate functions or use early returns".to_string(),
                confidence: 0.7,
                affected_lines: vec![],
                risk_assessment: 0.4,
            });
        }

        suggestions
    }

    /// Calculate maximum nesting level in code
    fn calculate_max_nesting(&self, code: &str) -> usize {
        let mut max_nesting = 0;
        let mut current_nesting = 0;

        for line in code.lines() {
            let open_braces = line.chars().filter(|&c| c == '{').count();
            let close_braces = line.chars().filter(|&c| c == '}').count();

            current_nesting += open_braces;
            max_nesting = max_nesting.max(current_nesting);
            current_nesting = current_nesting.saturating_sub(close_braces);
        }

        max_nesting
    }

    /// Apply a refactoring suggestion
    pub async fn apply_refactoring(&self, code: &str, suggestion: &RefactoringSuggestion) -> Result<String> {
        match suggestion.smell_type.as_str() {
            "transformation_opportunity" => {
                // Apply transformation rule
                self.apply_transformation(code, suggestion)
            }
            "error_handling" => {
                self.apply_error_handling_refactoring(code)
            }
            "magic_numbers" => {
                self.apply_magic_number_refactoring(code)
            }
            _ => Ok(code.to_string()), // No automatic application for complex refactorings
        }
    }

    /// Apply transformation refactoring
    fn apply_transformation(&self, code: &str, _suggestion: &RefactoringSuggestion) -> Result<String> {
        // Find the appropriate transformation rule
        for rule in &self.transformation_rules {
            if let Ok(regex) = regex::Regex::new(&rule.from_pattern) {
                if regex.is_match(code) {
                    let transformed = regex.replace_all(code, rule.to_pattern.as_str());
                    return Ok(transformed.to_string());
                }
            }
        }

        Ok(code.to_string())
    }

    /// Apply error handling refactoring
    fn apply_error_handling_refactoring(&self, code: &str) -> Result<String> {
        // Simple transformation: replace unwrap() with expect()
        let transformed = code.replace("unwrap()", "expect(\"Operation failed\")");
        Ok(transformed)
    }

    /// Apply magic number refactoring
    fn apply_magic_number_refactoring(&self, code: &str) -> Result<String> {
        // This would require more sophisticated analysis to identify and extract magic numbers
        // For now, return unchanged
        Ok(code.to_string())
    }

    /// Get available refactoring patterns
    pub fn get_refactoring_patterns(&self) -> &HashMap<String, RefactoringPattern> {
        &self.refactoring_patterns
    }

    /// Get code smells
    pub fn get_code_smells(&self) -> &HashMap<String, CodeSmell> {
        &self.code_smells
    }

    /// Add custom refactoring pattern
    pub fn add_refactoring_pattern(&mut self, name: String, pattern: RefactoringPattern) {
        self.refactoring_patterns.insert(name, pattern);
    }

    /// Add custom code smell
    pub fn add_code_smell(&mut self, name: String, smell: CodeSmell) {
        self.code_smells.insert(name, smell);
    }
}

/// Refactoring suggestion structure
#[derive(Debug, Clone)]
pub struct RefactoringSuggestion {
    pub smell_type: String,
    pub description: String,
    pub severity: IssueSeverity,
    pub suggested_refactoring: String,
    pub confidence: f32,
    pub affected_lines: Vec<u32>,
    pub risk_assessment: f32,
}

impl SeniorEngineerKnowledge {
    pub fn new() -> Self {
        let mut knowledge = Self {
            architecture_patterns: HashMap::new(),
            testing_strategies: HashMap::new(),
            devops_practices: HashMap::new(),
            database_design: HashMap::new(),
            api_design: HashMap::new(),
            documentation_templates: HashMap::new(),
        };

        // Initialize with common patterns
        knowledge.initialize_architecture_patterns();
        knowledge.initialize_testing_strategies();
        knowledge.initialize_devops_practices();
        knowledge.initialize_database_patterns();
        knowledge.initialize_api_design();
        knowledge.initialize_documentation_templates();

        knowledge
    }

    fn initialize_architecture_patterns(&mut self) {
        // MVC Pattern
        self.architecture_patterns.insert("MVC".to_string(), ArchitecturePattern {
            name: "Model-View-Controller".to_string(),
            description: "Separates application logic into three interconnected components".to_string(),
            use_case: "Web applications, GUI applications".to_string(),
            components: vec!["Model".to_string(), "View".to_string(), "Controller".to_string()],
            benefits: vec![
                "Separation of concerns".to_string(),
                "Testability".to_string(),
                "Maintainability".to_string(),
            ],
            tradeoffs: vec![
                "Increased complexity".to_string(),
                "Learning curve".to_string(),
            ],
            example_code: "struct Controller {\n    model: Model,\n}\n\nimpl Controller {\n    fn handle_request(&self) -> View {\n        // Logic here\n    }\n}".to_string(),
        });

        // Microservices
        self.architecture_patterns.insert("Microservices".to_string(), ArchitecturePattern {
            name: "Microservices Architecture".to_string(),
            description: "Application composed of small, independent services".to_string(),
            use_case: "Large-scale applications, cloud-native development".to_string(),
            components: vec!["Service Registry".to_string(), "API Gateway".to_string(), "Database per Service".to_string()],
            benefits: vec![
                "Scalability".to_string(),
                "Technology diversity".to_string(),
                "Independent deployment".to_string(),
            ],
            tradeoffs: vec![
                "Increased complexity".to_string(),
                "Distributed system challenges".to_string(),
                "Operational overhead".to_string(),
            ],
            example_code: "// Service definition\n#[derive(Serialize, Deserialize)]\nstruct UserService {\n    users: HashMap<String, User>,\n}".to_string(),
        });
    }

    fn initialize_testing_strategies(&mut self) {
        self.testing_strategies.insert("Unit Testing".to_string(), TestingStrategy {
            strategy_type: TestingType::Unit,
            description: "Test individual units/components in isolation".to_string(),
            frameworks: vec!["JUnit".to_string(), "pytest".to_string(), "RSpec".to_string()],
            coverage_goals: 0.8,
            automation_level: AutomationLevel::FullyAutomated,
            ci_cd_integration: true,
        });

        self.testing_strategies.insert("Integration Testing".to_string(), TestingStrategy {
            strategy_type: TestingType::Integration,
            description: "Test interactions between components".to_string(),
            frameworks: vec!["TestNG".to_string(), "pytest".to_string(), "Cucumber".to_string()],
            coverage_goals: 0.6,
            automation_level: AutomationLevel::FullyAutomated,
            ci_cd_integration: true,
        });
    }

    fn initialize_devops_practices(&mut self) {
        self.devops_practices.insert("CI/CD Pipeline".to_string(), DevOpsPractice {
            practice: "Continuous Integration/Continuous Deployment".to_string(),
            description: "Automated testing and deployment pipeline".to_string(),
            tools: vec!["Jenkins".to_string(), "GitLab CI".to_string(), "GitHub Actions".to_string()],
            benefits: vec![
                "Faster delivery".to_string(),
                "Reduced manual errors".to_string(),
                "Consistent deployments".to_string(),
            ],
            implementation_steps: vec![
                "Set up version control".to_string(),
                "Configure automated tests".to_string(),
                "Create deployment scripts".to_string(),
                "Set up monitoring".to_string(),
            ],
        });
    }

    fn initialize_database_patterns(&mut self) {
        self.database_design.insert("CQRS".to_string(), DatabasePattern {
            pattern: "Command Query Responsibility Segregation".to_string(),
            description: "Separate read and write operations".to_string(),
            use_cases: vec!["High-performance applications".to_string(), "Complex domain models".to_string()],
            sql_example: "-- Write model\nINSERT INTO users (id, name) VALUES (?, ?);\n\n-- Read model\nSELECT * FROM user_view WHERE id = ?;".to_string(),
            considerations: vec![
                "Increased complexity".to_string(),
                "Eventual consistency".to_string(),
                "Additional infrastructure".to_string(),
            ],
        });
    }

    fn initialize_api_design(&mut self) {
        self.api_design.insert("REST Resource".to_string(), ApiDesignPattern {
            pattern: "RESTful Resource Design".to_string(),
            description: "Design APIs using REST principles".to_string(),
            http_method: "GET, POST, PUT, DELETE".to_string(),
            example: "/api/users/{id}".to_string(),
            best_practices: vec![
                "Use nouns for resources".to_string(),
                "Use HTTP methods appropriately".to_string(),
                "Provide meaningful status codes".to_string(),
                "Version your APIs".to_string(),
            ],
        });
    }

    fn initialize_documentation_templates(&mut self) {
        self.documentation_templates.insert("API Documentation".to_string(), DocumentationTemplate {
            doc_type: "API Documentation".to_string(),
            template: "# API Documentation\n\n## Overview\n{description}\n\n## Endpoints\n\n### {method} {path}\n{description}\n\n**Parameters:**\n{parameters}\n\n**Response:**\n{response}\n\n## Examples\n{examples}".to_string(),
            sections: vec!["Overview".to_string(), "Endpoints".to_string(), "Examples".to_string()],
            examples: vec!["GET /api/users - Retrieve all users".to_string()],
        });
    }
}

impl TerminalIntelligence {
    pub fn new() -> Self {
        let mut intelligence = Self {
            command_patterns: HashMap::new(),
            script_templates: HashMap::new(),
            automation_workflows: HashMap::new(),
            package_managers: HashMap::new(),
            system_commands: HashMap::new(),
        };

        intelligence.initialize_command_patterns();
        intelligence.initialize_script_templates();
        intelligence.initialize_automation_workflows();
        intelligence.initialize_package_managers();
        intelligence.initialize_system_commands();

        intelligence
    }

    fn initialize_command_patterns(&mut self) {
        // File operations
        self.command_patterns.insert("find".to_string(), CommandPattern {
            command: "find".to_string(),
            description: "Search for files in a directory hierarchy".to_string(),
            category: CommandCategory::FileSystem,
            arguments: vec![
                CommandArgument {
                    name: "path".to_string(),
                    description: "Starting directory path".to_string(),
                    required: true,
                    default_value: Some(".".to_string()),
                },
                CommandArgument {
                    name: "name".to_string(),
                    description: "File name pattern".to_string(),
                    required: false,
                    default_value: None,
                },
            ],
            examples: vec![
                "find . -name '*.rs' -type f".to_string(),
                "find /home -name 'config*' -type f".to_string(),
            ],
            common_flags: vec!["-name".to_string(), "-type".to_string(), "-exec".to_string()],
        });

        // Process management
        self.command_patterns.insert("ps".to_string(), CommandPattern {
            command: "ps".to_string(),
            description: "Report process status".to_string(),
            category: CommandCategory::ProcessManagement,
            arguments: vec![],
            examples: vec![
                "ps aux".to_string(),
                "ps -ef | grep process_name".to_string(),
            ],
            common_flags: vec!["aux".to_string(), "-ef".to_string(), "-p".to_string()],
        });

        // Network diagnostics
        self.command_patterns.insert("curl".to_string(), CommandPattern {
            command: "curl".to_string(),
            description: "Transfer data from or to a server".to_string(),
            category: CommandCategory::Network,
            arguments: vec![
                CommandArgument {
                    name: "url".to_string(),
                    description: "URL to fetch".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            examples: vec![
                "curl -X GET https://api.example.com/users".to_string(),
                "curl -X POST -H 'Content-Type: application/json' -d '{\"key\":\"value\"}' https://api.example.com/data".to_string(),
            ],
            common_flags: vec!["-X".to_string(), "-H".to_string(), "-d".to_string(), "-v".to_string()],
        });
    }

    fn initialize_script_templates(&mut self) {
        self.script_templates.insert("backup_script".to_string(), ScriptTemplate {
            name: "Database Backup Script".to_string(),
            description: "Automated database backup with rotation".to_string(),
            language: "bash".to_string(),
            template: "#!/bin/bash\n\n# Database Backup Script\nBACKUP_DIR=\"${BACKUP_DIR:-/var/backups}\"\nDB_NAME=\"${DB_NAME}\"\nRETENTION_DAYS=\"${RETENTION_DAYS:-7}\"\n\n# Create backup directory\nmkdir -p \"$BACKUP_DIR\"\n\n# Generate timestamp\nTIMESTAMP=$(date +%Y%m%d_%H%M%S)\nBACKUP_FILE=\"$BACKUP_DIR/${DB_NAME}_$TIMESTAMP.sql\"\n\n# Create backup\necho \"Creating backup: $BACKUP_FILE\"\n{backup_command} > \"$BACKUP_FILE\"\n\n# Rotate old backups\nfind \"$BACKUP_DIR\" -name \"${DB_NAME}_*.sql\" -mtime +$RETENTION_DAYS -delete\n\necho \"Backup completed successfully\"".to_string(),
            variables: vec!["BACKUP_DIR".to_string(), "DB_NAME".to_string(), "RETENTION_DAYS".to_string()],
            use_cases: vec!["Regular database backups".to_string(), "Disaster recovery".to_string()],
        });

        self.script_templates.insert("deploy_script".to_string(), ScriptTemplate {
            name: "Application Deployment Script".to_string(),
            description: "Zero-downtime application deployment".to_string(),
            language: "bash".to_string(),
            template: "#!/bin/bash\n\n# Application Deployment Script\nAPP_NAME=\"${APP_NAME}\"\nAPP_DIR=\"${APP_DIR:-/opt/$APP_NAME}\"\nBACKUP_DIR=\"${BACKUP_DIR:-/opt/backups}\"\n\n# Pre-deployment checks\necho \"Running pre-deployment checks...\"\n{health_check_command}\n\n# Create backup\nBACKUP_FILE=\"$BACKUP_DIR/${APP_NAME}_$(date +%Y%m%d_%H%M%S).tar.gz\"\necho \"Creating backup: $BACKUP_FILE\"\ntar -czf \"$BACKUP_FILE\" -C \"$APP_DIR\" .\n\n# Deploy new version\necho \"Deploying new version...\"\n{service_stop_command}\n{deployment_command}\n{service_start_command}\n\n# Post-deployment verification\necho \"Running post-deployment checks...\"\n{verification_command}\n\necho \"Deployment completed successfully\"".to_string(),
            variables: vec!["APP_NAME".to_string(), "APP_DIR".to_string(), "BACKUP_DIR".to_string()],
            use_cases: vec!["Application deployments".to_string(), "Rolling updates".to_string()],
        });
    }

    fn initialize_automation_workflows(&mut self) {
        self.automation_workflows.insert("ci_pipeline".to_string(), AutomationWorkflow {
            name: "CI/CD Pipeline".to_string(),
            description: "Complete CI/CD pipeline with testing and deployment".to_string(),
            steps: vec![
                WorkflowStep {
                    command: "git fetch origin && git checkout $BRANCH".to_string(),
                    description: "Fetch and checkout target branch".to_string(),
                    timeout_seconds: Some(30),
                    continue_on_error: false,
                },
                WorkflowStep {
                    command: "cargo check".to_string(),
                    description: "Run code compilation check".to_string(),
                    timeout_seconds: Some(300),
                    continue_on_error: false,
                },
                WorkflowStep {
                    command: "cargo test".to_string(),
                    description: "Run unit tests".to_string(),
                    timeout_seconds: Some(600),
                    continue_on_error: false,
                },
                WorkflowStep {
                    command: "cargo build --release".to_string(),
                    description: "Build release binary".to_string(),
                    timeout_seconds: Some(600),
                    continue_on_error: false,
                },
                WorkflowStep {
                    command: "./deploy.sh".to_string(),
                    description: "Deploy application".to_string(),
                    timeout_seconds: Some(300),
                    continue_on_error: true,
                },
            ],
            triggers: vec!["push".to_string(), "pull_request".to_string()],
            error_handling: ErrorHandlingStrategy::StopOnError,
        });
    }

    fn initialize_package_managers(&mut self) {
        self.package_managers.insert("cargo".to_string(), PackageManager {
            name: "Cargo".to_string(),
            description: "Rust package manager".to_string(),
            commands: [
                ("install".to_string(), "cargo install <package>".to_string()),
                ("build".to_string(), "cargo build".to_string()),
                ("test".to_string(), "cargo test".to_string()),
                ("update".to_string(), "cargo update".to_string()),
            ].into_iter().collect(),
            ecosystems: vec!["Rust".to_string()],
        });

        self.package_managers.insert("npm".to_string(), PackageManager {
            name: "npm".to_string(),
            description: "Node.js package manager".to_string(),
            commands: [
                ("install".to_string(), "npm install <package>".to_string()),
                ("run".to_string(), "npm run <script>".to_string()),
                ("test".to_string(), "npm test".to_string()),
                ("update".to_string(), "npm update".to_string()),
            ].into_iter().collect(),
            ecosystems: vec!["JavaScript".to_string(), "TypeScript".to_string()],
        });

        self.package_managers.insert("pip".to_string(), PackageManager {
            name: "pip".to_string(),
            description: "Python package manager".to_string(),
            commands: [
                ("install".to_string(), "pip install <package>".to_string()),
                ("freeze".to_string(), "pip freeze".to_string()),
                ("uninstall".to_string(), "pip uninstall <package>".to_string()),
            ].into_iter().collect(),
            ecosystems: vec!["Python".to_string()],
        });
    }

    fn initialize_system_commands(&mut self) {
        self.system_commands.insert("top".to_string(), SystemCommand {
            name: "top".to_string(),
            description: "Display system processes and resource usage".to_string(),
            category: SystemCategory::Monitoring,
            syntax: "top [options]".to_string(),
            examples: vec![
                "top".to_string(),
                "top -u username".to_string(),
                "top -p PID".to_string(),
            ],
            flags: vec!["-u".to_string(), "-p".to_string(), "-d".to_string()],
        });

        self.system_commands.insert("netstat".to_string(), SystemCommand {
            name: "netstat".to_string(),
            description: "Display network connections and statistics".to_string(),
            category: SystemCategory::Network,
            syntax: "netstat [options]".to_string(),
            examples: vec![
                "netstat -tuln".to_string(),
                "netstat -r".to_string(),
                "netstat -i".to_string(),
            ],
            flags: vec!["-t".to_string(), "-u".to_string(), "-l".to_string(), "-n".to_string()],
        });

        self.system_commands.insert("df".to_string(), SystemCommand {
            name: "df".to_string(),
            description: "Display disk space usage".to_string(),
            category: SystemCategory::FileSystem,
            syntax: "df [options] [file]".to_string(),
            examples: vec![
                "df -h".to_string(),
                "df -i".to_string(),
                "df /home".to_string(),
            ],
            flags: vec!["-h".to_string(), "-i".to_string(), "-T".to_string()],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_engine_creation() {
        let config = AiConfig::default();
        let engine = AiEngine::new(config);
        
        let retrieved_config = engine.get_config().await
            .expect("Failed to retrieve config in test");
        assert!(retrieved_config.enabled);
        assert_eq!(retrieved_config.provider, "mock");
    }

    #[tokio::test]
    async fn test_code_completion() {
        let engine = AiEngine::new(AiConfig::default());
        
        let request = CompletionRequest {
            prompt: "Complete this function".to_string(),
            context: "fn main()".to_string(),
            language: "rust".to_string(),
            max_tokens: Some(100),
        };

        let response = engine.complete_code(request).await
            .expect("Failed to complete code in test");
        assert!(!response.text.is_empty());
        assert!(response.confidence >= 0.0 && response.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_code_analysis() {
        let engine = AiEngine::new(AiConfig::default());
        
        let code = "fn main() {\n    let x = Some(42);\n    println!(\"{}\", x.unwrap());\n}";
        let result = engine.analyze_code(code, "rust").await
            .expect("Failed to analyze code in test");
        
        assert!(!result.issues.is_empty() || !result.suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_learning_feedback() {
        let engine = AiEngine::new(AiConfig::default());
        
        engine.learn_from_feedback("pattern1".to_string(), true).await
            .expect("Failed to learn positive feedback in test");
        engine.learn_from_feedback("pattern1".to_string(), false).await
            .expect("Failed to learn negative feedback in test");
        
        // Verify learning data was updated
        let learning_data = engine.learning_data.read().await;
        assert!(learning_data.contains_key("pattern1"));
    }
}
