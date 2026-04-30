// src/main.rs
#![windows_subsystem = "windows"] // hide console on Windows

mod config;
mod state;
mod filesystem;
mod window;

use slint::{ModelRc, VecModel, Color}; // Added Color
use std::sync::{Arc, Mutex};
use std::str::FromStr; // Added for color parsing

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
            // TODO Phase 5: show add package dialog
            return;
        }

        let mut state = state_clone.lock().unwrap();
        state.toggle_expand(&id);

        if let Some(window) = window_weak.upgrade() {
            let expanded_id = state.expanded_package.clone()
                .unwrap_or_default();

            window.set_expanded_id(expanded_id.clone().into());

            // Set expanded panel data
            if let Some(pkg) = state.config.packages.iter()
                .find(|p| p.id == expanded_id) {

                window.set_expanded_name(pkg.name.clone().into());
                
                // Parse color from hex string
                let color = Color::from_str(&pkg.color).unwrap_or(Color::from_rgb_u8(168, 230, 163));
                window.set_expanded_color(color);
                
                // Convert items to Slint model
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

    // --- Handle item opened ---
    window.on_item_opened(move |path| {
        filesystem::open_file(&path);
    });

    // Show window and setup native bits
    window.show().unwrap();
    
    // Trap 3: Setup window AFTER show
    if let Some(hwnd) = get_hwnd(&window) {
        window::setup_window(hwnd);
    }

    window.run().unwrap();
}

fn refresh_packages(window: &MainWindow, state: &Arc<Mutex<state::AppState>>) {
    let state = state.lock().unwrap();
    let pkgs: Vec<PackageData> = state.config.packages.iter().map(|p| {
        PackageData {
            id: p.id.clone().into(),
            name: p.name.clone().into(),
            color: Color::from_str(&p.color).unwrap_or(Color::from_rgb_u8(168, 230, 163)),
            item_count: p.items.len() as i32,
            items: ModelRc::new(VecModel::from(vec![])), // simplified for pill view
        }
    }).collect();
    
    window.set_packages(ModelRc::new(VecModel::from(pkgs)));
}

// Helper to get HWND from Slint window
fn get_hwnd(window: &MainWindow) -> Option<windows::Win32::Foundation::HWND> {
    use i_slint_backend_winit::WinitWindowAccessor;
    window.window().as_ref()
        .and_then(|adapter| adapter.get_winit_window())
        .map(|w| windows::Win32::Foundation::HWND(w.hwnd() as isize))
}
