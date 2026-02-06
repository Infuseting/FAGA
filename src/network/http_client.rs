//! HTTP/HTTPS Client for FAGA Browser
//! Handles all network requests with proper error handling and caching support

use reqwest::{Client, header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, ACCEPT_ENCODING}};
use std::time::Duration;
use url::Url;
use super::response::Response;
use super::request::Request;

/// Configuration for the HTTP client
#[derive(Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub max_redirects: usize,
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_redirects: 10,
            user_agent: format!("FAGA Browser/0.1.0 (Windows NT 10.0; Win64; x64)"),
        }
    }
}

/// Main HTTP client for the browser
pub struct HttpClient {
    client: Client,
    config: HttpClientConfig,
}

impl HttpClient {
    /// Create a new HTTP client with default configuration
    pub fn new() -> Result<Self, HttpClientError> {
        Self::with_config(HttpClientConfig::default())
    }

    /// Create a new HTTP client with custom configuration
    pub fn with_config(config: HttpClientConfig) -> Result<Self, HttpClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&config.user_agent)
            .map_err(|_| HttpClientError::InvalidHeader)?);
        headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5,fr;q=0.3"));
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects))
            .cookie_store(true)
            .gzip(true)
            .brotli(true)
            .build()
            .map_err(|e| HttpClientError::ClientBuildError(e.to_string()))?;

        Ok(Self { client, config })
    }

    /// Perform a GET request
    pub async fn get(&self, url: &str) -> Result<Response, HttpClientError> {
        let parsed_url = Url::parse(url)
            .map_err(|e| HttpClientError::InvalidUrl(e.to_string()))?;

        log::info!("🌐 GET request to: {}", url);

        let response = self.client
            .get(parsed_url.as_str())
            .send()
            .await
            .map_err(|e| HttpClientError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html")
            .to_string();

        let body = response
            .text()
            .await
            .map_err(|e| HttpClientError::ResponseReadError(e.to_string()))?;

        log::info!("✅ Response received: {} bytes, status: {}", body.len(), status);

        Ok(Response {
            status,
            content_type,
            body,
            url: url.to_string(),
        })
    }

    /// Perform a POST request
    pub async fn post(&self, url: &str, body: &str) -> Result<Response, HttpClientError> {
        let parsed_url = Url::parse(url)
            .map_err(|e| HttpClientError::InvalidUrl(e.to_string()))?;

        log::info!("📤 POST request to: {}", url);

        let response = self.client
            .post(parsed_url.as_str())
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| HttpClientError::RequestFailed(e.to_string()))?;

        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html")
            .to_string();

        let response_body = response
            .text()
            .await
            .map_err(|e| HttpClientError::ResponseReadError(e.to_string()))?;

        Ok(Response {
            status,
            content_type,
            body: response_body,
            url: url.to_string(),
        })
    }

    /// Perform a request from a Request object
    pub async fn execute(&self, request: Request) -> Result<Response, HttpClientError> {
        match request.method.as_str() {
            "GET" => self.get(&request.url).await,
            "POST" => self.post(&request.url, &request.body.unwrap_or_default()).await,
            _ => Err(HttpClientError::UnsupportedMethod(request.method)),
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

/// Errors that can occur during HTTP operations
#[derive(Debug, Clone)]
pub enum HttpClientError {
    InvalidUrl(String),
    InvalidHeader,
    ClientBuildError(String),
    RequestFailed(String),
    ResponseReadError(String),
    UnsupportedMethod(String),
    Timeout,
    NetworkError(String),
}

impl std::fmt::Display for HttpClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl(e) => write!(f, "Invalid URL: {}", e),
            Self::InvalidHeader => write!(f, "Invalid header"),
            Self::ClientBuildError(e) => write!(f, "Client build error: {}", e),
            Self::RequestFailed(e) => write!(f, "Request failed: {}", e),
            Self::ResponseReadError(e) => write!(f, "Response read error: {}", e),
            Self::UnsupportedMethod(m) => write!(f, "Unsupported HTTP method: {}", m),
            Self::Timeout => write!(f, "Request timeout"),
            Self::NetworkError(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl std::error::Error for HttpClientError {}
