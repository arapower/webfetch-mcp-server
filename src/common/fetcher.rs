use reqwest::Client;
use rmcp::{Error as McpError, ServerHandler, model::*, schemars, tool};
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FetchRequest {
    pub url: String,
}

#[derive(Clone)]
pub struct WebFetcher {
    client: Arc<Client>,
}

#[tool(tool_box)]
impl WebFetcher {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Client::new()),
        }
    }

    #[tool(description = "Fetch the content of a web page by URL")]
    pub async fn fetch(&self, #[tool(aggr)] req: FetchRequest) -> Result<CallToolResult, McpError> {
        let resp = self.client.get(&req.url).send().await;
        match resp {
            Ok(response) => {
                let text = response.text().await;
                match text {
                    Ok(body) => Ok(CallToolResult::success(vec![Content::text(body)])),
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to read response body: {}",
                        e
                    ))])),
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to fetch the web page: {}",
                e
            ))])),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for WebFetcher {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server fetches web page content by URL using the 'fetch' tool.".to_string(),
            ),
        }
    }
}

#[derive(Debug, Error)]
pub enum WebfetchError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
}

impl From<WebfetchError> for McpError {
    fn from(e: WebfetchError) -> Self {
        McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None)
    }
}
