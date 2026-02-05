use std::collections::HashMap;


/*
    Node represents an element in the DOM tree. It can be either a text node or an element node.
    - For text nodes, the `node_type` will be `NodeType::Text`
    - For element nodes, the `node_type` will be `NodeType::Element`, which contains the tag name and attributes.
    Each node can have zero or more child nodes, which are stored in the `children` vector.

*/
#[derive(Debug, Clone)]
pub struct Node {
    pub children: Vec<Node>,
    pub node_type: NodeType,
}

/*
    NodeType is an enum that represents the type of a node in the DOM tree. It can be either:
    - Text: A text node, which contains a string of text.
    - Element: An element node, which contains an `ElementData` struct with the tag name and attributes.
*/
#[derive(Debug, Clone)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
}

/*
    ElementData is a struct that represents the data of an element node in the DOM tree. It contains:
    - tag_name: The name of the HTML tag (e.g., "div", "p", "span").
    - attributes: A HashMap of attribute names and their corresponding values (e.g., "id" -> "header", "class" -> "dark").
*/
#[derive(Debug, Clone)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
}
/*
    Create text node with the given data.
   
    @param data: The text content of the node.
    @return A Node representing a text node with the given data.
*/
pub fn text(data: String) -> Node {
    Node {
        children: vec![],
        node_type: NodeType::Text(data),
    }
}

/* 
    Create an element node with the given name, attributes, and children.
    
    @param name: The tag name of the element (e.g., "div", "p").
    @param attrs: A HashMap of attribute names and their corresponding values (e.g., "id" -> "header", "class" -> "dark").
    @param children: A vector of child nodes that are contained within this element.
    @return A Node representing an element node with the specified name, attributes, and children.
*/
pub fn elem(name: String, attrs: HashMap<String, String>, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs,
        }),
    }
}


/* 
    Parser is a struct that holds the state of the HTML parser. It contains:
    - pos: The current position in the input string.
    - input: The entire HTML source code as a string.
*/
pub struct Parser {
    pos: usize,
    input: String,
}

