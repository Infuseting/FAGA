//! CSS Parser for FAGA Browser
//! Parses CSS content into style rules

use std::collections::HashMap;

/// CSS Parser for the browser
pub struct CssParser;

/// Represents a CSS stylesheet
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
}

/// Represents a single CSS rule
#[derive(Debug, Clone)]
pub struct CssRule {
    pub selectors: Vec<String>,
    pub declarations: HashMap<String, CssValue>,
}

/// Represents a CSS value
#[derive(Debug, Clone)]
pub enum CssValue {
    Keyword(String),
    Length(f32, LengthUnit),
    Percentage(f32),
    Color(CssColor),
    Number(f32),
    String(String),
    Url(String),
    Multiple(Vec<CssValue>),
}

/// Length units in CSS
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthUnit {
    Px,
    Em,
    Rem,
    Vh,
    Vw,
    Percent,
    Pt,
    Cm,
    Mm,
    In,
}

/// CSS Color representation
#[derive(Debug, Clone, Copy)]
pub struct CssColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl CssColor {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Named colors lookup
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "black" => Some(Self::rgb(0, 0, 0)),
            "white" => Some(Self::rgb(255, 255, 255)),
            "red" => Some(Self::rgb(255, 0, 0)),
            "green" => Some(Self::rgb(0, 128, 0)),
            "blue" => Some(Self::rgb(0, 0, 255)),
            "yellow" => Some(Self::rgb(255, 255, 0)),
            "cyan" => Some(Self::rgb(0, 255, 255)),
            "magenta" => Some(Self::rgb(255, 0, 255)),
            "gray" | "grey" => Some(Self::rgb(128, 128, 128)),
            "silver" => Some(Self::rgb(192, 192, 192)),
            "maroon" => Some(Self::rgb(128, 0, 0)),
            "olive" => Some(Self::rgb(128, 128, 0)),
            "lime" => Some(Self::rgb(0, 255, 0)),
            "aqua" => Some(Self::rgb(0, 255, 255)),
            "teal" => Some(Self::rgb(0, 128, 128)),
            "navy" => Some(Self::rgb(0, 0, 128)),
            "fuchsia" => Some(Self::rgb(255, 0, 255)),
            "purple" => Some(Self::rgb(128, 0, 128)),
            "orange" => Some(Self::rgb(255, 165, 0)),
            "pink" => Some(Self::rgb(255, 192, 203)),
            "brown" => Some(Self::rgb(165, 42, 42)),
            "transparent" => Some(Self::rgba(0, 0, 0, 0.0)),
            _ => None,
        }
    }
}

impl CssParser {
    /// Parse CSS string into a Stylesheet
    pub fn parse(css: &str) -> Result<Stylesheet, CssParseError> {
        log::info!("🎨 Parsing CSS...");

        let mut stylesheet = Stylesheet::default();
        let css = Self::remove_comments(css);

        // Simple rule-based parsing
        let rules = Self::split_rules(&css);

        for rule_str in rules {
            if let Some(rule) = Self::parse_rule(&rule_str) {
                stylesheet.rules.push(rule);
            }
        }

        log::info!("✅ CSS parsing complete: {} rules", stylesheet.rules.len());
        Ok(stylesheet)
    }

