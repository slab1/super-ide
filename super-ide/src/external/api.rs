//! MCP API integration
//!
//! This module provides Rust bindings for the Python MCP (Model Context Protocol) system.

use super::{ExternalConfig, ExternalError, ExternalResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use log::{info, debug, error};

/// MCP function call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpFunctionCall {
    pub function_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// MCP function call response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpFunctionResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// MCP function description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpFunctionDescription {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub return_type: String,
}

/// Available data sources
#[derive(Debug, Clone, Copy)]
pub enum DataSource {
    Twitter,
    Booking,
    Commodities,
    Metal,
    Patents,
    Pinterest,
    Scholar,
    TripAdvisor,
    Yahoo,
}

impl DataSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataSource::Twitter => "twitter",
            DataSource::Booking => "booking",
            DataSource::Commodities => "commodities",
            DataSource::Metal => "metal",
            DataSource::Patents => "patents",
            DataSource::Pinterest => "pinterest",
            DataSource::Scholar => "scholar",
            DataSource::TripAdvisor => "tripadvisor",
            DataSource::Yahoo => "yahoo",
        }
    }
}

/// Twitter-specific request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterSearchRequest {
    pub query: String,
    pub limit: Option<i32>,
    pub lang: Option<String>,
    pub min_retweets: Option<i32>,
    pub min_likes: Option<i32>,
    pub min_replies: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterUserRequest {
    pub username: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterUserTweetsRequest {
    pub username: Option<String>,
    pub user_id: Option<String>,
    pub limit: Option<i32>,
    pub include_replies: Option<bool>,
    pub include_pinned: Option<bool>,
}

/// MCP API client
pub struct McpApiClient {
    client: Client,
    config: ExternalConfig,
    base_url: String,
}

impl McpApiClient {
    /// Create a new MCP API client
    pub fn new(config: ExternalConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout))
            .build()
            .expect("Failed to create HTTP client");

        let base_url = format!("http://localhost:{}", config.mcp_server_port);

        Self {
            client,
            config,
            base_url,
        }
    }

    /// Call an MCP function
    pub async fn call_function(
        &self,
        function_call: McpFunctionCall,
    ) -> ExternalResult<McpFunctionResponse> {
        let url = format!("{}/execute", self.base_url);

        let request_body = serde_json::json!({
            "request_id": uuid::Uuid::new_v4().to_string(),
            "function_name": function_call.function_name,
            "function_kind": "mcp",
            "caller_name": "super-ide",
            "parameters": function_call.parameters,
        });

        debug!("Calling MCP function: {} with params: {:?}", function_call.function_name, function_call.parameters);

        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ExternalError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExternalError::McpError(format!("HTTP {}: {}", response.status(), error_text)));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExternalError::JsonError(e.to_string()))?;

        let success = response_json
            .get("is_error")
            .and_then(|v| v.as_bool())
            .map(|v| !v)
            .unwrap_or(false);

        let message = response_json
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message")
            .to_string();

        let data = response_json.get("data").cloned();

        Ok(McpFunctionResponse {
            success,
            message,
            data,
        })
    }

    /// Search Twitter tweets
    pub async fn search_tweets(
        &self,
        request: TwitterSearchRequest,
    ) -> ExternalResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), serde_json::Value::String(request.query));
        params.insert("section".to_string(), serde_json::Value::String("top".to_string()));

        if let Some(limit) = request.limit {
            params.insert("limit".to_string(), serde_json::Value::Number(limit.into()));
        }

        if let Some(lang) = request.lang {
            params.insert("lang".to_string(), serde_json::Value::String(lang));
        }

        if let Some(min_retweets) = request.min_retweets {
            params.insert("min_retweets".to_string(), serde_json::Value::Number(min_retweets.into()));
        }

        if let Some(min_likes) = request.min_likes {
            params.insert("min_likes".to_string(), serde_json::Value::Number(min_likes.into()));
        }

        if let Some(min_replies) = request.min_replies {
            params.insert("min_replies".to_string(), serde_json::Value::Number(min_replies.into()));
        }

        if let Some(start_date) = request.start_date {
            params.insert("start_date".to_string(), serde_json::Value::String(start_date));
        }

        if let Some(end_date) = request.end_date {
            params.insert("end_date".to_string(), serde_json::Value::String(end_date));
        }

        let function_call = McpFunctionCall {
            function_name: "search_tweets".to_string(),
            parameters: params,
        };

        let response = self.call_function(function_call).await?;

        if response.success {
            response.data.ok_or_else(|| ExternalError::McpError("No data in response".to_string()))
        } else {
            Err(ExternalError::McpError(response.message))
        }
    }

    /// Get Twitter user information
    pub async fn get_twitter_user_info(
        &self,
        request: TwitterUserRequest,
    ) -> ExternalResult<serde_json::Value> {
        let mut params = HashMap::new();

        if let Some(username) = request.username {
            params.insert("username".to_string(), serde_json::Value::String(username));
        }

        if let Some(user_id) = request.user_id {
            params.insert("user_id".to_string(), serde_json::Value::String(user_id));
        }

        let function_call = McpFunctionCall {
            function_name: "get_user_info".to_string(),
            parameters: params,
        };

        let response = self.call_function(function_call).await?;

        if response.success {
            response.data.ok_or_else(|| ExternalError::McpError("No data in response".to_string()))
        } else {
            Err(ExternalError::McpError(response.message))
        }
    }

    /// Get Twitter user tweets
    pub async fn get_twitter_user_tweets(
        &self,
        request: TwitterUserTweetsRequest,
    ) -> ExternalResult<serde_json::Value> {
        let mut params = HashMap::new();

        if let Some(username) = request.username {
            params.insert("username".to_string(), serde_json::Value::String(username));
        }

        if let Some(user_id) = request.user_id {
            params.insert("user_id".to_string(), serde_json::Value::String(user_id));
        }

        if let Some(limit) = request.limit {
            params.insert("limit".to_string(), serde_json::Value::Number(limit.into()));
        }

        if let Some(include_replies) = request.include_replies {
            params.insert("include_replies".to_string(), serde_json::Value::Bool(include_replies));
        }

        if let Some(include_pinned) = request.include_pinned {
            params.insert("include_pinned".to_string(), serde_json::Value::Bool(include_pinned));
        }

        let function_call = McpFunctionCall {
            function_name: "get_user_tweets".to_string(),
            parameters: params,
        };

        let response = self.call_function(function_call).await?;

        if response.success {
            response.data.ok_or_else(|| ExternalError::McpError("No data in response".to_string()))
        } else {
            Err(ExternalError::McpError(response.message))
        }
    }

    /// Generic function call for any data source
    pub async fn call_data_source_function(
        &self,
        data_source: DataSource,
        function_name: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> ExternalResult<serde_json::Value> {
        let full_function_name = format!("{}_{}", data_source.as_str(), function_name);

        let function_call = McpFunctionCall {
            function_name: full_function_name,
            parameters,
        };

        let response = self.call_function(function_call).await?;

        if response.success {
            response.data.ok_or_else(|| ExternalError::McpError("No data in response".to_string()))
        } else {
            Err(ExternalError::McpError(response.message))
        }
    }

    /// Get available functions from MCP server
    pub async fn get_available_functions(&self) -> ExternalResult<Vec<McpFunctionDescription>> {
        // This would need to be implemented in the Python MCP server
        // For now, return a static list based on what we know
        let functions = vec![
            McpFunctionDescription {
                name: "search_tweets".to_string(),
                description: "Search for tweets on Twitter".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("query".to_string(), serde_json::json!("string"));
                    params.insert("limit".to_string(), serde_json::json!("number"));
                    params
                },
                return_type: "object".to_string(),
            },
            McpFunctionDescription {
                name: "get_user_info".to_string(),
                description: "Get Twitter user information".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("username".to_string(), serde_json::json!("string"));
                    params
                },
                return_type: "object".to_string(),
            },
            McpFunctionDescription {
                name: "get_user_tweets".to_string(),
                description: "Get tweets from a Twitter user".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("username".to_string(), serde_json::json!("string"));
                    params.insert("limit".to_string(), serde_json::json!("number"));
                    params
                },
                return_type: "object".to_string(),
            },
        ];

        Ok(functions)
    }
}