/* 
    The Parser struct provides methods to parse an HTML string and construct a DOM tree. It includes methods to:
    - Create a new parser with the given input string.
    - Get the next character in the input without consuming it.
    - Check if the input starts with a specific string at the current position.
    - Check if the end of the input has been reached.
    - Consume the next character and advance the position.
    - Consume characters while a certain condition is true (e.g., while they are whitespace).
    - Parse nodes, elements, text, tag names, attributes, and attribute values from the input string.
*/
impl Parser {
    /* 
        Create instance of Parser with the given input string.
        @param input: The HTML source code to be parsed.
        @return A new instance of the Parser struct initialized with the input string and position set to 0.
    */
    pub fn new(input: String) -> Self {
        Self { pos: 0, input }
    }
    /* 
        Get the next character in the input string without consuming it. This method looks at the current position and returns the character at that position, or a default character if the end of the input has been reached.
        @return The next character in the input string, or a default character if the end of the input has been reached.
    */
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or_default()
    }
    
    /* 
        Get the next characters in the input string and check if they match a specific string. This method checks if the substring starting at the current position matches the provided string `s`.
        @param s: The string to compare against the next characters in the input.
        @return `true` if the input starts with the string `s` at the current position, `false` otherwise.
    */
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }
    
    /* 
        Check if the end of the input string has been reached. This method compares the current position with the length of the input string to determine if there are more characters to parse.
        @return `true` if the current position is greater than or equal to the length of the input string (indicating that the end has been reached), `false` otherwise.
    */
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
    
    /* 
        Consume next character in the input string and advance the position. This method retrieves the next character at the current position, advances the position by the length of that character (to account for multi-byte characters), and returns the consumed character.
        @return The character that was consumed from the input string.
    */
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    /* 
        Consume characters from the input string while a certain condition is true. This method takes a closure `test` that defines the condition for consuming characters. It continues to consume characters and append them to a result string as long as the end of the input has not been reached and the next character satisfies the condition defined by `test`.
        @param test: A closure that takes a character as input and returns `true` if the character should be consumed, `false` otherwise.
        @return A string containing all the characters that were consumed while the condition defined by `test` was satisfied.
    */
    fn consume_while<F> (&mut self, test: F) -> String
    where F: Fn(char) -> bool {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    /* 
        Delete whitespace characters from the input string. This method uses the `consume_while` method with a condition that checks if a character is a whitespace character (using `char::is_whitespace`). It continues to consume characters until it encounters a non-whitespace character or reaches the end of the input.
        
        @return None. 
    */
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }
    
    /* 
        Parse nodes from the input string and construct a vector of `Node` objects representing the DOM tree. This method continues to parse nodes until it reaches the end of the input or encounters a closing tag (indicated by `</`). It uses the `parse_node` method to parse individual nodes and appends them to a vector, which is returned at the end.
        @return A vector of `Node` objects representing the parsed DOM tree.
    */
    pub fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        while !self.eof() {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }

    /* 
        Parse a single node from the input string. This method checks the next character to determine if it is the start of an element (indicated by `<`) or a text node. If it is an element, it calls the `parse_element` method to parse the element and its children. If it is not an element, it calls the `parse_text` method to parse a text node.
        @return A `Node` object representing the parsed node (either an element or a text node).
    */
    pub fn parse_node(&mut self) -> Node {
        if self.next_char() == '<' {
            self.parse_element()
        } else {
            self.parse_text()
        }
    }

    /* 
        Parse a text node from the input string. This method uses the `consume_while` method to consume characters until it encounters a `<` character, which indicates the start of an element. The consumed characters are returned as a text node using the `text` function.
        @return A `Node` object representing a text node with the consumed text content.
    */
    fn parse_text(&mut self) -> Node {
        text(self.consume_while(|c| c != '<'))
    }
    
    /* 
        Parse an element node from the input string. This method assumes that the current position is at the start of an element (indicated by `<`). It parses the tag name, attributes, and child nodes of the element. It also checks for the corresponding closing tag to ensure that the element is properly closed. The parsed element is returned as a `Node` object using the `elem` function.
        @return A `Node` object representing the parsed element with its tag name, attributes, and child nodes.
    */
    fn parse_element(&mut self) -> Node {
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        assert!(self.consume_char() == '>');

        let children = self.parse_nodes();

        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        elem(tag_name, attrs, children)
    }

    /* 
        Parse a tag name from the input string. This method uses the `consume_while` method to consume characters that are valid in a tag name (letters and digits). The consumed characters are returned as a string representing the tag name.
        @return A string representing the parsed tag name.
    */
    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => true,
            _ => false,
        })
    }

    /* 
        Parse attributes from the input string. This method continues to parse attributes until it encounters a `>` character, which indicates the end of the element's opening tag. It uses the `parse_attr` method to parse individual attributes and stores them in a `HashMap`, which is returned at the end.
        @return A `HashMap` containing attribute names and their corresponding values for the parsed element.
    */
    fn parse_attributes(&mut self) -> HashMap<String, String> {
        self.consume_whitespace();
        let mut attributes = HashMap::new();
        loop {
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
            self.consume_whitespace();
        }
        attributes
    }
    
    /* 
        Parse a single attribute from the input string. This method assumes that the current position is at the start of an attribute (after any whitespace). It parses the attribute name, expects an `=` character, and then parses the attribute value (which should be enclosed in quotes). The parsed attribute name and value are returned as a tuple.
        @return A tuple containing the attribute name and its corresponding value for the parsed attribute.
    */
    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        (name, value)
    }
    /* 
        Parse an attribute value from the input string. This method assumes that the current position is at the start of an attribute value (after the `=` character). It expects the value to be enclosed in either double quotes (`"`) or single quotes (`'`). It consumes the opening quote, then uses the `consume_while` method to consume characters until it encounters the matching closing quote. The consumed characters are returned as a string representing the attribute value.
        @return A string representing the parsed attribute value.
    */
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        value
    }
}

/* 
    Parse an HTML source string and construct a DOM tree represented by a `Node` object. This function creates a new instance of the `Parser` struct with the provided source string, calls the `parse_nodes` method to parse the nodes from the input, and returns either a single node (if there is only one) or a root node containing all parsed nodes as children.
    @param source: The HTML source code to be parsed.
    @return A `Node` object representing the root of the parsed DOM tree.
*/
pub fn parse(source: String) -> Node {
    let mut parser = Parser::new(source);
    let nodes = parser.parse_nodes();
    if nodes.len() == 1 {
        nodes.into_iter().next().unwrap()
    } else {
        elem("html".to_string(), HashMap::new(), nodes)
    }
}


