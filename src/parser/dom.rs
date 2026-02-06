//! DOM structure for FAGA Browser
//! Represents the parsed HTML document tree

use std::collections::HashMap;

/// Represents an HTML document
#[derive(Debug, Clone)]
pub struct Document {
    pub root: Option<Node>,
    pub title: String,
    pub stylesheets: Vec<String>,
    pub scripts: Vec<String>,
    pub base_url: String,
}

impl Document {
    pub fn new() -> Self {
        Self {
            root: None,
            title: String::new(),
            stylesheets: Vec::new(),
            scripts: Vec::new(),
            base_url: String::new(),
        }
    }

    /// Set the document title
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    /// Add a stylesheet URL
    pub fn add_stylesheet(&mut self, url: &str) {
        self.stylesheets.push(url.to_string());
    }

    /// Add a script URL
    pub fn add_script(&mut self, url: &str) {
        self.scripts.push(url.to_string());
    }

    /// Get element by ID
    pub fn get_element_by_id(&self, id: &str) -> Option<&Node> {
        if let Some(ref root) = self.root {
            Self::find_by_id(root, id)
        } else {
            None
        }
    }

    fn find_by_id<'a>(node: &'a Node, id: &str) -> Option<&'a Node> {
        if let Node::Element(ref elem) = node {
            if elem.attributes.get("id").map(|s| s.as_str()) == Some(id) {
                return Some(node);
            }
            for child in &elem.children {
                if let Some(found) = Self::find_by_id(child, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Get elements by tag name
    pub fn get_elements_by_tag_name(&self, tag: &str) -> Vec<&Node> {
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            Self::collect_by_tag(root, tag, &mut results);
        }
        results
    }

    fn collect_by_tag<'a>(node: &'a Node, tag: &str, results: &mut Vec<&'a Node>) {
        if let Node::Element(ref elem) = node {
            if elem.tag_name.eq_ignore_ascii_case(tag) {
                results.push(node);
            }
            for child in &elem.children {
                Self::collect_by_tag(child, tag, results);
            }
        }
    }

    /// Get elements by class name
    pub fn get_elements_by_class_name(&self, class: &str) -> Vec<&Node> {
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            Self::collect_by_class(root, class, &mut results);
        }
        results
    }

    fn collect_by_class<'a>(node: &'a Node, class: &str, results: &mut Vec<&'a Node>) {
        if let Node::Element(ref elem) = node {
            if let Some(classes) = elem.attributes.get("class") {
                if classes.split_whitespace().any(|c| c == class) {
                    results.push(node);
                }
            }
            for child in &elem.children {
                Self::collect_by_class(child, class, results);
            }
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a DOM node
#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(String),
    Comment(String),
}

impl Node {
    /// Get inner text content
    pub fn text_content(&self) -> String {
        match self {
            Node::Text(text) => text.clone(),
            Node::Element(elem) => {
                elem.children.iter()
                    .map(|child| child.text_content())
                    .collect::<Vec<_>>()
                    .join("")
            }
            Node::Comment(_) => String::new(),
        }
    }

    /// Check if this is an element node
    pub fn is_element(&self) -> bool {
        matches!(self, Node::Element(_))
    }

    /// Check if this is a text node
    pub fn is_text(&self) -> bool {
        matches!(self, Node::Text(_))
    }

    /// Get as element (if it is one)
    pub fn as_element(&self) -> Option<&Element> {
        if let Node::Element(elem) = self {
            Some(elem)
        } else {
            None
        }
    }
}

/// Represents an HTML element
#[derive(Debug, Clone)]
pub struct Element {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Node>,
    pub styles: HashMap<String, String>,
}

impl Element {
    pub fn new(tag_name: &str) -> Self {
        Self {
            tag_name: tag_name.to_lowercase(),
            attributes: HashMap::new(),
            children: Vec::new(),
            styles: HashMap::new(),
        }
    }

    /// Get an attribute value
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    /// Set an attribute
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }

    /// Check if element has an attribute
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Add a child node
    pub fn append_child(&mut self, node: Node) {
        self.children.push(node);
    }

    /// Get the ID attribute
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    /// Get class list
    pub fn class_list(&self) -> Vec<&str> {
        self.attributes
            .get("class")
            .map(|c| c.split_whitespace().collect())
            .unwrap_or_default()
    }

    /// Check if element has a specific class
    pub fn has_class(&self, class: &str) -> bool {
        self.class_list().contains(&class)
    }

    /// Check if this is a void element (self-closing)
    pub fn is_void_element(&self) -> bool {
        matches!(
            self.tag_name.as_str(),
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" |
            "link" | "meta" | "param" | "source" | "track" | "wbr"
        )
    }

    /// Check if this is a block element
    pub fn is_block_element(&self) -> bool {
        matches!(
            self.tag_name.as_str(),
            "address" | "article" | "aside" | "blockquote" | "canvas" | "dd" | "div" |
            "dl" | "dt" | "fieldset" | "figcaption" | "figure" | "footer" | "form" |
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "header" | "hr" | "li" | "main" |
            "nav" | "noscript" | "ol" | "p" | "pre" | "section" | "table" | "tfoot" | "ul" | "video"
        )
    }

    /// Check if this is an inline element
    pub fn is_inline_element(&self) -> bool {
        !self.is_block_element()
    }
}
