// src/filesystem.rs
use std::path::Path;

// Open a folder in Windows Explorer
pub fn open_folder(path: &str) {
    open::that(path).ok();
}

// Open a file with its default application
pub fn open_file(path: &str) {
    open::that(path).ok();
}

// Check if path still exists (desktop items can be deleted)
pub fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

// Get all items on the Desktop
pub fn get_desktop_items() -> Vec<(String, String, bool)> {
    // Returns Vec of (name, full_path, is_folder)
    if let Some(dirs) = directories::UserDirs::new() {
        if let Some(desktop) = dirs.desktop_dir() {
            return std::fs::read_dir(desktop)
                .unwrap()
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_folder = path.is_dir();
                    Some((name, path.to_string_lossy().to_string(), is_folder))
                })
                .collect();
        }
    }
    vec![]
}
