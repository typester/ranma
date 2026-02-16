use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::state::BarItem;

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    Add {
        name: String,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        icon: Option<String>,
        #[serde(default)]
        icon_color: Option<String>,
        #[serde(default)]
        background_color: Option<String>,
        #[serde(default)]
        position: Option<i32>,
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
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Response {
    Ok,
    Error { message: String },
    QueryResult { items: Vec<ItemDto> },
}

#[derive(Debug, Serialize)]
pub struct ItemDto {
    pub name: String,
    pub label: Option<String>,
    pub icon: Option<String>,
    pub icon_color: Option<String>,
    pub background_color: Option<String>,
    pub position: i32,
}

impl From<BarItem> for ItemDto {
    fn from(item: BarItem) -> Self {
        ItemDto {
            name: item.name,
            label: item.label,
            icon: item.icon,
            icon_color: item.icon_color,
            background_color: item.background_color,
            position: item.position,
        }
    }
}
