//! HTTP Request structure for FAGA Browser

use std::collections::HashMap;

/// Represents an HTTP request
#[derive(Debug, Clone)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Request {
    /// Create a new GET request
    pub fn get(url: &str) -> Self {
        Self {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Create a new POST request
    pub fn post(url: &str, body: &str) -> Self {
        Self {
            url: url.to_string(),
            method: "POST".to_string(),
            headers: HashMap::new(),
            body: Some(body.to_string()),
        }
    }

    /// Add a header to the request
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the request body
    pub fn with_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
}

/// Builder pattern for creating requests
pub struct RequestBuilder {
    request: Request,
}

impl RequestBuilder {
    pub fn new(url: &str) -> Self {
        Self {
            request: Request::get(url),
        }
    }

    pub fn method(mut self, method: &str) -> Self {
        self.request.method = method.to_string();
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.request.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.request.body = Some(body.to_string());
        self
    }

    pub fn build(self) -> Request {
        self.request
    }
}
