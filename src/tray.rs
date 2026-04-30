// src/tray.rs
use tray_icon::{TrayIcon, TrayIconBuilder, menu::{Menu, MenuItem}};
use std::sync::Arc;

pub fn create_tray() -> TrayIcon {
    let menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    let show_item = MenuItem::new("Show Clutch", true, None);
    
    menu.append(&show_item).ok();
    menu.append(&quit_item).ok();

    TrayIconBuilder::new()
        .with_tooltip("Clutch")
        // .with_icon(icon) // TODO: load an icon
        .with_menu(Box::new(menu))
        .build()
        .unwrap()
}
