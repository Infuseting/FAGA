//! HTTP Response structure for FAGA Browser

/// Represents an HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub content_type: String,
    pub body: String,
    pub url: String,
}

impl Response {
    /// Check if the response was successful (2xx status)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Check if the response is a redirect (3xx status)
    pub fn is_redirect(&self) -> bool {
        self.status >= 300 && self.status < 400
    }

    /// Check if the response is a client error (4xx status)
    pub fn is_client_error(&self) -> bool {
        self.status >= 400 && self.status < 500
    }

    /// Check if the response is a server error (5xx status)
    pub fn is_server_error(&self) -> bool {
        self.status >= 500 && self.status < 600
    }

    /// Check if the content type is HTML
    pub fn is_html(&self) -> bool {
        self.content_type.contains("text/html")
    }

    /// Check if the content type is CSS
    pub fn is_css(&self) -> bool {
        self.content_type.contains("text/css")
    }

    /// Check if the content type is JavaScript
    pub fn is_javascript(&self) -> bool {
        self.content_type.contains("javascript")
    }

    /// Check if the content type is JSON
    pub fn is_json(&self) -> bool {
        self.content_type.contains("application/json")
    }

    /// Check if the content type is an image
    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }

    /// Get the body as bytes
    pub fn body_bytes(&self) -> &[u8] {
        self.body.as_bytes()
    }

    /// Get the content length
    pub fn content_length(&self) -> usize {
        self.body.len()
    }
}
