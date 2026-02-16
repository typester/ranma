use std::collections::HashMap;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum NodeType {
    Item,
    Container,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct NodeStyle {
    pub background_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<f32>,
    pub corner_radius: Option<f32>,
    pub padding_left: Option<f32>,
    pub padding_right: Option<f32>,
    pub shadow_color: Option<String>,
    pub shadow_radius: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub gap: Option<f32>,
    pub notch_align: Option<String>,
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            background_color: None,
            border_color: None,
            border_width: None,
            corner_radius: None,
            padding_left: None,
            padding_right: None,
            shadow_color: None,
            shadow_radius: None,
            width: None,
            height: None,
            gap: None,
            notch_align: None,
        }
    }
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
                    if !matches!(p.node_type, NodeType::Container) {
                        return Err(format!("'{}' is not a container", parent_name));
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
                if matches!(node.node_type, NodeType::Container) {
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

        if let Some(target_display) = new_display {
            if target_display != current_display {
                let mut node = self.nodes.get_mut(&current_display).unwrap().remove(idx);
                node.display = target_display;
                Self::apply_properties(&mut node, properties)?;
                let display_nodes = self.nodes.entry(target_display).or_default();
                display_nodes.push(node.clone());
                display_nodes.sort_by_key(|n| n.position);
                return Ok(node);
            }
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
            if value.is_empty() { None } else { Some(value.to_string()) }
        }

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
                "border_width" => {
                    node.style.border_width = Some(
                        value
                            .parse()
                            .map_err(|_| format!("invalid border_width: {}", value))?,
                    );
                }
                "corner_radius" => {
                    node.style.corner_radius = Some(
                        value
                            .parse()
                            .map_err(|_| format!("invalid corner_radius: {}", value))?,
                    );
                }
                "padding_left" => {
                    node.style.padding_left = Some(
                        value
                            .parse()
                            .map_err(|_| format!("invalid padding_left: {}", value))?,
                    );
                }
                "padding_right" => {
                    node.style.padding_right = Some(
                        value
                            .parse()
                            .map_err(|_| format!("invalid padding_right: {}", value))?,
                    );
                }
                "shadow_radius" => {
                    node.style.shadow_radius = Some(
                        value
                            .parse()
                            .map_err(|_| format!("invalid shadow_radius: {}", value))?,
                    );
                }
                "width" => {
                    node.style.width = if value.is_empty() {
                        None
                    } else {
                        Some(
                            value
                                .parse()
                                .map_err(|_| format!("invalid width: {}", value))?,
                        )
                    };
                }
                "height" => {
                    node.style.height = Some(
                        value
                            .parse()
                            .map_err(|_| format!("invalid height: {}", value))?,
                    );
                }
                "gap" => {
                    node.style.gap = if value.is_empty() {
                        None
                    } else {
                        Some(
                            value
                                .parse()
                                .map_err(|_| format!("invalid gap: {}", value))?,
                        )
                    };
                }
                "font_size" => {
                    node.font_size = Some(
                        value
                            .parse()
                            .map_err(|_| format!("invalid font_size: {}", value))?,
                    );
                }
                "position" => {
                    node.position = value
                        .parse()
                        .map_err(|_| format!("invalid position: {}", value))?;
                }
                "display" => {}
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
