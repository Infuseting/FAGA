use layout::{LayoutBox, BoxType, Rect};
use css::{Value, Unit};
use html::{NodeType};

#[derive(Debug)]
pub enum DisplayCommand {
    SolidColor(u32, Rect),
    Text(String, Rect, u32),
}

pub type DisplayList = Vec<DisplayCommand>;

pub fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = Vec::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);
    render_text(list, layout_box);
    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    if let Some(color) = get_color(layout_box, "background") {
        list.push(DisplayCommand::SolidColor(
            color,
            layout_box.dimensions.border_box()
        ));
    }
}

fn render_text(list: &mut DisplayList, layout_box: &LayoutBox) {
    let node_opt = match layout_box.box_type {
        BoxType::BlockNode(styled_node) | BoxType::InlineNode(styled_node) => Some(styled_node.node),
        _ => None,
    };

    if let Some(node) = node_opt {
        if let NodeType::Text(ref text_content) = node.node_type {
            let color = get_color(layout_box, "color").unwrap_or(0xFF000000);

            list.push(DisplayCommand::Text(
                text_content.clone(),
                layout_box.dimensions.content,
                color
            ));
        }
    }
}

fn render_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
    let d = &layout_box.dimensions;
    let border_box = d.border_box();
}

fn get_color(layout_box: &LayoutBox, name: &str) -> Option<u32> {
    match layout_box.box_type {
        BoxType::BlockNode(node) | BoxType::InlineNode(node) => {
            match node.specified_values.get(name) {
                Some(Value::ColorValue(r, g, b, a)) => {
                    let color = ((*a as u32) << 24) | ((*r as u32) << 16) | ((*g as u32) << 8) | (*b as u32);
                    Some(color)
                },
                Some(Value::Keyword(k)) => match k.as_str() {
                    "black" => Some(0xFF000000),
                    "white" => Some(0xFFFFFFFF),
                    "red" => Some(0xFFFF0000),
                    "blue" => Some(0xFF0000FF),
                    "grey" | "gray" => Some(0xFF808080),
                    _ => None,
                },
                _ => None
            }
        }
        _ => None
    }
}

trait BoxDimensions {
    fn border_box(&self) -> Rect;
}

impl BoxDimensions for layout::Dimensions {
    fn border_box(&self) -> Rect {
        Rect {
            x: self.content.x - self.padding.left - self.border.left,
            y: self.content.y - self.padding.top - self.border.top,
            width: self.content.width + self.padding.left + self.padding.right + self.border.left + self.border.right,
            height: self.content.height + self.padding.top + self.padding.bottom + self.border.top + self.border.bottom,
        }
    }
}