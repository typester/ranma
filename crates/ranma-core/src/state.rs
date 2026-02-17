use std::collections::HashMap;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum NodeType {
    Item,
    Row,
    Column,
    Box,
}

#[derive(Debug, Clone, Default, uniffi::Record)]
pub struct NodeStyle {
    pub background_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<f32>,
    pub corner_radius: Option<f32>,
    pub padding_left: Option<f32>,
    pub padding_right: Option<f32>,
    pub padding_top: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub shadow_color: Option<String>,
    pub shadow_radius: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub gap: Option<f32>,
    pub margin_left: Option<f32>,
    pub margin_right: Option<f32>,
    pub margin_top: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub notch_align: Option<String>,
    pub align_items: Option<String>,
    pub justify_content: Option<String>,
    pub hover_background_color: Option<String>,
    pub hover_label_color: Option<String>,
    pub hover_icon_color: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct BarNode {
    pub name: String,
    pub node_type: NodeType,
    pub parent: Option<String>,
    pub position: i32,
    pub display: u32,
    pub style: NodeStyle,
    pub label: Option<String>,
    pub label_color: Option<String>,
    pub icon: Option<String>,
    pub icon_color: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<String>,
    pub font_family: Option<String>,
    pub on_click: Option<String>,
}

#[derive(Debug, Default)]
pub struct BarState {
    nodes: HashMap<u32, Vec<BarNode>>,
}

impl BarState {
    pub fn add_node(&mut self, node: BarNode) -> Result<(), String> {
        if let Some(ref parent_name) = node.parent {
            let parent = self.find_node_ref(parent_name);
            match parent {
                Some(p) => {
                    if matches!(p.node_type, NodeType::Item) {
                        return Err(format!(
                            "'{}' is an item and cannot have children",
                            parent_name
                        ));
                    }
                }
                None => return Err(format!("parent '{}' not found", parent_name)),
            }
        }

        let display_nodes = self.nodes.entry(node.display).or_default();
        if display_nodes.iter().any(|n| n.name == node.name) {
            return Err(format!(
                "node '{}' already exists on display {}",
                node.name, node.display
            ));
        }
        display_nodes.push(node);
        display_nodes.sort_by_key(|n| n.position);
        Ok(())
    }

    pub fn remove_node(&mut self, name: &str) -> Result<BarNode, String> {
        for nodes in self.nodes.values_mut() {
            if let Some(pos) = nodes.iter().position(|n| n.name == name) {
                let node = nodes.remove(pos);
                if !matches!(node.node_type, NodeType::Item) {
                    nodes.retain(|n| n.parent.as_deref() != Some(name));
                }
                return Ok(node);
            }
        }
        Err(format!("node '{}' not found", name))
    }

    pub fn set_properties(
        &mut self,
        name: &str,
        properties: &HashMap<String, String>,
    ) -> Result<BarNode, String> {
        let new_display = properties
            .get("display")
            .map(|v| {
                v.parse::<u32>()
                    .map_err(|_| format!("invalid display: {}", v))
            })
            .transpose()?;

        let (current_display, idx) = self.find_node(name)?;

        if let Some(target_display) = new_display
            && target_display != current_display
        {
            let mut node = self.nodes.get_mut(&current_display).unwrap().remove(idx);
            node.display = target_display;
            Self::apply_properties(&mut node, properties)?;
            let display_nodes = self.nodes.entry(target_display).or_default();
            display_nodes.push(node.clone());
            display_nodes.sort_by_key(|n| n.position);
            return Ok(node);
        }

        let nodes = self.nodes.get_mut(&current_display).unwrap();
        let node = &mut nodes[idx];
        Self::apply_properties(node, properties)?;
        let updated = node.clone();
        nodes.sort_by_key(|n| n.position);
        Ok(updated)
    }

