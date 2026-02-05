use css::{StyledNode, Value, Unit};
use std::default::Default;

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

impl Default for Dimensions {
    fn default() -> Self {
        Dimensions {
            content: Rect::default(),
            padding: EdgeSizes::default(),
            border: EdgeSizes::default(),
            margin: EdgeSizes::default(),
        }
    }
}

#[derive(Debug)]
pub struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    pub box_type: BoxType<'a>,
    pub children: Vec<LayoutBox<'a>>,
}

#[derive(Debug)]
pub enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}

impl<'a> LayoutBox<'a> {
    pub fn new(box_type: BoxType<'a>) -> Self {
        LayoutBox {
            box_type,
            dimensions: Default::default(),
            children: Vec::new(),
        }
    }

    fn get_style_node(&self) -> &'a StyledNode<'a> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block has no style node"),
        }
    }

    fn property(&self, name: &str) -> Option<Value> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => {
                node.specified_values.get(name).cloned()
            }
            BoxType::AnonymousBlock => None,
        }
    }
    fn lookup(&self, name: &str, name_fallback: &str, default: f32) -> f32 {
        if let Some(Value::Length(v, Unit::Px)) = self.property(name) { v }
        else if let Some(Value::Length(v, Unit::Px)) = self.property(name_fallback) { v }
        else { default }
    }
}

pub fn layout_tree<'a>(node: &'a StyledNode<'a>, containing_block: Dimensions) -> LayoutBox<'a> {
    let mut root = LayoutBox::new(BoxType::BlockNode(node));
    calculate_width(&mut root, &containing_block);
    root.dimensions.content.x = containing_block.content.x + root.dimensions.margin.left + root.dimensions.border.left + root.dimensions.padding.left;
    root.dimensions.content.y = containing_block.content.y + root.dimensions.margin.top + root.dimensions.border.top + root.dimensions.padding.top;
    let mut child_y = root.dimensions.content.y;

    for child in &node.children {
        let mut parent_dims = root.dimensions.clone();
        parent_dims.content.height = 0.0;
        parent_dims.content.y = child_y;

        let child_box = layout_tree(child, parent_dims);
        child_y += child_box.dimensions.margin.box_height()
            + child_box.dimensions.content.height;

        root.children.push(child_box);
    }

    root.dimensions.content.height = child_y - root.dimensions.content.y;

    if let Some(Value::Length(h, Unit::Px)) = root.property("height") {
        root.dimensions.content.height = h;
    }

    root
}

fn calculate_width(layout_box: &mut LayoutBox, containing_block: &Dimensions) {
    let style = layout_box.get_style_node();

    let zero = Value::Length(0.0, Unit::Px);
    let auto = Value::Keyword("auto".to_string());

    let width = layout_box.property("width").unwrap_or(auto.clone());

    let margin_left = layout_box.property("margin-left").unwrap_or(zero.clone());
    let margin_right = layout_box.property("margin-right").unwrap_or(zero.clone());

    let total_width = containing_block.content.width;

    if let Value::Keyword(s) = width {
        if s == "auto" {
            let ml = to_px(margin_left.clone());
            let mr = to_px(margin_right.clone());
            layout_box.dimensions.content.width = total_width - ml - mr;
        }
    } else {
        layout_box.dimensions.content.width = to_px(width);
    }

    layout_box.dimensions.margin.left = to_px(margin_left);
    layout_box.dimensions.margin.right = to_px(margin_right);
}

fn to_px(value: Value) -> f32 {
    match value {
        Value::Length(v, Unit::Px) => v,
        _ => 0.0,
    }
}

impl EdgeSizes {
    fn box_height(&self) -> f32 { self.top + self.bottom }
}