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
    pub display_explicit: bool,
    pub style: NodeStyle,
    pub label: Option<String>,
    pub label_color: Option<String>,
    pub icon: Option<String>,
    pub icon_color: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<String>,
    pub font_family: Option<String>,
    pub on_click: Option<String>,
    pub image: Option<String>,
    pub image_scale: Option<f32>,
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
                    // Recursively collect all descendant names (transitive closure)
                    let mut removed_names: std::collections::HashSet<String> =
                        std::collections::HashSet::new();
                    removed_names.insert(name.to_string());
                    loop {
                        let mut changed = false;
                        for n in nodes.iter() {
                            if let Some(ref parent) = n.parent
                                && removed_names.contains(parent.as_str())
                                && !removed_names.contains(&n.name)
                            {
                                removed_names.insert(n.name.clone());
                                changed = true;
                            }
                        }
                        if !changed {
                            break;
                        }
                    }
                    nodes.retain(|n| !removed_names.contains(&n.name));
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
        // None = not specified, Some(None) = reset (empty string), Some(Some(id)) = explicit
        let display_change: Option<Option<u32>> = properties
            .get("display")
            .map(|v| {
                if v.is_empty() {
                    Ok(None)
                } else {
                    v.parse::<u32>()
                        .map(Some)
                        .map_err(|_| format!("invalid display: {}", v))
                }
            })
            .transpose()?;

        match display_change {
            Some(target_opt) => {
                let (explicit, target_display) = match target_opt {
                    Some(id) => (true, id),
                    None => (false, crate::main_display_id()),
                };
                let (current_display, idx) = self.find_node(name)?;
                if target_display != current_display {
                    let mut node = self.nodes.get_mut(&current_display).unwrap().remove(idx);
                    node.display = target_display;
                    node.display_explicit = explicit;
                    Self::apply_properties(&mut node, properties)?;
                    let display_nodes = self.nodes.entry(target_display).or_default();
                    display_nodes.push(node.clone());
                    display_nodes.sort_by_key(|n| n.position);
                    return Ok(node);
                }
                let nodes = self.nodes.get_mut(&current_display).unwrap();
                let node = &mut nodes[idx];
                node.display_explicit = explicit;
                Self::apply_properties(node, properties)?;
                let updated = node.clone();
                nodes.sort_by_key(|n| n.position);
                Ok(updated)
            }
            None => {
                let (current_display, _idx) = self.find_node(name)?;
                let nodes = self.nodes.get_mut(&current_display).unwrap();
                let node = nodes.iter_mut().find(|n| n.name == name).unwrap();
                Self::apply_properties(node, properties)?;
                let updated = node.clone();
                nodes.sort_by_key(|n| n.position);
                Ok(updated)
            }
        }
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
                "image" => node.image = optional_str(value),
                "image_scale" => {
                    node.image_scale = if value.is_empty() {
                        None
                    } else {
                        Some(parse_f32("image_scale", value)?)
                    };
                }
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

    pub fn migrate_nodes(&mut self, from_display: u32, to_display: u32) -> Vec<BarNode> {
        let Some(nodes) = self.nodes.get_mut(&from_display) else {
            return vec![];
        };

        let mut staying_names: std::collections::HashSet<String> = nodes
            .iter()
            .filter(|n| n.display_explicit)
            .map(|n| n.name.clone())
            .collect();

        loop {
            let mut changed = false;
            for node in nodes.iter() {
                if staying_names.contains(&node.name) {
                    continue;
                }
                if let Some(ref parent) = node.parent
                    && staying_names.contains(parent)
                {
                    staying_names.insert(node.name.clone());
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let mut migrate = vec![];
        nodes.retain(|node| {
            if staying_names.contains(&node.name) {
                true
            } else {
                migrate.push(node.clone());
                false
            }
        });

        for node in &mut migrate {
            node.display = to_display;
        }

        if nodes.is_empty() {
            self.nodes.remove(&from_display);
        }

        let target = self.nodes.entry(to_display).or_default();
        target.extend(migrate.iter().cloned());
        target.sort_by_key(|n| n.position);

        migrate
    }

    pub fn get_nodes(&self) -> Vec<BarNode> {
        self.nodes.values().flatten().cloned().collect()
    }

    pub fn get_nodes_for_display(&self, display: u32) -> Vec<BarNode> {
        self.nodes.get(&display).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(name: &str, node_type: NodeType, parent: Option<&str>, display: u32) -> BarNode {
        BarNode {
            name: name.to_string(),
            node_type,
            parent: parent.map(|s| s.to_string()),
            position: 0,
            display,
            display_explicit: false,
            style: NodeStyle::default(),
            label: None,
            label_color: None,
            icon: None,
            icon_color: None,
            font_size: None,
            font_weight: None,
            font_family: None,
            on_click: None,
            image: None,
            image_scale: None,
        }
    }

    #[test]
    fn remove_item_node() {
        let mut state = BarState::default();
        state
            .add_node(make_node("item1", NodeType::Item, None, 1))
            .unwrap();

        let removed = state.remove_node("item1").unwrap();
        assert_eq!(removed.name, "item1");
        assert!(state.get_nodes().is_empty());
    }

    #[test]
    fn remove_nonexistent_node() {
        let mut state = BarState::default();
        let result = state.remove_node("ghost");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn remove_container_without_children() {
        let mut state = BarState::default();
        state
            .add_node(make_node("row1", NodeType::Row, None, 1))
            .unwrap();

        let removed = state.remove_node("row1").unwrap();
        assert_eq!(removed.name, "row1");
        assert!(state.get_nodes().is_empty());
    }

    #[test]
    fn remove_container_with_direct_children() {
        let mut state = BarState::default();
        state
            .add_node(make_node("row1", NodeType::Row, None, 1))
            .unwrap();
        state
            .add_node(make_node("child1", NodeType::Item, Some("row1"), 1))
            .unwrap();
        state
            .add_node(make_node("child2", NodeType::Item, Some("row1"), 1))
            .unwrap();

        let removed = state.remove_node("row1").unwrap();
        assert_eq!(removed.name, "row1");
        assert!(state.get_nodes().is_empty());
    }

    #[test]
    fn remove_container_with_deep_nesting() {
        let mut state = BarState::default();
        // root -> col1 -> row1 -> item1
        state
            .add_node(make_node("root", NodeType::Column, None, 1))
            .unwrap();
        state
            .add_node(make_node("col1", NodeType::Column, Some("root"), 1))
            .unwrap();
        state
            .add_node(make_node("row1", NodeType::Row, Some("col1"), 1))
            .unwrap();
        state
            .add_node(make_node("item1", NodeType::Item, Some("row1"), 1))
            .unwrap();

        let removed = state.remove_node("root").unwrap();
        assert_eq!(removed.name, "root");
        assert!(state.get_nodes().is_empty());
    }

    #[test]
    fn remove_container_leaves_unrelated_nodes() {
        let mut state = BarState::default();
        state
            .add_node(make_node("row1", NodeType::Row, None, 1))
            .unwrap();
        state
            .add_node(make_node("child1", NodeType::Item, Some("row1"), 1))
            .unwrap();
        state
            .add_node(make_node("unrelated", NodeType::Item, None, 1))
            .unwrap();

        state.remove_node("row1").unwrap();

        let remaining: Vec<String> = state.get_nodes().into_iter().map(|n| n.name).collect();
        assert_eq!(remaining, vec!["unrelated"]);
    }

    #[test]
    fn remove_item_does_not_cascade() {
        let mut state = BarState::default();
        state
            .add_node(make_node("row1", NodeType::Row, None, 1))
            .unwrap();
        state
            .add_node(make_node("sibling1", NodeType::Item, Some("row1"), 1))
            .unwrap();
        state
            .add_node(make_node("sibling2", NodeType::Item, Some("row1"), 1))
            .unwrap();

        state.remove_node("sibling1").unwrap();

        let remaining: Vec<String> = state.get_nodes().into_iter().map(|n| n.name).collect();
        assert!(remaining.contains(&"row1".to_string()));
        assert!(remaining.contains(&"sibling2".to_string()));
        assert_eq!(remaining.len(), 2);
    }
}
