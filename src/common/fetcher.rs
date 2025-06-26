use reqwest::Client;
use rmcp::{Error as McpError, ServerHandler, model::*, schemars, tool};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FetchRequest {
    pub url: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchCrateRequest {
    pub query: String,
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
        let url = req.url.clone();
        let req_builder = self.build_request(&url);
        let resp = req_builder.send().await;
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

    #[tool(description = "Search Rust crates on docs.rs by keyword")]
    pub async fn search_crate(&self, #[tool(aggr)] req: SearchCrateRequest) -> Result<CallToolResult, McpError> {
        let encoded_query = urlencoding::encode(&req.query);
        let url = format!("https://docs.rs/releases/search?query={}", encoded_query);
        let req_builder = self.build_request(&url);
        let resp = req_builder.send().await;
        match resp {
            Ok(response) => {
                let text = response.text().await;
                match text {
                    Ok(body) => {
                        let results = parse_crate_search_results(&body, "https://docs.rs");
                        let json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
                        Ok(CallToolResult::success(vec![Content::text(json)]))
                    },
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to read response body: {}",
                        e
                    ))])),
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to search crates: {}",
                e
            ))])),
        }
    }

    #[tool(description = "Fetch and parse a docs.rs crate page, returning structured documentation info. The URL must be of the form https://docs.rs/cratename/version/cratename/")]
    pub async fn fetch_docsrs(&self, #[tool(aggr)] req: FetchRequest) -> Result<CallToolResult, McpError> {
        let url = req.url.clone();
        if !is_docsrs_crate_page(&url) {
            return Ok(CallToolResult::error(vec![Content::text("Not a recognized docs.rs crate page URL. The URL must be of the form https://docs.rs/cratename/version/cratename/".to_string())]));
        }
        let req_builder = self.build_request(&url);
        let resp = req_builder.send().await;
        match resp {
            Ok(response) => {
                let text = response.text().await;
                match text {
                    Ok(body) => {
                        let structured = parse_docsrs_page_structured(&body);
                        let json = serde_json::to_string(&structured).unwrap_or_else(|_| "{}".to_string());
                        Ok(CallToolResult::success(vec![Content::text(json)]))
                    },
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to read response body: {}",
                        e
                    ))])),
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to fetch the docs.rs page: {}",
                e
            ))])),
        }
    }

    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        self.client.get(url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Code/1.101.1 Chrome/134.0.6998.205 Electron/35.5.1 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
            .header("Accept-Language", "en")
    }
}

