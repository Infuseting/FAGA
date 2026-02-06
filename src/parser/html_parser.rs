//! HTML Parser for FAGA Browser
//! Parses HTML content into a DOM tree structure

use scraper::{Html, Selector, ElementRef};
use super::dom::{Document, Element, Node};

/// HTML Parser using scraper crate
pub struct HtmlParser;

impl HtmlParser {
    /// Parse HTML string into a Document
    pub fn parse(html: &str, base_url: &str) -> Result<Document, HtmlParseError> {
        log::info!("📄 Parsing HTML document...");

        let parsed = Html::parse_document(html);
        let mut document = Document::new();
        document.base_url = base_url.to_string();

        // Extract title
        if let Some(title) = Self::extract_title(&parsed) {
            document.set_title(&title);
            log::debug!("📌 Document title: {}", title);
        }

        // Extract stylesheets
        document.stylesheets = Self::extract_stylesheets(&parsed, base_url);
        log::debug!("🎨 Found {} stylesheets", document.stylesheets.len());

        // Extract scripts
        document.scripts = Self::extract_scripts(&parsed, base_url);
        log::debug!("📜 Found {} scripts", document.scripts.len());

        // Build DOM tree from body
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body) = parsed.select(&body_selector).next() {
                document.root = Some(Self::element_to_node(body));
            }
        }

        // Fallback: parse html element if no body
        if document.root.is_none() {
            if let Ok(html_selector) = Selector::parse("html") {
                if let Some(html_elem) = parsed.select(&html_selector).next() {
                    document.root = Some(Self::element_to_node(html_elem));
                }
            }
        }

        log::info!("✅ HTML parsing complete");
        Ok(document)
    }

    /// Extract document title
    fn extract_title(html: &Html) -> Option<String> {
        let selector = Selector::parse("title").ok()?;
        html.select(&selector).next().map(|el| el.text().collect::<String>())
    }

    /// Extract stylesheet URLs
    fn extract_stylesheets(html: &Html, base_url: &str) -> Vec<String> {
        let mut stylesheets = Vec::new();

        // External stylesheets via <link>
        if let Ok(selector) = Selector::parse("link[rel='stylesheet'], link[rel='Stylesheet']") {
            for link in html.select(&selector) {
                if let Some(href) = link.value().attr("href") {
                    let url = Self::resolve_url(href, base_url);
                    stylesheets.push(url);
                }
            }
        }

        // Inline styles via <style>
        if let Ok(selector) = Selector::parse("style") {
            for style in html.select(&selector) {
                let css = style.text().collect::<String>();
                if !css.trim().is_empty() {
                    // Mark as inline with special prefix
                    stylesheets.push(format!("inline:{}", css));
                }
            }
        }

        stylesheets
    }

    /// Extract script URLs
    fn extract_scripts(html: &Html, base_url: &str) -> Vec<String> {
        let mut scripts = Vec::new();

        if let Ok(selector) = Selector::parse("script[src]") {
            for script in html.select(&selector) {
                if let Some(src) = script.value().attr("src") {
                    let url = Self::resolve_url(src, base_url);
                    scripts.push(url);
                }
            }
        }

        scripts
    }

    /// Convert a scraper ElementRef to our Node structure
    fn element_to_node(element: ElementRef) -> Node {
        let tag_name = element.value().name().to_string();
        let mut elem = Element::new(&tag_name);

        // Copy attributes
        for (name, value) in element.value().attrs() {
            elem.set_attribute(name, value);
        }

        // Process children
        for child in element.children() {
            match child.value() {
                scraper::node::Node::Element(_) => {
                    if let Some(child_elem) = ElementRef::wrap(child) {
                        elem.append_child(Self::element_to_node(child_elem));
                    }
                }
                scraper::node::Node::Text(text) => {
                    let text_content = text.text.to_string();
                    if !text_content.trim().is_empty() {
                        elem.append_child(Node::Text(text_content));
                    }
                }
                scraper::node::Node::Comment(comment) => {
                    elem.append_child(Node::Comment(comment.comment.to_string()));
                }
                _ => {}
            }
        }

        Node::Element(elem)
    }

    /// Resolve a relative URL against a base URL
    fn resolve_url(href: &str, base_url: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("//") {
            if href.starts_with("//") {
                format!("https:{}", href)
            } else {
                href.to_string()
            }
        } else if href.starts_with('/') {
            // Absolute path
            if let Ok(base) = url::Url::parse(base_url) {
                format!("{}://{}{}", base.scheme(), base.host_str().unwrap_or(""), href)
            } else {
                href.to_string()
            }
        } else {
            // Relative path
            if let Ok(base) = url::Url::parse(base_url) {
                base.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string())
            } else {
                href.to_string()
            }
        }
    }

    /// Parse a fragment of HTML (not a complete document)
    pub fn parse_fragment(html: &str) -> Result<Node, HtmlParseError> {
        let fragment = Html::parse_fragment(html);

        if let Ok(selector) = Selector::parse("*") {
            if let Some(root) = fragment.select(&selector).next() {
                return Ok(Self::element_to_node(root));
            }
        }

        Err(HtmlParseError::EmptyDocument)
    }

    /// Extract all text content from HTML
    pub fn extract_text(html: &str) -> String {
        let parsed = Html::parse_document(html);
        let mut text = String::new();

        if let Ok(selector) = Selector::parse("body") {
            if let Some(body) = parsed.select(&selector).next() {
                Self::collect_text(body, &mut text);
            }
        }

        text.trim().to_string()
    }

    fn collect_text(element: ElementRef, output: &mut String) {
        for child in element.children() {
            match child.value() {
                scraper::node::Node::Text(text) => {
                    output.push_str(&text.text);
                    output.push(' ');
                }
                scraper::node::Node::Element(_) => {
                    // Skip script and style content
                    let tag = child.value().as_element().map(|e| e.name()).unwrap_or("");
                    if tag != "script" && tag != "style" {
                        if let Some(child_elem) = ElementRef::wrap(child) {
                            Self::collect_text(child_elem, output);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Errors during HTML parsing
#[derive(Debug, Clone)]
pub enum HtmlParseError {
    EmptyDocument,
    InvalidHtml(String),
    SelectorError(String),
}

impl std::fmt::Display for HtmlParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDocument => write!(f, "Empty document"),
            Self::InvalidHtml(e) => write!(f, "Invalid HTML: {}", e),
            Self::SelectorError(e) => write!(f, "Selector error: {}", e),
        }
    }
}

impl std::error::Error for HtmlParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Page</title></head>
            <body>
                <h1>Hello World</h1>
                <p class="intro">This is a test.</p>
            </body>
            </html>
        "#;

        let doc = HtmlParser::parse(html, "https://example.com").unwrap();
        assert_eq!(doc.title, "Test Page");
    }

    #[test]
    fn test_extract_text() {
        let html = r#"
            <html>
            <body>
                <h1>Title</h1>
                <p>Paragraph text</p>
            </body>
            </html>
        "#;

        let text = HtmlParser::extract_text(html);
        assert!(text.contains("Title"));
        assert!(text.contains("Paragraph"));
    }
}