    fn apply_properties(
        node: &mut BarNode,
        properties: &HashMap<String, String>,
    ) -> Result<(), String> {
        fn optional_str(value: &str) -> Option<String> {
            if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        }

        fn parse_f32(key: &str, value: &str) -> Result<f32, String> {
            value
                .parse()
                .map_err(|_| format!("invalid {}: {}", key, value))
        }

        // Pass 1: apply shorthands (all → directional) so specifics can override
        if let Some(v) = properties.get("padding") {
            let val = Some(parse_f32("padding", v)?);
            node.style.padding_left = val;
            node.style.padding_right = val;
            node.style.padding_top = val;
            node.style.padding_bottom = val;
        }
        if let Some(v) = properties.get("padding_horizontal") {
            let val = Some(parse_f32("padding_horizontal", v)?);
            node.style.padding_left = val;
            node.style.padding_right = val;
        }
        if let Some(v) = properties.get("padding_vertical") {
            let val = Some(parse_f32("padding_vertical", v)?);
            node.style.padding_top = val;
            node.style.padding_bottom = val;
        }
        if let Some(v) = properties.get("margin") {
            let val = Some(parse_f32("margin", v)?);
            node.style.margin_left = val;
            node.style.margin_right = val;
            node.style.margin_top = val;
            node.style.margin_bottom = val;
        }
        if let Some(v) = properties.get("margin_horizontal") {
            let val = Some(parse_f32("margin_horizontal", v)?);
            node.style.margin_left = val;
            node.style.margin_right = val;
        }
        if let Some(v) = properties.get("margin_vertical") {
            let val = Some(parse_f32("margin_vertical", v)?);
            node.style.margin_top = val;
            node.style.margin_bottom = val;
        }

        // Pass 2: apply specific properties (override shorthands)
        for (key, value) in properties {
            match key.as_str() {
                "label" => node.label = optional_str(value),
                "label_color" => node.label_color = optional_str(value),
                "icon" => node.icon = optional_str(value),
                "icon_color" => node.icon_color = optional_str(value),
                "font_weight" => node.font_weight = optional_str(value),
                "font_family" => node.font_family = optional_str(value),
                "parent" => node.parent = optional_str(value),
                "background_color" => node.style.background_color = optional_str(value),
                "border_color" => node.style.border_color = optional_str(value),
                "shadow_color" => node.style.shadow_color = optional_str(value),
                "notch_align" => node.style.notch_align = optional_str(value),
                "align_items" => node.style.align_items = optional_str(value),
                "justify_content" => node.style.justify_content = optional_str(value),
                "hover_background_color" => node.style.hover_background_color = optional_str(value),
                "hover_label_color" => node.style.hover_label_color = optional_str(value),
                "hover_icon_color" => node.style.hover_icon_color = optional_str(value),
                "on_click" => node.on_click = optional_str(value),
                "border_width" => {
                    node.style.border_width = Some(parse_f32("border_width", value)?);
                }
                "corner_radius" => {
                    node.style.corner_radius = Some(parse_f32("corner_radius", value)?);
                }
                "padding_left" => {
                    node.style.padding_left = Some(parse_f32("padding_left", value)?);
                }
                "padding_right" => {
                    node.style.padding_right = Some(parse_f32("padding_right", value)?);
                }
                "padding_top" => {
                    node.style.padding_top = Some(parse_f32("padding_top", value)?);
                }
                "padding_bottom" => {
                    node.style.padding_bottom = Some(parse_f32("padding_bottom", value)?);
                }
                "shadow_radius" => {
                    node.style.shadow_radius = Some(parse_f32("shadow_radius", value)?);
                }
                "width" => {
                    node.style.width = if value.is_empty() {
                        None
                    } else {
                        Some(parse_f32("width", value)?)
                    };
                }
                "height" => {
                    node.style.height = Some(parse_f32("height", value)?);
                }
                "gap" => {
                    node.style.gap = if value.is_empty() {
                        None
                    } else {
                        Some(parse_f32("gap", value)?)
                    };
                }
                "margin_left" => {
                    node.style.margin_left = Some(parse_f32("margin_left", value)?);
                }
                "margin_right" => {
                    node.style.margin_right = Some(parse_f32("margin_right", value)?);
                }
                "margin_top" => {
                    node.style.margin_top = Some(parse_f32("margin_top", value)?);
                }
                "margin_bottom" => {
                    node.style.margin_bottom = Some(parse_f32("margin_bottom", value)?);
                }
                "font_size" => {
                    node.font_size = Some(parse_f32("font_size", value)?);
                }
                "position" => {
                    node.position = value
                        .parse()
                        .map_err(|_| format!("invalid position: {}", value))?;
                }
                "display" => {}
                // Shorthands handled in pass 1
                "padding" | "padding_horizontal" | "padding_vertical" | "margin"
                | "margin_horizontal" | "margin_vertical" => {}
                _ => return Err(format!("unknown property: {}", key)),
            }
        }
        Ok(())
    }

    fn find_node(&self, name: &str) -> Result<(u32, usize), String> {
        for (&display, nodes) in &self.nodes {
            if let Some(pos) = nodes.iter().position(|n| n.name == name) {
                return Ok((display, pos));
            }
        }
        Err(format!("node '{}' not found", name))
    }

    fn find_node_ref(&self, name: &str) -> Option<&BarNode> {
        for nodes in self.nodes.values() {
            if let Some(node) = nodes.iter().find(|n| n.name == name) {
                return Some(node);
            }
        }
        None
    }

    pub fn get_nodes(&self) -> Vec<BarNode> {
        self.nodes.values().flatten().cloned().collect()
    }

    pub fn get_nodes_for_display(&self, display: u32) -> Vec<BarNode> {
        self.nodes.get(&display).cloned().unwrap_or_default()
    }
}