#[tool(tool_box)]
impl ServerHandler for WebFetcher {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides tools for fetching web page content by URL. The 'fetch' tool fetches any web page by URL. The 'fetch_docsrs_structured' tool only accepts docs.rs crate page URLs.".to_string()),
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

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CrateSearchResult {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DocsrsPageStructured {
    pub content: String,
}

fn parse_crate_search_results(html: &str, base_url: &str) -> Vec<CrateSearchResult> {
    let document = Html::parse_document(html);
    let release_selector = Selector::parse("a.release").unwrap();
    let name_selector = Selector::parse("div.name").unwrap();
    let desc_selector = Selector::parse("div.description").unwrap();
    let mut results = Vec::new();
    for a in document.select(&release_selector) {
        let url_path = a.value().attr("href").unwrap_or("");
        let url = format!("{}{}", base_url, url_path);
        let name_version = a.select(&name_selector).next().map(|e| e.text().collect::<String>()).unwrap_or_default();
        let description = a.select(&desc_selector).next().map(|e| e.text().collect::<String>()).unwrap_or_default();
        // name-version 形式を分割
        let (name, version) = if let Some(idx) = name_version.rfind('-') {
            (name_version[..idx].to_string(), name_version[idx+1..].to_string())
        } else {
            (name_version, String::new())
        };
        if !name.is_empty() && !version.is_empty() {
            results.push(CrateSearchResult {
                name,
                version,
                description,
                url,
            });
        }
    }
    results
}

pub fn parse_docsrs_page_structured(html: &str) -> DocsrsPageStructured {
    let document = Html::parse_document(html);
    let selector = Selector::parse("#main-content, #main").unwrap();
    let content = document
        .select(&selector)
        .next()
        .map(|e| {
            e.text()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    DocsrsPageStructured {
        content,
    }
}

pub fn is_docsrs_crate_page(url: &str) -> bool {
    // 例: https://docs.rs/cratename/latest/cratename/ または https://docs.rs/cratename/version/cratename/index.html
    let re = regex::Regex::new(r"^https://docs\.rs/[^/]+/(latest|[\d.]+)/[^/]+(/index\.html)?/?$").unwrap();
    re.is_match(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use rmcp::model::RawContent;

    #[tokio::test]
    async fn test_search_crate_success() {
        let fetcher = WebFetcher::new();
        let query = "serde".to_string();
        let result = fetcher.search_crate(SearchCrateRequest { query }).await;
        match result {
            Ok(res) => {
                let texts: Vec<&str> = res.content.iter().filter_map(|c| {
                    match &c.raw {
                        RawContent::Text(text_content) => Some(text_content.text.as_str()),
                        _ => None,
                    }
                }).collect();
                let json = texts.join("");
                let parsed: Vec<CrateSearchResult> = serde_json::from_str(&json).expect("valid json");
                assert!(parsed.iter().any(|c| c.name.to_lowercase().contains("serde")), "Should contain a crate named 'serde'");
            }
            Err(e) => panic!("search_crate failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_fetch_success() {
        let fetcher = WebFetcher::new();
        let req = FetchRequest {
            url: "https://www.rust-lang.org/".to_string(),
        };
        let result = fetcher.fetch(req).await;
        match result {
            Ok(res) => {
                let texts: Vec<&str> = res.content.iter().filter_map(|c| {
                    match &c.raw {
                        RawContent::Text(text_content) => Some(text_content.text.as_str()),
                        _ => None,
                    }
                }).collect();
                assert!(texts.iter().any(|t| t.contains("Rust")), "Response should contain 'Rust'");
            }
            Err(e) => panic!("fetch failed: {}", e),
        }
    }

    #[test]
    fn test_build_request_headers() {
        let fetcher = WebFetcher::new();
        let url = "https://example.com";
        let req = fetcher.build_request(url).build().unwrap();
        let headers = req.headers();
        assert_eq!(headers.get("User-Agent").unwrap(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Code/1.101.1 Chrome/134.0.6998.205 Electron/35.5.1 Safari/537.36");
        assert_eq!(headers.get("Accept").unwrap(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7");
        assert_eq!(headers.get("Accept-Language").unwrap(), "en");
    }

    #[test]
    fn test_parse_crate_search_results() {
        let html = r#"
        <a class="release" href="/serde/1.0.197/serde/">
            <div class="name">serde-1.0.197</div>
            <div class="description">A generic serialization/deserialization framework</div>
        </a>
        <a class="release" href="/tokio/1.36.0/tokio/">
            <div class="name">tokio-1.36.0</div>
            <div class="description">An event-driven, non-blocking I/O platform</div>
        </a>
        "#;
        let base_url = "https://docs.rs";
        let results = super::parse_crate_search_results(html, base_url);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], super::CrateSearchResult {
            name: "serde".to_string(),
            version: "1.0.197".to_string(),
            description: "A generic serialization/deserialization framework".to_string(),
            url: "https://docs.rs/serde/1.0.197/serde/".to_string(),
        });
        assert_eq!(results[1], super::CrateSearchResult {
            name: "tokio".to_string(),
            version: "1.36.0".to_string(),
            description: "An event-driven, non-blocking I/O platform".to_string(),
            url: "https://docs.rs/tokio/1.36.0/tokio/".to_string(),
        });
    }

    #[tokio::test]
    async fn test_search_crate_with_space_in_query() {
        let fetcher = WebFetcher::new();
        let query = "serde json".to_string();
        let result = fetcher.search_crate(SearchCrateRequest { query }).await;
        match result {
            Ok(res) => {
                let texts: Vec<&str> = res.content.iter().filter_map(|c| {
                    match &c.raw {
                        RawContent::Text(text_content) => Some(text_content.text.as_str()),
                        _ => None,
                    }
                }).collect();
                let json = texts.join("");
                let parsed: Vec<CrateSearchResult> = serde_json::from_str(&json).expect("valid json");
                assert!(!parsed.is_empty(), "Should return at least one crate for a multi-word query");
            }
            Err(e) => panic!("search_crate failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_fetch_docsrs_serde() {
        let fetcher = WebFetcher::new();
        let req = FetchRequest {
            url: "https://docs.rs/serde/1.0.219/serde/index.html".to_string(),
        };
        let result = fetcher.fetch_docsrs(req).await.expect("call success");
        let json = result.content.iter().find_map(|c| match &c.raw {
            RawContent::Text(text) => Some(text.text.as_str()),
            _ => None,
        }).expect("should have json");
        let doc_page: DocsrsPageStructured = serde_json::from_str(json).expect("valid DocsrsPageStructured json");
        // content should contain some expected HTML
        assert!(doc_page.content.contains("serde"), "content should contain 'serde'");
    }

    #[tokio::test]
    async fn test_fetch_docsrs_rmcp() {
        let fetcher = WebFetcher::new();
        let req = FetchRequest {
            url: "https://docs.rs/rmcp/latest/rmcp/".to_string(),
        };
        let result = fetcher.fetch_docsrs(req).await.expect("call success");
        let json = result.content.iter().find_map(|c| match &c.raw {
            RawContent::Text(text) => Some(text.text.as_str()),
            _ => None,
        }).expect("should have json");
        let doc_page: DocsrsPageStructured = serde_json::from_str(json).expect("valid DocsrsPageStructured json");
        assert!(!doc_page.content.is_empty(), "content should not be empty");
    }
}
