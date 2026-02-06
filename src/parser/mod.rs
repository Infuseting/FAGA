pub mod html_parser;
pub mod css_parser;
pub mod dom;
pub mod renderer;

pub use html_parser::HtmlParser;
pub use renderer::{HtmlRenderer, StyledText, flatten_render_tree_with_body};
