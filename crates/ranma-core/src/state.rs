use std::collections::HashMap;

#[derive(Debug, Clone, uniffi::Record)]
pub struct BarItem {
    pub name: String,
    pub label: Option<String>,
    pub icon: Option<String>,
    pub icon_color: Option<String>,
    pub background_color: Option<String>,
    pub position: i32,
    pub display: u32,
}

#[derive(Debug, Default)]
pub struct BarState {
    items: HashMap<u32, Vec<BarItem>>,
}

impl BarState {
    pub fn add_item(&mut self, item: BarItem) -> Result<(), String> {
        let display_items = self.items.entry(item.display).or_default();
        if display_items.iter().any(|i| i.name == item.name) {
            return Err(format!("item '{}' already exists on display {}", item.name, item.display));
        }
        display_items.push(item);
        display_items.sort_by_key(|i| i.position);
        Ok(())
    }

    pub fn remove_item(&mut self, name: &str) -> Result<BarItem, String> {
        for items in self.items.values_mut() {
            if let Some(pos) = items.iter().position(|i| i.name == name) {
                return Ok(items.remove(pos));
            }
        }
        Err(format!("item '{}' not found", name))
    }

    pub fn set_properties(
        &mut self,
        name: &str,
        properties: &HashMap<String, String>,
    ) -> Result<BarItem, String> {
        // Check if we need to move the item to a different display
        let new_display = properties.get("display").map(|v| {
            v.parse::<u32>()
                .map_err(|_| format!("invalid display: {}", v))
        }).transpose()?;

        let (current_display, idx) = self.find_item(name)?;

        if let Some(target_display) = new_display {
            if target_display != current_display {
                let mut item = self.items.get_mut(&current_display).unwrap().remove(idx);
                item.display = target_display;
                Self::apply_properties(&mut item, properties)?;
                let display_items = self.items.entry(target_display).or_default();
                display_items.push(item.clone());
                display_items.sort_by_key(|i| i.position);
                return Ok(item);
            }
        }

        let items = self.items.get_mut(&current_display).unwrap();
        let item = &mut items[idx];
        Self::apply_properties(item, properties)?;
        let updated = item.clone();
        items.sort_by_key(|i| i.position);
        Ok(updated)
    }

    fn apply_properties(item: &mut BarItem, properties: &HashMap<String, String>) -> Result<(), String> {
        for (key, value) in properties {
            match key.as_str() {
                "label" => item.label = Some(value.clone()),
                "icon" => item.icon = Some(value.clone()),
                "icon_color" => item.icon_color = Some(value.clone()),
                "background_color" => item.background_color = Some(value.clone()),
                "position" => {
                    item.position = value
                        .parse()
                        .map_err(|_| format!("invalid position: {}", value))?;
                }
                "display" => {} // handled separately
                _ => return Err(format!("unknown property: {}", key)),
            }
        }
        Ok(())
    }

    fn find_item(&self, name: &str) -> Result<(u32, usize), String> {
        for (&display, items) in &self.items {
            if let Some(pos) = items.iter().position(|i| i.name == name) {
                return Ok((display, pos));
            }
        }
        Err(format!("item '{}' not found", name))
    }

    pub fn get_items(&self) -> Vec<BarItem> {
        self.items.values().flatten().cloned().collect()
    }

    pub fn get_items_for_display(&self, display: u32) -> Vec<BarItem> {
        self.items.get(&display).cloned().unwrap_or_default()
    }
}
