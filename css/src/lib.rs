use std::collections::HashMap;
use html::{Node, NodeType, ElementData};


/*
    StyledNode represents a node in the styled tree, which is a combination of the DOM tree and the CSS styles applied to each node. It contains a reference to the original DOM node, a map of specified CSS property values for that node, and a list of child StyledNodes representing the styled children of the original DOM node. This structure allows us to easily access both the content and the styling information for each node when we later perform layout and rendering.
*/
#[derive(Debug)]
pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

/*
    This type represents a mapping of CSS property names to their corresponding values. It is used to store the specified values for each node in the styled tree. The keys are strings representing the CSS property names (e.g., "color", "margin"), and the values are of type Value, which can represent different types of CSS values (e.g., keywords, lengths, colors).
*/
pub type PropertyMap = HashMap<String, Value>;

/*
    This module defines the structures and functions for parsing CSS stylesheets. It includes the main data structures for representing stylesheets, rules, selectors, declarations, and values, as well as a Parser struct that implements the logic for parsing a CSS stylesheet from a string input.
*/
#[derive(Debug)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}


/*
    Rule represents a CSS rule, which consists of a list of selectors and a list of declarations. For example, in the CSS rule "h1, h2 { color: red; }", the selectors would be "h1" and "h2", and the declaration would be "color: red".

*/
#[derive(Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}


/*
    Selector represents a CSS selector, which can be a simple selector (like "div", "#id", ".class") or more complex selectors (like "div > p", "a:hover"). For simplicity, we only implement simple selectors here.
*/
#[derive(Debug)]
pub enum Selector {
    Simple(SimpleSelector),
}
/*
    SimpleSelector represents a basic CSS selector, which can include a tag name, an ID, and multiple classes. For example, the selector "div#main.content" would have a tag_name of "div", an id of "main", and a class vector containing "content".
 */
#[derive(Debug)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}


/*
    Declaration represents a CSS declaration, which consists of a property name and a value. For example, in the declaration "color: red;", the name would be "color" and the value would be a Value::Keyword("red").
*/
#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

/*
    Value represents the value of a CSS declaration. It can be a keyword (like "red", "blue"), a length (like "10px"), or a color value (represented as RGBA). For simplicity, we only implement a few keywords and length units here.
*/
#[derive(Debug, Clone)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(u8, u8, u8, u8),
}


/*
    Unit represents the unit of a length value in CSS. For example, "px" for pixels. In this implementation, we only support pixels, but in a full implementation, you would also want to support other units like "em", "rem", "%", etc.
*/
#[derive(Debug, Clone)]
pub enum Unit {
    Px,
}

/*
    Parser is responsible for parsing a CSS stylesheet from a string input. It maintains the current position in the input string and provides methods to consume characters, parse rules, selectors, declarations, and values. The main entry point is the parse_stylesheet method, which returns a Stylesheet struct representing the parsed CSS.
*/
#[derive(Debug)]
pub struct Parser {
    pos: usize,
    input: String,
}


/*
    Parser implementation provides methods to parse a CSS stylesheet. It includes methods to consume characters, parse rules, selectors, declarations, and values. The parse_stylesheet method is the main entry point for parsing a CSS stylesheet from a string input.
*/
impl Parser {

    /*
        constructor for the Parser struct, which takes a string input representing the CSS stylesheet to be parsed. It initializes the position to 0 and stores the input string in the struct.

        @Param input: A string containing the CSS stylesheet to be parsed.
        @Returns: A new instance of the Parser struct initialized with the provided input string.
    */
    pub fn new(input: String) -> Self {
        Parser { pos: 0, input }
    }

