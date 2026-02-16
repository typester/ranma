use std::collections::HashMap;

#[derive(Debug, Clone, uniffi::Record)]
pub struct BarItem {
    pub name: String,
    pub label: Option<String>,
    pub icon: Option<String>,
    pub icon_color: Option<String>,
    pub background_color: Option<String>,
    pub position: i32,
}

#[derive(Debug, Default)]
pub struct BarState {
    items: Vec<BarItem>,
}

impl BarState {
    pub fn add_item(&mut self, item: BarItem) -> Result<(), String> {
        if self.items.iter().any(|i| i.name == item.name) {
            return Err(format!("item '{}' already exists", item.name));
        }
        self.items.push(item);
        self.items.sort_by_key(|i| i.position);
        Ok(())
    }

    pub fn remove_item(&mut self, name: &str) -> Result<BarItem, String> {
        let pos = self
            .items
            .iter()
            .position(|i| i.name == name)
            .ok_or_else(|| format!("item '{}' not found", name))?;
        Ok(self.items.remove(pos))
    }

    pub fn set_properties(
        &mut self,
        name: &str,
        properties: &HashMap<String, String>,
    ) -> Result<BarItem, String> {
        let item = self
            .items
            .iter_mut()
            .find(|i| i.name == name)
            .ok_or_else(|| format!("item '{}' not found", name))?;

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
                _ => return Err(format!("unknown property: {}", key)),
            }
        }

        let updated = item.clone();
        self.items.sort_by_key(|i| i.position);
        Ok(updated)
    }

    pub fn get_items(&self) -> Vec<BarItem> {
        self.items.clone()
    }
}
