// src/config.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ClutchConfig {
    pub packages: Vec<Package>,
    pub pill_position: PillPosition,
    pub theme: Theme,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Package {
    pub id: String,           // UUID
    pub name: String,         // "Work Projects"
    pub color: String,        // "#A8E6A3" hex color
    pub icon: String,         // emoji or icon name
    pub items: Vec<Item>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
    pub path: String,         // full path: "C:\Users\Harshal\Desktop\Velocity"
    pub name: String,         // display name: "Velocity"
    pub item_type: ItemType,  // Folder or File
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")] // Trap 5: tag enums for correct deserialization
pub enum ItemType {
    Folder,
    File,
    Shortcut,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PillPosition {
    pub x: i32,
    pub y: i32,
    pub edge: String,  // "right", "left", "top", "bottom"
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Theme {
    pub accent_color: String,
    pub pill_opacity: f32,
    pub dark_mode: bool,
}

impl ClutchConfig {
    pub fn load() -> Self {
        let config_path = get_config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let config_path = get_config_path();
        let content = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(config_path, content).ok();
    }
}

impl Default for ClutchConfig {
    fn default() -> Self {
        Self {
            packages: vec![],
            pill_position: PillPosition {
                x: -1,   // -1 = auto position on right edge
                y: -1,
                edge: "right".to_string(),
            },
            theme: Theme {
                accent_color: "#C15F3C".to_string(),
                pill_opacity: 0.92,
                dark_mode: true,
            },
        }
    }
}

fn get_config_path() -> std::path::PathBuf {
    let dirs = directories::ProjectDirs::from("com", "harshal", "clutch").unwrap();
    let config_dir = dirs.config_dir();
    std::fs::create_dir_all(config_dir).ok();
    config_dir.join("config.json")
}