    /// Remove CSS comments
    fn remove_comments(css: &str) -> String {
        let mut result = String::new();
        let mut chars = css.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '/' {
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume '*'
                    // Skip until */
                    while let Some(c2) = chars.next() {
                        if c2 == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Split CSS into individual rules
    fn split_rules(css: &str) -> Vec<String> {
        let mut rules = Vec::new();
        let mut current = String::new();
        let mut brace_depth = 0;

        for c in css.chars() {
            match c {
                '{' => {
                    brace_depth += 1;
                    current.push(c);
                }
                '}' => {
                    brace_depth -= 1;
                    current.push(c);
                    if brace_depth == 0 {
                        let rule = current.trim().to_string();
                        if !rule.is_empty() {
                            rules.push(rule);
                        }
                        current.clear();
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        rules
    }

    /// Parse a single CSS rule
    fn parse_rule(rule: &str) -> Option<CssRule> {
        let brace_pos = rule.find('{')?;
        let end_brace = rule.rfind('}')?;

        let selector_part = rule[..brace_pos].trim();
        let declarations_part = rule[brace_pos + 1..end_brace].trim();

        // Skip @-rules for now (media queries, keyframes, etc.)
        if selector_part.starts_with('@') {
            return None;
        }

        let selectors: Vec<String> = selector_part
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if selectors.is_empty() {
            return None;
        }

        let declarations = Self::parse_declarations(declarations_part);

        Some(CssRule {
            selectors,
            declarations,
        })
    }

    /// Parse CSS declarations (property: value pairs)
    fn parse_declarations(declarations: &str) -> HashMap<String, CssValue> {
        let mut result = HashMap::new();

        for decl in declarations.split(';') {
            let decl = decl.trim();
            if decl.is_empty() {
                continue;
            }

            if let Some(colon_pos) = decl.find(':') {
                let property = decl[..colon_pos].trim().to_lowercase();
                let value = decl[colon_pos + 1..].trim();

                // Remove !important for now
                let value = value.trim_end_matches("!important").trim();

                // Handle shorthand properties with multiple values
                Self::parse_shorthand_property(&property, value, &mut result);
            }
        }

        result
    }

    /// Parse shorthand properties (margin, padding, etc.) with multiple values
    fn parse_shorthand_property(property: &str, value: &str, result: &mut HashMap<String, CssValue>) {
        match property {
            "margin" => {
                let parts: Vec<&str> = value.split_whitespace().collect();
                match parts.len() {
                    1 => {
                        // margin: 10px; -> all sides
                        if let Some(v) = Self::parse_value(parts[0]) {
                            result.insert("margin-top".to_string(), v.clone());
                            result.insert("margin-right".to_string(), v.clone());
                            result.insert("margin-bottom".to_string(), v.clone());
                            result.insert("margin-left".to_string(), v);
                        }
                    }
                    2 => {
                        // margin: 10px auto; -> vertical horizontal
                        if let Some(v) = Self::parse_value(parts[0]) {
                            result.insert("margin-top".to_string(), v.clone());
                            result.insert("margin-bottom".to_string(), v);
                        }
                        if let Some(h) = Self::parse_value(parts[1]) {
                            result.insert("margin-left".to_string(), h.clone());
                            result.insert("margin-right".to_string(), h);
                        }
                    }
                    3 => {
                        // margin: 10px 20px 30px; -> top horizontal bottom
                        if let Some(t) = Self::parse_value(parts[0]) {
                            result.insert("margin-top".to_string(), t);
                        }
                        if let Some(h) = Self::parse_value(parts[1]) {
                            result.insert("margin-left".to_string(), h.clone());
                            result.insert("margin-right".to_string(), h);
                        }
                        if let Some(b) = Self::parse_value(parts[2]) {
                            result.insert("margin-bottom".to_string(), b);
                        }
                    }
                    4 => {
                        // margin: 10px 20px 30px 40px; -> top right bottom left
                        if let Some(t) = Self::parse_value(parts[0]) {
                            result.insert("margin-top".to_string(), t);
                        }
                        if let Some(r) = Self::parse_value(parts[1]) {
                            result.insert("margin-right".to_string(), r);
                        }
                        if let Some(b) = Self::parse_value(parts[2]) {
                            result.insert("margin-bottom".to_string(), b);
                        }
                        if let Some(l) = Self::parse_value(parts[3]) {
                            result.insert("margin-left".to_string(), l);
                        }
                    }
                    _ => {}
                }
            }
            "padding" => {
                let parts: Vec<&str> = value.split_whitespace().collect();
                match parts.len() {
                    1 => {
                        if let Some(v) = Self::parse_value(parts[0]) {
                            result.insert("padding-top".to_string(), v.clone());
                            result.insert("padding-right".to_string(), v.clone());
                            result.insert("padding-bottom".to_string(), v.clone());
                            result.insert("padding-left".to_string(), v);
                        }
                    }
                    2 => {
                        if let Some(v) = Self::parse_value(parts[0]) {
                            result.insert("padding-top".to_string(), v.clone());
                            result.insert("padding-bottom".to_string(), v);
                        }
                        if let Some(h) = Self::parse_value(parts[1]) {
                            result.insert("padding-left".to_string(), h.clone());
                            result.insert("padding-right".to_string(), h);
                        }
                    }
                    3 => {
                        if let Some(t) = Self::parse_value(parts[0]) {
                            result.insert("padding-top".to_string(), t);
                        }
                        if let Some(h) = Self::parse_value(parts[1]) {
                            result.insert("padding-left".to_string(), h.clone());
                            result.insert("padding-right".to_string(), h);
                        }
                        if let Some(b) = Self::parse_value(parts[2]) {
                            result.insert("padding-bottom".to_string(), b);
                        }
                    }
                    4 => {
                        if let Some(t) = Self::parse_value(parts[0]) {
                            result.insert("padding-top".to_string(), t);
                        }
                        if let Some(r) = Self::parse_value(parts[1]) {
                            result.insert("padding-right".to_string(), r);
                        }
                        if let Some(b) = Self::parse_value(parts[2]) {
                            result.insert("padding-bottom".to_string(), b);
                        }
                        if let Some(l) = Self::parse_value(parts[3]) {
                            result.insert("padding-left".to_string(), l);
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                // Not a shorthand property, parse normally
                if let Some(css_value) = Self::parse_value(value) {
                    result.insert(property.to_string(), css_value);
                }
            }
        }
    }

    /// Parse a CSS value
    fn parse_value(value: &str) -> Option<CssValue> {
        let value = value.trim();

        if value.is_empty() {
            return None;
        }

        // Try to parse as color (hex)
        if value.starts_with('#') {
            if let Some(color) = CssColor::from_hex(value) {
                return Some(CssValue::Color(color));
            }
        }

        // Try to parse as named color
        if let Some(color) = CssColor::from_name(value) {
            return Some(CssValue::Color(color));
        }

        // Try to parse as rgb/rgba
        if value.starts_with("rgb") {
            if let Some(color) = Self::parse_rgb(value) {
                return Some(CssValue::Color(color));
            }
        }

        // Try to parse as url()
        if value.starts_with("url(") && value.ends_with(')') {
            let url = value[4..value.len() - 1].trim();
            let url = url.trim_matches('"').trim_matches('\'');
            return Some(CssValue::Url(url.to_string()));
        }

        // Try to parse as length with unit
        if let Some(length) = Self::parse_length(value) {
            return Some(length);
        }

        // Try to parse as number
        if let Ok(num) = value.parse::<f32>() {
            return Some(CssValue::Number(num));
        }

        // Default to keyword
        Some(CssValue::Keyword(value.to_string()))
    }

    /// Parse a CSS length value
    fn parse_length(value: &str) -> Option<CssValue> {
        let units = [
            ("px", LengthUnit::Px),
            ("em", LengthUnit::Em),
            ("rem", LengthUnit::Rem),
            ("vh", LengthUnit::Vh),
            ("vw", LengthUnit::Vw),
            ("%", LengthUnit::Percent),
            ("pt", LengthUnit::Pt),
            ("cm", LengthUnit::Cm),
            ("mm", LengthUnit::Mm),
            ("in", LengthUnit::In),
        ];

        for (suffix, unit) in units {
            if value.ends_with(suffix) {
                let num_part = &value[..value.len() - suffix.len()];
                if let Ok(num) = num_part.parse::<f32>() {
                    if unit == LengthUnit::Percent {
                        return Some(CssValue::Percentage(num));
                    }
                    return Some(CssValue::Length(num, unit));
                }
            }
        }

        None
    }

    /// Parse rgb() or rgba() color
    fn parse_rgb(value: &str) -> Option<CssColor> {
        let is_rgba = value.starts_with("rgba");
        let start = if is_rgba { 5 } else { 4 };

        let inner = value.get(start..value.len() - 1)?.trim();
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

        if parts.len() >= 3 {
            let r = parts[0].trim_end_matches('%').parse::<f32>().ok()?;
            let g = parts[1].trim_end_matches('%').parse::<f32>().ok()?;
            let b = parts[2].trim_end_matches('%').parse::<f32>().ok()?;

            let r = if parts[0].ends_with('%') { (r * 2.55) as u8 } else { r as u8 };
            let g = if parts[1].ends_with('%') { (g * 2.55) as u8 } else { g as u8 };
            let b = if parts[2].ends_with('%') { (b * 2.55) as u8 } else { b as u8 };

            let a = if parts.len() >= 4 {
                parts[3].parse::<f32>().ok()?
            } else {
                1.0
            };

            return Some(CssColor::rgba(r, g, b, a));
        }

        None
    }

    /// Parse inline style attribute
    pub fn parse_inline_style(style: &str) -> HashMap<String, CssValue> {
        Self::parse_declarations(style)
    }

    /// Get computed style for an element based on matching rules
    pub fn get_computed_style(
        stylesheet: &Stylesheet,
        element_tag: &str,
        element_id: Option<&str>,
        element_classes: &[&str],
    ) -> HashMap<String, CssValue> {
        let mut computed = HashMap::new();

        for rule in &stylesheet.rules {
            for selector in &rule.selectors {
                if Self::selector_matches(selector, element_tag, element_id, element_classes) {
                    // Merge declarations (later rules override)
                    for (prop, value) in &rule.declarations {
                        computed.insert(prop.clone(), value.clone());
                    }
                }
            }
        }

        computed
    }

    /// Check if a selector matches an element (simplified)
    fn selector_matches(
        selector: &str,
        tag: &str,
        id: Option<&str>,
        classes: &[&str],
    ) -> bool {
        let selector = selector.trim();

        // Universal selector
        if selector == "*" {
            return true;
        }

        // ID selector
        if selector.starts_with('#') {
            return id == Some(&selector[1..]);
        }

        // Class selector
        if selector.starts_with('.') {
            return classes.contains(&&selector[1..]);
        }

        // Tag selector (simple case)
        if selector.eq_ignore_ascii_case(tag) {
            return true;
        }

        // Combined selectors (tag.class, tag#id, etc.)
        if let Some(dot_pos) = selector.find('.') {
            let tag_part = &selector[..dot_pos];
            let class_part = &selector[dot_pos + 1..];
            return (tag_part.is_empty() || tag_part.eq_ignore_ascii_case(tag))
                && classes.contains(&class_part);
        }

        if let Some(hash_pos) = selector.find('#') {
            let tag_part = &selector[..hash_pos];
            let id_part = &selector[hash_pos + 1..];
            return (tag_part.is_empty() || tag_part.eq_ignore_ascii_case(tag))
                && id == Some(id_part);
        }

        false
    }
}

/// Errors during CSS parsing
#[derive(Debug, Clone)]
pub enum CssParseError {
    InvalidSyntax(String),
    UnexpectedToken(String),
}

impl std::fmt::Display for CssParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSyntax(e) => write!(f, "Invalid CSS syntax: {}", e),
            Self::UnexpectedToken(e) => write!(f, "Unexpected token: {}", e),
        }
    }
}

impl std::error::Error for CssParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_css() {
        let css = r#"
            body {
                background-color: #ffffff;
                font-size: 16px;
            }
            .container {
                width: 100%;
                margin: 0 auto;
            }
        "#;

        let stylesheet = CssParser::parse(css).unwrap();
        assert_eq!(stylesheet.rules.len(), 2);
    }

    #[test]
    fn test_parse_color() {
        let color = CssColor::from_hex("#ff0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }
}
