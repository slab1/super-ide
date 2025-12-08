//! Browser automation integration
//!
//! This module provides Rust bindings for the Python browser automation system.

use super::{ExternalConfig, ExternalError, ExternalResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use log::debug;

/// Browser automation request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserNavigateRequest {
    pub url: String,
    pub wait_for_load: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserScreenshotRequest {
    pub full_page: Option<bool>,
    pub selector: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserExecuteScriptRequest {
    pub script: String,
    pub args: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserClickRequest {
    pub selector: String,
    pub button: Option<String>, // "left", "right", "middle"
    pub modifiers: Option<Vec<String>>, // ["Shift", "Control", "Alt", "Meta"]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTypeRequest {
    pub selector: Option<String>,
    pub text: String,
    pub delay: Option<u32>, // milliseconds between keystrokes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserWaitRequest {
    pub selector: String,
    pub timeout: Option<u32>,
    pub state: Option<String>, // "visible", "hidden", "attached", "detached"
}

/// Browser automation response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserPageInfo {
    pub url: String,
    pub title: String,
    pub loading: bool,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserElementInfo {
    pub tag_name: String,
    pub text_content: Option<String>,
    pub attributes: HashMap<String, String>,
    pub bounding_box: Option<ElementBoundingBox>,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementBoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Browser automation client
pub struct BrowserClient {
    client: Client,
    _config: ExternalConfig,
    debug_url: String,
}

impl BrowserClient {
    /// Create a new browser client
    pub fn new(config: ExternalConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout))
            .build()
            .expect("Failed to create HTTP client");

        let debug_url = format!("http://localhost:{}", config.browser_debug_port);

        Self {
            client,
            _config: config,
            debug_url,
        }
    }

    /// Navigate to a URL
    pub async fn navigate(&self, request: BrowserNavigateRequest) -> ExternalResult<BrowserPageInfo> {
        let mut params = HashMap::new();
        params.insert("url".to_string(), serde_json::Value::String(request.url.clone()));
        params.insert("action".to_string(), serde_json::Value::String("navigate".to_string()));

        if let Some(wait_for_load) = request.wait_for_load {
            params.insert("wait_for_load".to_string(), serde_json::Value::Bool(wait_for_load));
        }

        let response = self.call_browser_action("navigate", params).await?;

        if response.success {
            if let Some(data) = response.data {
                let page_info: BrowserPageInfo = serde_json::from_value(data)
                    .map_err(|e| ExternalError::JsonError(e.to_string()))?;
                Ok(page_info)
            } else {
                Err(ExternalError::BrowserError("No page info in response".to_string()))
            }
        } else {
            Err(ExternalError::BrowserError(response.message))
        }
    }

    /// Take a screenshot
    pub async fn screenshot(&self, request: BrowserScreenshotRequest) -> ExternalResult<Vec<u8>> {
        let mut params = HashMap::new();
        params.insert("action".to_string(), serde_json::Value::String("screenshot".to_string()));

        if let Some(full_page) = request.full_page {
            params.insert("full_page".to_string(), serde_json::Value::Bool(full_page));
        }

        if let Some(selector) = &request.selector {
            params.insert("selector".to_string(), serde_json::Value::String(selector.clone()));
        }

        let response = self.call_browser_action("screenshot", params).await?;

        if response.success {
            if let Some(data) = response.data {
                if let Some(base64_str) = data.as_str() {
                    // Assume the response contains base64 encoded image data
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_str)
                        .map_err(|e| ExternalError::JsonError(format!("Invalid base64: {}", e)))
                } else {
                    Err(ExternalError::BrowserError("Screenshot data is not a string".to_string()))
                }
            } else {
                Err(ExternalError::BrowserError("No screenshot data in response".to_string()))
            }
        } else {
            Err(ExternalError::BrowserError(response.message))
        }
    }

    /// Execute JavaScript
    pub async fn execute_script(&self, request: BrowserExecuteScriptRequest) -> ExternalResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("script".to_string(), serde_json::Value::String(request.script));
        params.insert("action".to_string(), serde_json::Value::String("execute_script".to_string()));

        if let Some(args) = request.args {
            params.insert("args".to_string(), serde_json::Value::Array(args));
        }

        let response = self.call_browser_action("execute_script", params).await?;

        if response.success {
            response.data.ok_or_else(|| ExternalError::BrowserError("No script result in response".to_string()))
        } else {
            Err(ExternalError::BrowserError(response.message))
        }
    }

    /// Click on an element
    pub async fn click(&self, request: BrowserClickRequest) -> ExternalResult<()> {
        let mut params = HashMap::new();
        params.insert("selector".to_string(), serde_json::Value::String(request.selector));
        params.insert("action".to_string(), serde_json::Value::String("click".to_string()));

        if let Some(button) = request.button {
            params.insert("button".to_string(), serde_json::Value::String(button));
        }

        if let Some(modifiers) = request.modifiers {
            params.insert("modifiers".to_string(), serde_json::Value::Array(
                modifiers.into_iter().map(serde_json::Value::String).collect()
            ));
        }

        let response = self.call_browser_action("click", params).await?;

        if response.success {
            Ok(())
        } else {
            Err(ExternalError::BrowserError(response.message))
        }
    }

    /// Type text
    pub async fn type_text(&self, request: BrowserTypeRequest) -> ExternalResult<()> {
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::Value::String(request.text));
        params.insert("action".to_string(), serde_json::Value::String("type".to_string()));

        if let Some(selector) = request.selector {
            params.insert("selector".to_string(), serde_json::Value::String(selector));
        }

        if let Some(delay) = request.delay {
            params.insert("delay".to_string(), serde_json::Value::Number(delay.into()));
        }

        let response = self.call_browser_action("type", params).await?;

        if response.success {
            Ok(())
        } else {
            Err(ExternalError::BrowserError(response.message))
        }
    }

    /// Wait for element
    pub async fn wait_for_element(&self, request: BrowserWaitRequest) -> ExternalResult<BrowserElementInfo> {
        let mut params = HashMap::new();
        params.insert("selector".to_string(), serde_json::Value::String(request.selector));
        params.insert("action".to_string(), serde_json::Value::String("wait_for_element".to_string()));

        if let Some(timeout) = request.timeout {
            params.insert("timeout".to_string(), serde_json::Value::Number(timeout.into()));
        }

        if let Some(state) = request.state {
            params.insert("state".to_string(), serde_json::Value::String(state));
        }

        let response = self.call_browser_action("wait_for_element", params).await?;

        if response.success {
            if let Some(data) = response.data {
                let element_info: BrowserElementInfo = serde_json::from_value(data)
                    .map_err(|e| ExternalError::JsonError(e.to_string()))?;
                Ok(element_info)
            } else {
                Err(ExternalError::BrowserError("No element info in response".to_string()))
            }
        } else {
            Err(ExternalError::BrowserError(response.message))
        }
    }

    /// Get page information
    pub async fn get_page_info(&self) -> ExternalResult<BrowserPageInfo> {
        let mut params = HashMap::new();
        params.insert("action".to_string(), serde_json::Value::String("get_page_info".to_string()));

        let response = self.call_browser_action("get_page_info", params).await?;

        if response.success {
            if let Some(data) = response.data {
                let page_info: BrowserPageInfo = serde_json::from_value(data)
                    .map_err(|e| ExternalError::JsonError(e.to_string()))?;
                Ok(page_info)
            } else {
                Err(ExternalError::BrowserError("No page info in response".to_string()))
            }
        } else {
            Err(ExternalError::BrowserError(response.message))
        }
    }

    /// Get element information
    pub async fn get_element_info(&self, selector: &str) -> ExternalResult<BrowserElementInfo> {
        let mut params = HashMap::new();
        params.insert("selector".to_string(), serde_json::Value::String(selector.to_string()));
        params.insert("action".to_string(), serde_json::Value::String("get_element_info".to_string()));

        let response = self.call_browser_action("get_element_info", params).await?;

        if response.success {
            if let Some(data) = response.data {
                let element_info: BrowserElementInfo = serde_json::from_value(data)
                    .map_err(|e| ExternalError::JsonError(e.to_string()))?;
                Ok(element_info)
            } else {
                Err(ExternalError::BrowserError("No element info in response".to_string()))
            }
        } else {
            Err(ExternalError::BrowserError(response.message))
        }
    }

    /// Generic browser action call
    async fn call_browser_action(
        &self,
        action: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> ExternalResult<BrowserResponse> {
        // For browser automation, we'll use Chrome DevTools Protocol over WebSocket
        // or a REST API if the Python server exposes one

        let url = format!("{}/execute", self.debug_url);

        let request_body = serde_json::json!({
            "action": action,
            "parameters": parameters,
            "request_id": uuid::Uuid::new_v4().to_string(),
        });

        debug!("Calling browser action: {} with params: {:?}", action, parameters);

        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ExternalError::HttpError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExternalError::BrowserError(format!("HTTP {}: {}", status, error_text)));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExternalError::JsonError(e.to_string()))?;

        let success = response_json
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let message = response_json
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message")
            .to_string();

        let data = response_json.get("data").cloned();

        Ok(BrowserResponse {
            success,
            message,
            data,
        })
    }

    /// Check if browser is available
    pub async fn is_browser_available(&self) -> bool {
        // Try to connect to the browser debug endpoint
        match self.client
            .get(&format!("{}/health", self.debug_url))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}