    /*
        checks if the end of the input string has been reached. It compares the current position with the length of the input string and returns true if the position is greater than or equal to the length, indicating that there are no more characters to parse.

        @Returns: A boolean value indicating whether the end of the input string has been reached (true) or not (false).
    */
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /*
        gets the next character in the input string without consuming it. It uses the current position to slice the input string and retrieves the next character using the chars() method. If there are no more characters to retrieve, it returns a default character (space).

        @Returns: The next character in the input string, or a default character if there are no more characters to retrieve.
     */
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or_default()
    }

    /*
        consumes the next character in the input string and advances the position. It uses the char_indices() method to get the current character and the next character's position. The current character is returned, and the position is updated to point to the next character.

        @Returns: The character that was consumed from the input string.
    */
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    /*
        consumes characters from the input string while a given test function returns true. It takes a closure (test) as an argument, which is called for each character to determine whether it should be consumed. The method continues to consume characters until the end of the input string is reached or the test function returns false. The consumed characters are collected into a result string, which is returned at the end.

        @Param test: A closure that takes a character as input and returns a boolean indicating whether the character should be consumed.
        @Returns: A string containing the characters that were consumed while the test function returned true.
    */
    fn consume_while<F>(&mut self, test: F) -> String where F: Fn(char) -> bool {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    /*
        consumes whitespace characters from the input string. It uses the consume_while method with the char::is_whitespace function as the test to consume all consecutive whitespace characters. This is useful for skipping over spaces, tabs, and other whitespace characters in the CSS input.
    */
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }
    /*

    */
    pub fn parse_stylesheet(&mut self) -> Stylesheet {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() { break; }
            rules.push(self.parse_rule());
        }
        Stylesheet { rules }
    }

    fn parse_rule(&mut self) -> Rule {
        let mut selectors = Vec::new();
        loop {
            selectors.push(self.parse_simple_selector());
            self.consume_whitespace();
            match self.next_char() {
                ',' => { self.consume_char(); self.consume_whitespace(); },
                '{' => break,
                _ => break,
            }
        }

        let mut declarations = Vec::new();
        assert_eq!(self.consume_char(), '{');
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }

        Rule { selectors, declarations }
    }

    fn parse_simple_selector(&mut self) -> Selector {
        let mut selector = SimpleSelector { tag_name: None, id: None, class: Vec::new() };

        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }

        }
        Selector::Simple(selector)
    }

    fn parse_declaration(&mut self) -> Declaration {
        let property_name = self.parse_identifier();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ';');
        Declaration { name: property_name, value }
    }

    fn parse_value(&mut self) -> Value {
        let s = self.parse_identifier();
        if s == "red" { return Value::ColorValue(255, 0, 0, 255); }
        if s == "blue" { return Value::ColorValue(0, 0, 255, 255); }
        if s == "black" { return Value::ColorValue(0, 0, 0, 255); }
        if s == "white" { return Value::ColorValue(255, 255, 255, 255); }

        if let Ok(num) = s.trim_end_matches("px").parse::<f32>() {
            return Value::Length(num, Unit::Px);
        }

        Value::Keyword(s)
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }
}

fn valid_identifier_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}

pub fn parse(source: String) -> Stylesheet {
    let mut parser = Parser::new(source);
    parser.parse_stylesheet()
}


fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {

    if let Some(ref tag_name) = selector.tag_name {
        if tag_name != &elem.tag_name {
            return false;
        }
    }

    if let Some(ref id) = selector.id {
        match elem.attributes.get("id") {
            Some(elem_id) if elem_id == id => {},
            _ => return false,
        }
    }

    if !selector.class.is_empty() {
        match elem.attributes.get("class") {
            Some(class_str) => {
                let elem_classes: Vec<&str> = class_str.split_whitespace().collect();
                for required_class in &selector.class {
                    if !elem_classes.contains(&required_class.as_str()) {
                        return false;
                    }
                }
            },
            None => return false,
        }
    }

    true
}

fn matches(elem: &ElementData, rule: &Rule) -> bool {
    // Si *un* des sélecteurs de la règle matche, c'est bon (ex: h1, h2, h3 { ... })
    rule.selectors.iter().any(|s| match s {
        Selector::Simple(simple) => matches_simple_selector(elem, simple)
    })
}

fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();

    // On parcourt toutes les règles du CSS
    for rule in &stylesheet.rules {
        if matches(elem, rule) {
            // Si ça matche, on applique les déclarations
            for declaration in &rule.declarations {
                values.insert(declaration.name.clone(), declaration.value.clone());
            }
        }
    }
    values
}

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    let specified_values = match root.node_type {
        NodeType::Element(ref elem_data) => specified_values(elem_data, stylesheet),
        NodeType::Text(_) => HashMap::new(),
    };

    let children = root.children.iter()
        .map(|child| style_tree(child, stylesheet))
        .collect();

    StyledNode {
        node: root,
        specified_values,
        children,
    }
}