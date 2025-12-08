# Super IDE External Integrations

This document describes the integration of external Python-based components into the Super IDE Rust application.

## Overview

The Super IDE now includes integration with two major external components:

1. **MCP (Model Context Protocol) API** - A system for accessing various external APIs and data sources
2. **Browser Automation** - Playwright-based browser automation with error capture capabilities

## Architecture

### External Module Structure

```
super-ide/src/external/
├── mod.rs           # Main external module with manager
├── api.rs           # MCP API client and data source integrations
└── browser.rs       # Browser automation client
```

### Dependencies Added

- `base64` - For encoding/decoding browser screenshots
- Existing: `reqwest`, `serde`, `uuid`, `tokio`

## MCP API Integration

### Available Data Sources

The MCP system provides access to various data sources through a unified HTTP API:

- **Twitter** - Tweet search, user info, user tweets
- **Booking.com** - Travel booking data
- **Commodities** - Market data
- **Metal** - Metal prices
- **Patents** - Patent search
- **Pinterest** - Image search
- **Scholar** - Academic papers
- **TripAdvisor** - Travel reviews
- **Yahoo** - Financial data

### Usage Example

```rust
use super_ide::external::{ExternalConfig, api::McpApiClient};

let config = ExternalConfig::default();
let client = McpApiClient::new(config);

// Search Twitter
let search_request = super_ide::external::api::TwitterSearchRequest {
    query: "Rust programming".to_string(),
    limit: Some(10),
    lang: Some("en".to_string()),
    ..Default::default()
};

match client.search_tweets(search_request).await {
    Ok(tweets) => println!("Found tweets: {:?}", tweets),
    Err(e) => eprintln!("Error: {}", e),
}
```

### API Endpoints

The following REST API endpoints are available in Super IDE:

- `POST /api/external/mcp/search_tweets` - Search Twitter tweets
- `POST /api/external/mcp/user_info` - Get Twitter user information
- `POST /api/external/mcp/user_tweets` - Get user tweets
- `GET /api/external/mcp/functions` - List available functions

## Browser Automation Integration

### Features

- Navigate to URLs
- Take screenshots (full page or element-specific)
- Execute JavaScript
- Click elements
- Type text
- Wait for elements
- Get page/element information

### Browser Extension

The browser automation includes a Chrome extension (`browser/browser_extension/error_capture/`) that:

- Captures JavaScript errors
- Logs them to `window.__matrix_errors__`
- Provides debugging capabilities

### Usage Example

```rust
use super_ide::external::{ExternalConfig, browser::BrowserClient};

let config = ExternalConfig::default();
let client = BrowserClient::new(config);

// Navigate to a page
let navigate_request = super_ide::external::browser::BrowserNavigateRequest {
    url: "https://www.rust-lang.org/".to_string(),
    wait_for_load: Some(true),
};

match client.navigate(navigate_request).await {
    Ok(page_info) => println!("Page loaded: {}", page_info.title),
    Err(e) => eprintln!("Navigation failed: {}", e),
}

// Take a screenshot
let screenshot_request = super_ide::external::browser::BrowserScreenshotRequest {
    full_page: Some(true),
    selector: None,
};

match client.screenshot(screenshot_request).await {
    Ok(image_data) => println!("Screenshot taken: {} bytes", image_data.len()),
    Err(e) => eprintln!("Screenshot failed: {}", e),
}
```

### API Endpoints

- `POST /api/external/browser/navigate` - Navigate to URL
- `POST /api/external/browser/screenshot` - Take screenshot
- `POST /api/external/browser/execute_script` - Execute JavaScript
- `POST /api/external/browser/click` - Click element
- `POST /api/external/browser/type` - Type text
- `POST /api/external/browser/wait` - Wait for element
- `GET /api/external/browser/page_info` - Get page information
- `POST /api/external/browser/element_info` - Get element information

## Configuration

### ExternalConfig Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalConfig {
    pub python_path: String,           // Path to Python executable
    pub external_api_path: String,     // Path to external_api directory
    pub browser_path: String,          // Path to browser directory
    pub mcp_server_port: u16,         // MCP server port (default: 12306)
    pub browser_debug_port: u16,       // Browser debug port (default: 9222)
    pub request_timeout: u64,          // Request timeout in seconds
}
```

### Default Configuration

```rust
let config = ExternalConfig {
    python_path: "python".to_string(),
    external_api_path: "./external_api".to_string(),
    browser_path: "./browser".to_string(),
    mcp_server_port: 12306,
    browser_debug_port: 9222,
    request_timeout: 30,
};
```

## Service Management

### ExternalManager

The `ExternalManager` handles starting and stopping external services:

```rust
let manager = ExternalManager::new(config);

// Start services
manager.start_mcp_server().await?;
manager.start_browser().await?;

// Check status
let mcp_running = manager.is_mcp_server_running().await;
let browser_running = manager.is_browser_running().await;

// Stop services
manager.stop_mcp_server().await?;
manager.stop_browser().await?;
```

## Setup Requirements

### Python Dependencies

The external components require Python with the following packages:

```bash
pip install aiohttp pydantic playwright
```

### Playwright Browsers

Install Playwright browsers:

```bash
playwright install
```

### Starting Services

Before using the integrations, start the required services:

1. **MCP Server**: Runs on port 12306
2. **Browser Automation**: Runs on port 9222 with Chrome DevTools Protocol

## Error Handling

All external operations return `ExternalResult<T>` which is `Result<T, ExternalError>`.

### Error Types

- `PythonError` - Python execution failures
- `HttpError` - HTTP request failures
- `JsonError` - JSON parsing errors
- `BrowserError` - Browser operation failures
- `McpError` - MCP operation failures
- `IoError` - I/O errors
- `ProcessError` - Process spawn/kill failures

## Testing

Run the example to test the integration:

```bash
cargo run --example external_usage
```

## API Reference

### MCP Functions

#### Twitter API

- `search_tweets(query, limit?, lang?, ...)` - Search tweets
- `get_user_info(username?, user_id?)` - Get user information
- `get_user_tweets(username?, user_id?, limit?, ...)` - Get user tweets

### Browser Functions

- `navigate(url, wait_for_load?)` - Navigate to URL
- `screenshot(full_page?, selector?)` - Take screenshot
- `execute_script(script, args?)` - Execute JavaScript
- `click(selector, button?, modifiers?)` - Click element
- `type_text(text, selector?, delay?)` - Type text
- `wait_for_element(selector, timeout?, state?)` - Wait for element
- `get_page_info()` - Get page information
- `get_element_info(selector)` - Get element information

## Future Enhancements

- Add more data sources to MCP
- Implement WebSocket support for real-time browser events
- Add screenshot comparison and visual testing
- Implement browser pool management
- Add authentication handling for APIs
