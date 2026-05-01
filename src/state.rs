// src/state.rs
use crate::config::{ClutchConfig, Package, Item};

pub struct AppState {
    pub config: ClutchConfig,
    pub expanded_package: Option<String>, // ID of currently expanded package
    pub hovered_package: Option<String>,  // ID of hovered package
}

impl AppState {
    pub fn new() -> Self {
        let config = ClutchConfig::load();
        let mut state = Self {
            config,
            expanded_package: None,
            hovered_package: None,
        };
        
        // Add a sample package if empty so the user sees something
        if state.config.packages.is_empty() {
            state.add_package("General".to_string(), "#A8E6A3".to_string());
        }
        
        state
    }

    pub fn add_package(&mut self, name: String, color: String) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.config.packages.push(Package {
            id: id.clone(),
            name,
            color,
            icon: "📁".to_string(),
            items: vec![],
        });
        self.config.save();
        id
    }

    pub fn add_item_to_package(&mut self, package_id: &str, path: String) {
        if let Some(pkg) = self.config.packages.iter_mut()
            .find(|p| p.id == package_id) {

            let name = std::path::Path::new(&path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let is_folder = std::path::Path::new(&path).is_dir();

            pkg.items.push(Item {
                path: path.clone(),
                name,
                item_type: if is_folder {
                    crate::config::ItemType::Folder
                } else {
                    crate::config::ItemType::File
                },
            });

            self.config.save();
        }
    }

    pub fn remove_package(&mut self, package_id: &str) {
        self.config.packages.retain(|p| p.id != package_id);
        self.config.save();
    }

    pub fn remove_item(&mut self, package_id: &str, item_path: &str) {
        if let Some(pkg) = self.config.packages.iter_mut()
            .find(|p| p.id == package_id) {
            pkg.items.retain(|i| i.path != item_path);
            self.config.save();
        }
    }

    pub fn toggle_expand(&mut self, package_id: &str) {
        if self.expanded_package.as_deref() == Some(package_id) {
            self.expanded_package = None;
        } else {
            self.expanded_package = Some(package_id.to_string());
        }
    }
}
