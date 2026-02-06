//! HTML/CSS Rendering Engine for FAGA Browser
//! Converts parsed HTML/CSS into visual elements for display

use std::collections::HashMap;
use std::fs;
use super::dom::{Document, Node, Element};
use super::css_parser::{CssParser, CssValue, Stylesheet};

/// Load the default CSS from the assets folder
fn load_default_css() -> String {
    // Try multiple possible paths for the default CSS
    let possible_paths = [
        "assets/css/default.css",
        "./assets/css/default.css",
        "../assets/css/default.css",
    ];

    for path in &possible_paths {
        if let Ok(css) = fs::read_to_string(path) {
            log::info!("📄 Loaded default CSS from: {}", path);
            return css;
        }
    }

    log::warn!("⚠️ Could not load default.css, using minimal fallback");
    r#"
        body { margin: 8px; font-family: sans-serif; font-size: 16px; line-height: 1.5; }
        h1 { font-size: 2em; font-weight: bold; margin: 0.67em 0; }
        h2 { font-size: 1.5em; font-weight: bold; margin: 0.83em 0; }
        h3 { font-size: 1.17em; font-weight: bold; margin: 1em 0; }
        p { margin: 1em 0; }
        a { color: #1a0dab; text-decoration: underline; }
        strong, b { font-weight: bold; }
        em, i { font-style: italic; }
        ul, ol { margin: 1em 0; padding-left: 40px; }
        pre, code { font-family: monospace; background: #f5f5f5; }
        script, style, head { display: none; }
    "#.to_string()
}

/// Rendered element for display
#[derive(Debug, Clone)]
pub struct RenderNode {
    pub node_type: RenderNodeType,
    pub styles: ComputedStyles,
    pub children: Vec<RenderNode>,
    pub text: String,
    pub tag: String, // Tag name for identification (e.g., "body", "div")
    pub href: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RenderNodeType {
    Block,
    Inline,
    InlineBlock,
    ListItem,
    Table,
    TableRow,
    TableCell,
    Hidden,
    Text,
}

/// Computed CSS styles for rendering
#[derive(Debug, Clone)]
pub struct ComputedStyles {
    pub display: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub text_decoration: TextDecoration,
    pub text_align: TextAlign,
    pub line_height: f32,
    pub color: RenderColor,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub margin_left_auto: bool,  // Pour margin: auto
    pub margin_right_auto: bool, // Pour margin: auto
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub padding_right: f32,
    pub background_color: RenderColor,
    pub border_width: f32,
    pub border_color: RenderColor,
    pub border_radius: f32,
    pub list_style_type: String,
    pub width: Option<f32>,      // Largeur en pixels (None = auto)
    pub width_percent: Option<f32>, // Largeur en pourcentage
}

#[derive(Debug, Clone, Copy)]
pub enum FontWeight { Normal, Bold }

#[derive(Debug, Clone, Copy)]
pub enum FontStyle { Normal, Italic }

#[derive(Debug, Clone, Copy)]
pub enum TextDecoration { None, Underline, LineThrough }

#[derive(Debug, Clone, Copy)]
pub enum TextAlign { Left, Center, Right, Justify }

#[derive(Debug, Clone, Copy)]
pub struct RenderColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl RenderColor {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self { Self { r, g, b, a: 1.0 } }
    pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self { Self { r, g, b, a } }
    pub fn transparent() -> Self { Self::rgba(0, 0, 0, 0.0) }
    pub fn to_iced_color(&self) -> iced::Color {
        iced::Color::from_rgba8(self.r, self.g, self.b, self.a)
    }
}

impl Default for ComputedStyles {
    fn default() -> Self {
        Self {
            display: "block".to_string(),
            font_size: 16.0,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            text_decoration: TextDecoration::None,
            text_align: TextAlign::Left,
            line_height: 1.5,
            color: RenderColor::rgb(26, 26, 26),
            margin_top: 0.0, margin_bottom: 0.0, margin_left: 0.0, margin_right: 0.0,
            margin_left_auto: false, margin_right_auto: false,
            padding_top: 0.0, padding_bottom: 0.0, padding_left: 0.0, padding_right: 0.0,
            background_color: RenderColor::transparent(),
            border_width: 0.0,
            border_color: RenderColor::transparent(),
            border_radius: 0.0,
            list_style_type: "none".to_string(),
            width: None,
            width_percent: None,
        }
    }
}

/// HTML Renderer - converts DOM to render tree using external CSS file
pub struct HtmlRenderer {
    default_stylesheet: Stylesheet,
    page_stylesheets: Vec<Stylesheet>,
    base_font_size: f32,
    viewport_width: f32,
    viewport_height: f32,
}

impl HtmlRenderer {
    pub fn new() -> Self {
        let default_css = load_default_css();
        let default_stylesheet = CssParser::parse(&default_css).unwrap_or_default();
        Self {
            default_stylesheet,
            page_stylesheets: Vec::new(),
            base_font_size: 16.0,
            viewport_width: 1200.0,
            viewport_height: 800.0,
        }
    }

    pub fn with_viewport(mut self, width: f32, height: f32) -> Self {
        self.viewport_width = width;
        self.viewport_height = height;
        self
    }

    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    pub fn add_stylesheet(&mut self, css: &str) {
        if let Ok(stylesheet) = CssParser::parse(css) {
            self.page_stylesheets.push(stylesheet);
        }
    }

    pub fn clear_stylesheets(&mut self) {
        self.page_stylesheets.clear();
    }

    pub fn render(&self, document: &Document) -> Option<RenderNode> {
        document.root.as_ref().map(|root| self.render_node(root, &ComputedStyles::default()))
    }

    fn render_node(&self, node: &Node, parent_styles: &ComputedStyles) -> RenderNode {
        match node {
            Node::Text(text) => RenderNode {
                node_type: RenderNodeType::Text,
                styles: parent_styles.clone(),
                children: Vec::new(),
                text: text.clone(),
                tag: String::new(),
                href: None,
            },
            Node::Comment(_) => RenderNode {
                node_type: RenderNodeType::Hidden,
                styles: ComputedStyles::default(),
                children: Vec::new(),
                text: String::new(),
                tag: String::new(),
                href: None,
            },
            Node::Element(elem) => self.render_element(elem, parent_styles),
        }
    }

    fn render_element(&self, elem: &Element, parent_styles: &ComputedStyles) -> RenderNode {
        let styles = self.compute_styles(elem, parent_styles);
        let node_type = self.determine_node_type(&elem.tag_name, &styles);
        let tag = elem.tag_name.to_lowercase();

        // Récupérer l'attribut href pour les liens <a>
        let href = if tag == "a" {
            elem.attributes.get("href").cloned()
        } else {
            None
        };

        if matches!(node_type, RenderNodeType::Hidden) {
            return RenderNode {
                node_type: RenderNodeType::Hidden,
                styles,
                children: Vec::new(),
                text: String::new(),
                tag,
                href: None,
            };
        }

        let children: Vec<RenderNode> = elem.children
            .iter()
            .map(|child| self.render_node(child, &styles))
            .filter(|n| !matches!(n.node_type, RenderNodeType::Hidden))
            .collect();

        RenderNode { node_type, styles, children, text: String::new(), tag, href }
    }

    fn compute_styles(&self, elem: &Element, parent_styles: &ComputedStyles) -> ComputedStyles {
        let mut styles = ComputedStyles::default();
        let parent_font_size = parent_styles.font_size; // Sauvegarder le font-size parent
        styles.font_size = parent_font_size;
        styles.color = parent_styles.color;
        styles.line_height = parent_styles.line_height;
        styles.text_align = parent_styles.text_align;

        self.apply_tag_defaults(&elem.tag_name, &mut styles);

        let font_size_after_defaults = styles.font_size;

        let id = elem.attributes.get("id").map(|s| s.as_str());
        let classes: Vec<&str> = elem.attributes
            .get("class")
            .map(|c| c.split_whitespace().collect())
            .unwrap_or_default();

        // Pour le CSS, les em sont relatifs au parent (pas aux tag defaults)
        self.apply_stylesheet_styles_with_parent(&self.default_stylesheet, &elem.tag_name, id, &classes, &mut styles, parent_font_size);

        let font_size_after_default_css = styles.font_size;

        for stylesheet in &self.page_stylesheets {
            self.apply_stylesheet_styles_with_parent(stylesheet, &elem.tag_name, id, &classes, &mut styles, parent_font_size);
        }

        let font_size_after_page_css = styles.font_size;

        if let Some(inline_style) = elem.attributes.get("style") {
            let declarations = CssParser::parse_inline_style(inline_style);
            self.apply_declarations_with_parent(&declarations, &mut styles, parent_font_size);
        }

        // Log pour les éléments de titre
        if elem.tag_name.starts_with('h') && elem.tag_name.len() == 2 {
            log::info!(
                "🎨 <{}> styles: parent_font={}px, after_defaults={}px, after_default_css={}px, after_page_css={}px, final={}px",
                elem.tag_name,
                parent_font_size,
                font_size_after_defaults,
                font_size_after_default_css,
                font_size_after_page_css,
                styles.font_size
            );
        }

        // Log pour le body
        if elem.tag_name.eq_ignore_ascii_case("body") {
            log::info!(
                "🎨 <body> styles: margin_top={}px, margin_left_auto={}, margin_right_auto={}, width_percent={:?}",
                styles.margin_top,
                styles.margin_left_auto,
                styles.margin_right_auto,
                styles.width_percent
            );
        }

        styles
    }

    fn apply_tag_defaults(&self, tag: &str, styles: &mut ComputedStyles) {
        match tag.to_lowercase().as_str() {
            "div" | "article" | "aside" | "footer" | "header" | "main" | "nav" | "section" => {
                styles.display = "block".to_string();
            }
            // Headings - basé sur le CSS par défaut de Chrome
            "h1" => {
                styles.display = "block".to_string();
                styles.font_size = self.base_font_size * 2.0; // 2em
                styles.font_weight = FontWeight::Bold;
                styles.margin_top = self.base_font_size * 2.0 * 0.67; // 0.67em relatif à font-size
                styles.margin_bottom = self.base_font_size * 2.0 * 0.67;
            }
            "h2" => {
                styles.display = "block".to_string();
                styles.font_size = self.base_font_size * 1.5; // 1.5em
                styles.font_weight = FontWeight::Bold;
                styles.margin_top = self.base_font_size * 1.5 * 0.83; // 0.83em relatif à font-size
                styles.margin_bottom = self.base_font_size * 1.5 * 0.83;
            }
            "h3" => {
                styles.display = "block".to_string();
                styles.font_size = self.base_font_size * 1.17; // 1.17em
                styles.font_weight = FontWeight::Bold;
                styles.margin_top = self.base_font_size * 1.17; // 1em relatif à font-size
                styles.margin_bottom = self.base_font_size * 1.17;
            }
            "h4" => {
                styles.display = "block".to_string();
                styles.font_size = self.base_font_size; // 1em (pas de changement)
                styles.font_weight = FontWeight::Bold;
                styles.margin_top = self.base_font_size * 1.33; // 1.33em
                styles.margin_bottom = self.base_font_size * 1.33;
            }
            "h5" => {
                styles.display = "block".to_string();
                styles.font_size = self.base_font_size * 0.83; // 0.83em
                styles.font_weight = FontWeight::Bold;
                styles.margin_top = self.base_font_size * 0.83 * 1.67; // 1.67em relatif
                styles.margin_bottom = self.base_font_size * 0.83 * 1.67;
            }
            "h6" => {
                styles.display = "block".to_string();
                styles.font_size = self.base_font_size * 0.67; // 0.67em
                styles.font_weight = FontWeight::Bold;
                styles.margin_top = self.base_font_size * 0.67 * 2.33; // 2.33em relatif
                styles.margin_bottom = self.base_font_size * 0.67 * 2.33;
            }
            "p" => {
                styles.display = "block".to_string();
                styles.margin_top = self.base_font_size; // 1em
                styles.margin_bottom = self.base_font_size;
            }
            "ul" | "ol" => {
                styles.display = "block".to_string();
                styles.margin_top = self.base_font_size;
                styles.margin_bottom = self.base_font_size;
                styles.padding_left = 40.0;
            }
            "li" => {
                styles.display = "block".to_string();
                styles.list_style_type = "disc".to_string();
            }
            "strong" | "b" => { styles.font_weight = FontWeight::Bold; }
            "em" | "i" => { styles.font_style = FontStyle::Italic; }
            "u" => { styles.text_decoration = TextDecoration::Underline; }
            "a" => {
                styles.color = RenderColor::rgb(26, 13, 171); // Bleu lien classique
                styles.text_decoration = TextDecoration::Underline;
            }
            "code" => {
                styles.background_color = RenderColor::rgb(245, 245, 245);
                styles.font_size = self.base_font_size * 0.9;
            }
            "pre" => {
                styles.display = "block".to_string();
                styles.background_color = RenderColor::rgb(245, 245, 245);
                styles.font_size = self.base_font_size * 0.9;
                styles.padding_top = 10.0;
                styles.padding_bottom = 10.0;
                styles.padding_left = 10.0;
                styles.padding_right = 10.0;
                styles.margin_top = self.base_font_size;
                styles.margin_bottom = self.base_font_size;
            }
            "blockquote" => {
                styles.display = "block".to_string();
                styles.margin_top = self.base_font_size;
                styles.margin_bottom = self.base_font_size;
                styles.margin_left = 40.0;
                styles.margin_right = 40.0;
            }
            "hr" => {
                styles.display = "block".to_string();
                styles.margin_top = 8.0;
                styles.margin_bottom = 8.0;
            }
            "script" | "style" | "head" | "title" | "meta" | "link" | "noscript" | "template" => {
                styles.display = "none".to_string();
            }
            "body" => {
                styles.display = "block".to_string();
                styles.margin_top = 8.0;
                styles.margin_bottom = 8.0;
                styles.margin_left = 8.0;
                styles.margin_right = 8.0;
            }
            "html" => {
                styles.display = "block".to_string();
            }
            _ => {}
        }
    }

    fn apply_stylesheet_styles(&self, stylesheet: &Stylesheet, tag: &str, id: Option<&str>, classes: &[&str], styles: &mut ComputedStyles) {
        for rule in &stylesheet.rules {
            for selector in &rule.selectors {
                if self.selector_matches(selector, tag, id, classes) {
                    self.apply_declarations(&rule.declarations, styles);
                }
            }
        }
    }

    fn apply_stylesheet_styles_with_parent(&self, stylesheet: &Stylesheet, tag: &str, id: Option<&str>, classes: &[&str], styles: &mut ComputedStyles, parent_font_size: f32) {
        for rule in &stylesheet.rules {
            for selector in &rule.selectors {
                if self.selector_matches(selector, tag, id, classes) {
                    self.apply_declarations_with_parent(&rule.declarations, styles, parent_font_size);
                }
            }
        }
    }

    fn selector_matches(&self, selector: &str, tag: &str, id: Option<&str>, classes: &[&str]) -> bool {
        let selector = selector.trim();
        if selector == "*" { return true; }
        if selector.starts_with('#') { return id == Some(&selector[1..]); }
        if selector.starts_with('.') { return classes.contains(&&selector[1..]); }
        selector.eq_ignore_ascii_case(tag)
    }

    fn apply_declarations(&self, declarations: &HashMap<String, CssValue>, styles: &mut ComputedStyles) {
        self.apply_declarations_with_parent(declarations, styles, styles.font_size);
    }

    fn apply_declarations_with_parent(&self, declarations: &HashMap<String, CssValue>, styles: &mut ComputedStyles, parent_font_size: f32) {
        // Capture viewport dimensions pour la closure
        let viewport_width = self.viewport_width;
        let viewport_height = self.viewport_height;
        let base_font_size = self.base_font_size;

        // Helper pour convertir une longueur CSS en pixels
        let convert_length = |size: f32, unit: &super::css_parser::LengthUnit, current_font_size: f32| -> f32 {
            use super::css_parser::LengthUnit;
            match unit {
                LengthUnit::Px => size,
                LengthUnit::Em => current_font_size * size,
                LengthUnit::Rem => base_font_size * size,
                LengthUnit::Pt => size * 1.333,
                LengthUnit::Percent => current_font_size * size / 100.0,
                LengthUnit::Vh => viewport_height * size / 100.0, // vh = % de la hauteur du viewport
                LengthUnit::Vw => viewport_width * size / 100.0,  // vw = % de la largeur du viewport
                _ => size,
            }
        };

        for (property, value) in declarations {
            match property.as_str() {
                "display" => if let CssValue::Keyword(v) = value { styles.display = v.clone(); },

                // Font properties
                "font-size" => {
                    match value {
                        CssValue::Length(size, unit) => {
                            styles.font_size = convert_length(*size, unit, parent_font_size);
                            log::debug!("📐 font-size: {}px (from {:?} {:?}, parent={}px)", styles.font_size, size, unit, parent_font_size);
                        }
                        CssValue::Keyword(kw) => {
                            styles.font_size = match kw.as_str() {
                                "xx-small" => 9.0,
                                "x-small" => 10.0,
                                "small" => 13.0,
                                "medium" => 16.0,
                                "large" => 18.0,
                                "x-large" => 24.0,
                                "xx-large" => 32.0,
                                "larger" => parent_font_size * 1.2,
                                "smaller" => parent_font_size / 1.2,
                                _ => styles.font_size,
                            };
                        }
                        CssValue::Number(n) => {
                            styles.font_size = *n;
                        }
                        _ => {}
                    }
                },
                "font-weight" => {
                    match value {
                        CssValue::Keyword(v) => {
                            styles.font_weight = if v == "bold" || v == "700" || v == "800" || v == "900" {
                                FontWeight::Bold
                            } else {
                                FontWeight::Normal
                            };
                        }
                        CssValue::Number(n) => {
                            styles.font_weight = if *n >= 700.0 { FontWeight::Bold } else { FontWeight::Normal };
                        }
                        _ => {}
                    }
                },
                "font-style" => if let CssValue::Keyword(v) = value {
                    styles.font_style = if v == "italic" || v == "oblique" {
                        FontStyle::Italic
                    } else {
                        FontStyle::Normal
                    };
                },
                "font-family" => {
                    // On ignore font-family pour l'instant car iced utilise la police par défaut
                    // mais on pourrait stocker la valeur pour utilisation future
                },

                // Color properties
                "color" => if let CssValue::Color(c) = value {
                    styles.color = RenderColor::rgba(c.r, c.g, c.b, c.a);
                },
                "background-color" | "background" => if let CssValue::Color(c) = value {
                    styles.background_color = RenderColor::rgba(c.r, c.g, c.b, c.a);
                },
                "opacity" => {
                    match value {
                        CssValue::Number(n) => {
                            styles.color.a *= n;
                            styles.background_color.a *= n;
                        }
                        _ => {}
                    }
                },

                // Margin properties - support des unités relatives et auto
                "margin" => {
                    match value {
                        CssValue::Length(m, unit) => {
                            let margin = convert_length(*m, unit, styles.font_size);
                            styles.margin_top = margin;
                            styles.margin_bottom = margin;
                            styles.margin_left = margin;
                            styles.margin_right = margin;
                            styles.margin_left_auto = false;
                            styles.margin_right_auto = false;
                        }
                        CssValue::Keyword(kw) if kw == "auto" => {
                            styles.margin_left_auto = true;
                            styles.margin_right_auto = true;
                        }
                        _ => {}
                    }
                },
                "margin-top" => {
                    match value {
                        CssValue::Length(m, unit) => {
                            styles.margin_top = convert_length(*m, unit, styles.font_size);
                            log::debug!("📐 margin-top: {}px (from {:?} {:?})", styles.margin_top, m, unit);
                        }
                        _ => {}
                    }
                },
                "margin-bottom" => {
                    match value {
                        CssValue::Length(m, unit) => {
                            styles.margin_bottom = convert_length(*m, unit, styles.font_size);
                        }
                        _ => {}
                    }
                },
                "margin-left" => {
                    match value {
                        CssValue::Length(m, unit) => {
                            styles.margin_left = convert_length(*m, unit, styles.font_size);
                            styles.margin_left_auto = false;
                            log::debug!("📐 margin-left: {}px", styles.margin_left);
                        }
                        CssValue::Keyword(kw) => {
                            if kw.eq_ignore_ascii_case("auto") {
                                styles.margin_left_auto = true;
                                log::info!("📐 margin-left: auto (keyword='{}')", kw);
                            } else {
                                log::debug!("📐 margin-left: unknown keyword '{}'", kw);
                            }
                        }
                        _ => {
                            log::debug!("📐 margin-left: unhandled value {:?}", value);
                        }
                    }
                },
                "margin-right" => {
                    match value {
                        CssValue::Length(m, unit) => {
                            styles.margin_right = convert_length(*m, unit, styles.font_size);
                            styles.margin_right_auto = false;
                            log::debug!("📐 margin-right: {}px", styles.margin_right);
                        }
                        CssValue::Keyword(kw) => {
                            if kw.eq_ignore_ascii_case("auto") {
                                styles.margin_right_auto = true;
                                log::info!("📐 margin-right: auto (keyword='{}')", kw);
                            } else {
                                log::debug!("📐 margin-right: unknown keyword '{}'", kw);
                            }
                        }
                        _ => {
                            log::debug!("📐 margin-right: unhandled value {:?}", value);
                        }
                    }
                },

                // Padding properties - support des unités relatives
                "padding" => {
                    if let CssValue::Length(p, unit) = value {
                        let padding = convert_length(*p, unit, styles.font_size);
                        styles.padding_top = padding;
                        styles.padding_bottom = padding;
                        styles.padding_left = padding;
                        styles.padding_right = padding;
                    }
                },
                "padding-top" => if let CssValue::Length(p, unit) = value {
                    styles.padding_top = convert_length(*p, unit, styles.font_size);
                },
                "padding-bottom" => if let CssValue::Length(p, unit) = value {
                    styles.padding_bottom = convert_length(*p, unit, styles.font_size);
                },
                "padding-left" => if let CssValue::Length(p, unit) = value {
                    styles.padding_left = convert_length(*p, unit, styles.font_size);
                },
                "padding-right" => if let CssValue::Length(p, unit) = value {
                    styles.padding_right = convert_length(*p, unit, styles.font_size);
                },

                // Text properties
                "text-align" => if let CssValue::Keyword(v) = value {
                    styles.text_align = match v.as_str() {
                        "center" => TextAlign::Center,
                        "right" => TextAlign::Right,
                        "justify" => TextAlign::Justify,
                        _ => TextAlign::Left,
                    };
                },
                "text-decoration" => if let CssValue::Keyword(v) = value {
                    styles.text_decoration = match v.as_str() {
                        "underline" => TextDecoration::Underline,
                        "line-through" => TextDecoration::LineThrough,
                        _ => TextDecoration::None,
                    };
                },
                "line-height" => {
                    match value {
                        CssValue::Number(n) => {
                            styles.line_height = *n;
                        }
                        CssValue::Length(l, unit) => {
                            styles.line_height = convert_length(*l, unit, styles.font_size) / styles.font_size;
                        }
                        _ => {}
                    }
                },

                // Width
                "width" => {
                    match value {
                        CssValue::Length(w, unit) => {
                            use super::css_parser::LengthUnit;
                            match unit {
                                LengthUnit::Vw => {
                                    // Stocker en pourcentage du viewport
                                    styles.width_percent = Some(*w);
                                    log::debug!("📐 width: {}vw", w);
                                }
                                LengthUnit::Percent => {
                                    styles.width_percent = Some(*w);
                                    log::debug!("📐 width: {}%", w);
                                }
                                _ => {
                                    styles.width = Some(convert_length(*w, unit, styles.font_size));
                                    log::debug!("📐 width: {}px", styles.width.unwrap());
                                }
                            }
                        }
                        CssValue::Percentage(p) => {
                            styles.width_percent = Some(*p);
                        }
                        CssValue::Keyword(kw) if kw == "auto" => {
                            styles.width = None;
                            styles.width_percent = None;
                        }
                        _ => {}
                    }
                },

                _ => {}
            }
        }
    }

    fn determine_node_type(&self, tag: &str, styles: &ComputedStyles) -> RenderNodeType {
        if styles.display == "none" { return RenderNodeType::Hidden; }
        match styles.display.as_str() {
            "block" => RenderNodeType::Block,
            "inline" => RenderNodeType::Inline,
            "none" => RenderNodeType::Hidden,
            _ => match tag.to_lowercase().as_str() {
                "div" | "p" | "h1" | "h2" | "h3" | "ul" | "ol" | "li" => RenderNodeType::Block,
                _ => RenderNodeType::Inline
            }
        }
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self { Self::new() }
}

/// Structure pour retourner le contenu rendu avec les styles du body
pub struct RenderedContent {
    pub styled_content: Vec<StyledText>,
    pub body_styles: Option<ComputedStyles>,
}

pub fn flatten_render_tree(node: &RenderNode) -> Vec<StyledText> {
    let mut result = Vec::new();
    flatten_node(node, &mut result, 0, None);
    result
}

/// Flatten avec extraction des styles du body
pub fn flatten_render_tree_with_body(node: &RenderNode) -> RenderedContent {
    let mut result = Vec::new();
    let body_styles = find_body_styles(node);
    flatten_node(node, &mut result, 0, None);
    RenderedContent {
        styled_content: result,
        body_styles,
    }
}

/// Recherche les styles du body dans l'arbre de rendu
fn find_body_styles(node: &RenderNode) -> Option<ComputedStyles> {
    if node.tag == "body" {
        return Some(node.styles.clone());
    }
    for child in &node.children {
        if let Some(styles) = find_body_styles(child) {
            return Some(styles);
        }
    }
    None
}

fn flatten_node(node: &RenderNode, result: &mut Vec<StyledText>, depth: usize, parent_href: Option<&str>) {
    // Si ce nœud est un lien <a>, utiliser son href, sinon utiliser celui du parent
    let current_href = node.href.as_deref().or(parent_href);

    match node.node_type {
        RenderNodeType::Hidden => return,
        RenderNodeType::Text => {
            if !node.text.trim().is_empty() {
                result.push(StyledText {
                    text: node.text.clone(),
                    styles: node.styles.clone(),
                    is_block: false,
                    depth,
                    href: current_href.map(|s| s.to_string()),
                });
            }
        }
        RenderNodeType::Block | RenderNodeType::ListItem => {
            if !result.is_empty() && result.last().map(|l| !l.text.ends_with('\n')).unwrap_or(false) {
                result.push(StyledText {
                    text: "\n".to_string(),
                    styles: node.styles.clone(),
                    is_block: true,
                    depth,
                    href: None,
                });
            }
            if matches!(node.node_type, RenderNodeType::ListItem) {
                result.push(StyledText {
                    text: "• ".to_string(),
                    styles: node.styles.clone(),
                    is_block: false,
                    depth,
                    href: None,
                });
            }
            for child in &node.children {
                flatten_node(child, result, depth + 1, current_href);
            }
            result.push(StyledText {
                text: "\n".to_string(),
                styles: node.styles.clone(),
                is_block: true,
                depth,
                href: None,
            });
        }
        _ => {
            for child in &node.children {
                flatten_node(child, result, depth, current_href);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StyledText {
    pub text: String,
    pub styles: ComputedStyles,
    pub is_block: bool,
    pub depth: usize,
    pub href: Option<String>,
}
