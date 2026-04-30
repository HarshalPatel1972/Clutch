// src/main.rs
#![windows_subsystem = "windows"] // hide console on Windows

mod config;
mod state;
mod filesystem;
mod window;

use slint::{ModelRc, VecModel, Color}; // Added Color
use std::sync::{Arc, Mutex};

// Include the compiled Slint UI
slint::include_modules!();

fn main() {
    // Load app state
    let state = Arc::new(Mutex::new(state::AppState::new()));

    // Create the Slint window
    let window = MainWindow::new().unwrap();

    // Initial data — populate packages from config
    refresh_packages(&window, &state);

    // --- Handle package card clicked ---
    let state_clone = state.clone();
    let window_weak = window.as_weak();
    window.on_package_clicked(move |id| {
        let id = id.to_string();

        if id == "__add__" {
            let mut state = state_clone.lock().unwrap();
            state.add_package("New Package".to_string(), "#A8E6A3".to_string());
            if let Some(window) = window_weak.upgrade() {
                refresh_packages(&window, &state_clone);
            }
            return;
        }

        let mut state = state_clone.lock().unwrap();
        state.toggle_expand(&id);
        
        if let Some(window) = window_weak.upgrade() {
            refresh_packages_and_panel(&window, &state);
        }
    });

    // --- Handle panel closed ---
    let state_clone = state.clone();
    let window_weak = window.as_weak();
    window.on_panel_closed(move || {
        let mut state = state_clone.lock().unwrap();
        state.expanded_package = None;
        if let Some(window) = window_weak.upgrade() {
            window.set_expanded_id("".into());
        }
    });

    // --- Handle item opened/added ---
    let state_clone = state.clone();
    let window_weak = window.as_weak();
    window.on_item_opened(move |path| {
        filesystem::open_file(&path);
    });

    window.on_item_right_clicked(move |path| {
        let path = path.to_string();
        if path == "__add_item__" {
            let state = state_clone.lock().unwrap();
            let expanded_id = match &state.expanded_package {
                Some(id) => id.clone(),
                None => return,
            };
            drop(state);

            let desktop_items = filesystem::get_desktop_items();
            let picker = DesktopPicker::new().unwrap();
            
            let picker_items: Vec<DesktopItem> = desktop_items.iter().map(|(name, path, is_folder)| {
                DesktopItem {
                    name: name.clone().into(),
                    path: path.clone().into(),
                    is_folder: *is_folder,
                }
            }).collect();
            
            picker.set_desktop_items(ModelRc::new(VecModel::from(picker_items)));
            
            let state_inner = state_clone.clone();
            let window_inner = window_weak.clone();
            let picker_weak = picker.as_weak();
            let target_pkg_id = expanded_id.clone();
            
            picker.on_item_selected(move |item_path| {
                let mut state = state_inner.lock().unwrap();
                state.add_item_to_package(&target_pkg_id, item_path.to_string());
                
                if let Some(window) = window_inner.upgrade() {
                    refresh_packages_and_panel(&window, &state);
                }
                if let Some(picker) = picker_weak.upgrade() {
                    picker.hide().unwrap();
                }
            });
            
            picker.show().unwrap();
        }
    });

    // Show window and setup native bits
    window.show().unwrap();
    
    // Trap 3: Setup window AFTER show
    if let Some(hwnd) = get_hwnd(&window) {
        window::setup_window(hwnd);
    }

    window.run().unwrap();
}

fn refresh_packages_and_panel(window: &MainWindow, state: &state::AppState) {
    let expanded_id = state.expanded_package.clone().unwrap_or_default();
    window.set_expanded_id(expanded_id.clone().into());

    // Set expanded panel data
    if let Some(pkg) = state.config.packages.iter().find(|p| p.id == expanded_id) {
        window.set_expanded_name(pkg.name.clone().into());
        window.set_expanded_color(parse_color(&pkg.color));

        let items_vec: Vec<ItemData> = pkg.items.iter().map(|i| {
            ItemData {
                name: i.name.clone().into(),
                path: i.path.clone().into(),
                is_folder: matches!(i.item_type, crate::config::ItemType::Folder),
                exists: filesystem::path_exists(&i.path),
            }
        }).collect();
        window.set_expanded_items(ModelRc::new(VecModel::from(items_vec)));
    }
    
    // Also refresh the pill list to update counts
    let pkgs: Vec<PackageData> = state.config.packages.iter().map(|p| {
        PackageData {
            id: p.id.clone().into(),
            name: p.name.clone().into(),
            color: parse_color(&p.color),
            item_count: p.items.len() as i32,
            items: ModelRc::new(VecModel::from(vec![])),
        }
    }).collect();
    window.set_packages(ModelRc::new(VecModel::from(pkgs)));
}

fn refresh_packages(window: &MainWindow, state_arc: &Arc<Mutex<state::AppState>>) {
    let state = state_arc.lock().unwrap();
    let pkgs: Vec<PackageData> = state.config.packages.iter().map(|p| {
        PackageData {
            id: p.id.clone().into(),
            name: p.name.clone().into(),
            color: parse_color(&p.color),
            item_count: p.items.len() as i32,
            items: ModelRc::new(VecModel::from(vec![])),
        }
    }).collect();
    
    window.set_packages(ModelRc::new(VecModel::from(pkgs)));
}

fn parse_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(168);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(230);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(163);
        Color::from_rgb_u8(r, g, b)
    } else {
        Color::from_rgb_u8(168, 230, 163)
    }
}

// Helper to get HWND from Slint window
fn get_hwnd(_window: &MainWindow) -> Option<windows::Win32::Foundation::HWND> {
    // Since we just called window.show(), the window should be active.
    // This is a reliable way to get the HWND for a single-window app.
    unsafe {
        let hwnd = windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow();
        if hwnd.0.is_null() {
            None
        } else {
            Some(hwnd)
        }
    }
}
