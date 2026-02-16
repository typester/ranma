use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::state::BarNode;

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    Add {
        name: String,
        #[serde(default)]
        node_type: Option<String>,
        #[serde(default)]
        parent: Option<String>,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        label_color: Option<String>,
        #[serde(default)]
        icon: Option<String>,
        #[serde(default)]
        icon_color: Option<String>,
        #[serde(default)]
        background_color: Option<String>,
        #[serde(default)]
        border_color: Option<String>,
        #[serde(default)]
        border_width: Option<f32>,
        #[serde(default)]
        corner_radius: Option<f32>,
        #[serde(default)]
        padding_left: Option<f32>,
        #[serde(default)]
        padding_right: Option<f32>,
        #[serde(default)]
        shadow_color: Option<String>,
        #[serde(default)]
        shadow_radius: Option<f32>,
        #[serde(default)]
        height: Option<f32>,
        #[serde(default)]
        font_size: Option<f32>,
        #[serde(default)]
        font_weight: Option<String>,
        #[serde(default)]
        font_family: Option<String>,
        #[serde(default)]
        notch_align: Option<String>,
        #[serde(default)]
        position: Option<i32>,
        #[serde(default)]
        display: Option<u32>,
    },
    Set {
        name: String,
        properties: HashMap<String, String>,
    },
    Remove {
        name: String,
    },
    Query {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        display: Option<u32>,
    },
    Displays,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Response {
    Ok,
    Error { message: String },
    QueryResult { nodes: Vec<NodeDto> },
    DisplayList { displays: Vec<DisplayDto> },
}

#[derive(Debug, Serialize)]
pub struct NodeDto {
    pub name: String,
    pub node_type: String,
    pub parent: Option<String>,
    pub position: i32,
    pub display: u32,
    pub label: Option<String>,
    pub label_color: Option<String>,
    pub icon: Option<String>,
    pub icon_color: Option<String>,
    pub background_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<f32>,
    pub corner_radius: Option<f32>,
    pub padding_left: Option<f32>,
    pub padding_right: Option<f32>,
    pub shadow_color: Option<String>,
    pub shadow_radius: Option<f32>,
    pub height: Option<f32>,
    pub notch_align: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<String>,
    pub font_family: Option<String>,
}

impl From<BarNode> for NodeDto {
    fn from(node: BarNode) -> Self {
        NodeDto {
            name: node.name,
            node_type: match node.node_type {
                crate::state::NodeType::Item => "item".to_string(),
                crate::state::NodeType::Container => "container".to_string(),
            },
            parent: node.parent,
            position: node.position,
            display: node.display,
            label: node.label,
            label_color: node.label_color,
            icon: node.icon,
            icon_color: node.icon_color,
            background_color: node.style.background_color,
            border_color: node.style.border_color,
            border_width: node.style.border_width,
            corner_radius: node.style.corner_radius,
            padding_left: node.style.padding_left,
            padding_right: node.style.padding_right,
            shadow_color: node.style.shadow_color,
            shadow_radius: node.style.shadow_radius,
            height: node.style.height,
            notch_align: node.style.notch_align,
            font_size: node.font_size,
            font_weight: node.font_weight,
            font_family: node.font_family,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DisplayDto {
    pub id: u32,
    pub name: String,
    pub is_main: bool,
}
