//! Example demonstrating how to use the integrated external APIs in Super IDE

use super_ide::external::{ExternalManager, ExternalConfig, api::McpApiClient, browser::BrowserClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Super IDE External API Integration Example");

    // Initialize external manager
    let config = ExternalConfig {
        python_path: "python".to_string(),
        external_api_path: "./external_api".to_string(),
        browser_path: "./browser".to_string(),
        mcp_server_port: 12306,
        browser_debug_port: 9222,
        request_timeout: 30,
    };

    let manager = ExternalManager::new(config.clone());

    // Start MCP server
    println!("📡 Starting MCP server...");
    if let Err(e) = manager.start_mcp_server().await {
        eprintln!("Failed to start MCP server: {}", e);
    }

    // Start browser automation
    println!("🌐 Starting browser automation...");
    if let Err(e) = manager.start_browser().await {
        eprintln!("Failed to start browser: {}", e);
    }

    // Create API client
    let api_client = McpApiClient::new(config.clone());

    // Example: Search Twitter tweets
    println!("🐦 Searching Twitter for 'Rust programming'...");
    let search_request = super_ide::external::api::TwitterSearchRequest {
        query: "Rust programming".to_string(),
        limit: Some(5),
        lang: Some("en".to_string()),
        ..Default::default()
    };

    match api_client.search_tweets(search_request).await {
        Ok(tweets) => {
            println!("✅ Found tweets:");
            if let Some(tweets_array) = tweets.as_array() {
                for tweet in tweets_array.iter().take(3) {
                    if let Some(text) = tweet.get("text").and_then(|t| t.as_str()) {
                        println!("  - {}", text.chars().take(100).collect::<String>());
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Twitter search failed: {}", e);
        }
    }

    // Create browser client
    let browser_client = BrowserClient::new(config.clone());

    // Example: Navigate to a webpage
    println!("🌍 Navigating to Rust website...");
    let navigate_request = super_ide::external::browser::BrowserNavigateRequest {
        url: "https://www.rust-lang.org/".to_string(),
        wait_for_load: Some(true),
    };

    match browser_client.navigate(navigate_request).await {
        Ok(page_info) => {
            println!("✅ Navigation successful:");
            println!("  - URL: {}", page_info.url);
            println!("  - Title: {}", page_info.title);
        }
        Err(e) => {
            println!("❌ Navigation failed: {}", e);
        }
    }

    // Example: Take a screenshot
    println!("📸 Taking screenshot...");
    let screenshot_request = super_ide::external::browser::BrowserScreenshotRequest {
        full_page: Some(true),
        selector: None,
    };

    match browser_client.screenshot(screenshot_request).await {
        Ok(image_data) => {
            println!("✅ Screenshot taken, {} bytes", image_data.len());
        }
        Err(e) => {
            println!("❌ Screenshot failed: {}", e);
        }
    }

    // Example: Execute JavaScript
    println!("⚡ Executing JavaScript...");
    let script_request = super_ide::external::browser::BrowserExecuteScriptRequest {
        script: "return document.title;".to_string(),
        args: None,
    };

    match browser_client.execute_script(script_request).await {
        Ok(result) => {
            println!("✅ Script result: {:?}", result);
        }
        Err(e) => {
            println!("❌ Script execution failed: {}", e);
        }
    }

    // Stop services
    println!("🛑 Stopping services...");
    let _ = manager.stop_mcp_server().await;
    let _ = manager.stop_browser().await;

    println!("✅ Example completed!");
    Ok(())
}
