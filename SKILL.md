---
name: clutch-skills
description: Ground truth technical reference for building Clutch — a desktop organizer overlay app built with Rust + Slint. Features a floating pill dock with package cards that expand on hover to show grouped desktop items. GPU-accelerated, <5MB RAM idle. Read this completely before writing any code.
---

# Clutch — Agent SKILL.md
# Read every word. Then build. Never guess.

---

## What Clutch Is

A floating overlay app that lives permanently on top of the Windows desktop.
It does NOT move files. It does NOT create folders.
It is a VISUAL GROUPING LAYER over the existing desktop.

```
Your messy desktop (unchanged)
         +
Clutch pill (floating overlay, always on top)
         =
Clean looking desktop with organized access
```

---

## 1. Tech Stack — Exact Choices

```
Language:     Rust (latest stable)
UI Framework: Slint 1.9.x
Windows API:  windows-rs 0.58.x
Config:       serde + serde_json
Build:        Cargo
```

### Why Slint

- GPU-accelerated rendering — same pipeline as Apple's Metal/Core Animation
- 300KB runtime RAM — total app idles under 5MB
- Declarative animation syntax with spring physics built in
- Transparent window support
- Compiles to native binary — no runtime, no VM, no browser

### Why NOT Tauri/Electron

Both use Chromium WebView — minimum 80-150MB RAM idle.
Clutch runs 24/7. 150MB is unacceptable for a background dock.

---

## 2. Project Structure

```
clutch/
├── Cargo.toml
├── build.rs                    ← Slint build script
├── src/
│   ├── main.rs                 ← entry point
│   ├── window.rs               ← transparent always-on-top window
│   ├── state.rs                ← app state (packages, items)
│   ├── config.rs               ← load/save JSON config
│   ├── filesystem.rs           ← open folders/files via Windows API
│   └── tray.rs                 ← system tray icon
├── ui/
│   ├── app.slint               ← main UI definition
│   ├── pill.slint              ← the dock pill component
│   ├── card.slint              ← individual package card
│   └── panel.slint             ← expanded package panel
└── assets/
    ├── icon.ico
    └── folder.svg
```

---

## 3. Cargo.toml — Complete and Correct

```toml
[package]
name = "clutch"
version = "1.0.0"
edition = "2021"

[dependencies]
# Slint — GPU-accelerated UI framework
slint = "1.9"

# Windows API — for file operations, window management
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_UI_Shell",           # SHOpenFolderAndSelectItems
    "Win32_UI_WindowsAndMessaging", # SetWindowPos, always on top
    "Win32_Foundation",         # HWND, BOOL, RECT
    "Win32_Graphics_Gdi",       # screen dimensions
    "Win32_System_Threading",   # background threads
    "Win32_UI_Shell_Common",    # shell item
] }

# Config — JSON serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Tray icon
tray-icon = "0.16"

# File watching — detect desktop changes
notify = "6"

# Open files/folders cross-platform
open = "5"

# UUID for package IDs
uuid = { version = "1", features = ["v4"] }

# Directories — find Desktop, AppData paths
directories = "5"

[build-dependencies]
slint-build = "1.9"
```

---

## 4. build.rs — Required For Slint

```rust
// build.rs — MUST exist or Slint UI files won't compile
fn main() {
    slint::compile("ui/app.slint").unwrap();
}
```

---

## 5. Slint Animation System — How To Get Apple Feel

Slint has built-in animation with easing curves.
The Apple dock feel comes from these specific values:

### Spring-like bounce (Apple dock magnification)
```slint
animate width {
    duration: 250ms;
    easing: cubic-bezier(0.34, 1.56, 0.64, 1); // overshoots then settles
}
```

### Smooth expand (panel opening)
```slint
animate height {
    duration: 300ms;
    easing: cubic-bezier(0.16, 1, 0.3, 1); // ease-out-expo
}
```

### Fade in
```slint
animate opacity {
    duration: 200ms;
    easing: ease-in-out;
}
```

### Apple dock hover magnification — exact values
```slint
// Card hover state
card := Rectangle {
    width: self.has-hover ? 72px : 60px;
    height: self.has-hover ? 72px : 60px;

    animate width {
        duration: 200ms;
        easing: cubic-bezier(0.34, 1.56, 0.64, 1);
    }
    animate height {
        duration: 200ms;
        easing: cubic-bezier(0.34, 1.56, 0.64, 1);
    }
}
```

---

## 6. Slint Syntax — Key Concepts

### Basic component structure
```slint
component MyComponent {
    // Properties
    in property <string> label: "default";
    in property <bool> expanded: false;
    out property <bool> clicked;

    // Callbacks
    callback on-click();

    // Visual tree
    Rectangle {
        background: #1a1a2e;
        border-radius: 12px;

        Text {
            text: label;
            color: white;
        }

        TouchArea {
            clicked => {
                root.on-click();
            }
        }
    }
}
```

### Conditional visibility
```slint
Rectangle {
    visible: expanded; // show/hide based on state
    opacity: expanded ? 1.0 : 0.0;

    animate opacity {
        duration: 250ms;
        easing: ease-in-out;
    }
}
```

### Repeater — render list of items
```slint
for item in items : Rectangle {
    Text { text: item.name; }
}
```

### Touch and hover
```slint
TouchArea {
    has-hover: false; // built-in hover detection

    pointer-event(event) => {
        if event.kind == PointerEventKind.up {
            // handle click
        }
    }
}
```

---

## 7. Transparent Always-On-Top Window — Windows API

This is the most critical technical piece.
Clutch must be a frameless, transparent, always-on-top window.

```rust
// window.rs
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn setup_window(hwnd: HWND) {
    unsafe {
        // Make window always on top
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,  // always on top
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE,
        );

        // Remove window decorations (no title bar, no border)
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        SetWindowLongW(
            hwnd,
            GWL_STYLE,
            style & !(WS_OVERLAPPEDWINDOW.0 as i32),
        );

        // Enable click-through on transparent areas
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            ex_style | WS_EX_LAYERED.0 as i32 | WS_EX_TRANSPARENT.0 as i32,
        );
    }
}
```

### Slint transparent window setup
```rust
// main.rs — when creating Slint window
let window = MainWindow::new().unwrap();

// Get the raw HWND from Slint
use slint::platform::WindowAdapter;
let hwnd = window.window().as_ref()
    .get_winit_window()  // get underlying winit window
    .map(|w| w.hwnd());  // get Win32 HWND

if let Some(hwnd) = hwnd {
    setup_window(HWND(hwnd as isize));
}
```

---

## 8. Config Structure — JSON

Clutch saves packages to a JSON config file in AppData:

```rust
// config.rs
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
```

---

## 9. Opening Files and Folders

```rust
// filesystem.rs

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
    std::path::Path::new(path).exists()
}

// Get all items on the Desktop
pub fn get_desktop_items() -> Vec<(String, String, bool)> {
    // Returns Vec of (name, full_path, is_folder)
    let dirs = directories::UserDirs::new().unwrap();
    let desktop = dirs.desktop_dir().unwrap();

    std::fs::read_dir(desktop)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let is_folder = path.is_dir();
            Some((name, path.to_string_lossy().to_string(), is_folder))
        })
        .collect()
}
```

---

## 10. The Slint UI Files

### app.slint — Main entry
```slint
import { Pill } from "pill.slint";
import { ExpandedPanel } from "panel.slint";

export component MainWindow inherits Window {
    // Transparent background — critical for overlay
    background: transparent;

    // No window decorations
    no-frame: true;

    // Always fits content
    width: 80px;
    height: 400px;

    // The pill dock
    Pill {
        x: 0;
        y: 0;
        width: parent.width;
        height: parent.height;
    }
}
```

### pill.slint — The dock
```slint
import { Card } from "card.slint";

export component Pill inherits Rectangle {
    // Semi-transparent dark pill
    background: rgba(20, 20, 30, 0.88);
    border-radius: 20px;

    // Frosted glass effect via border
    border-width: 1px;
    border-color: rgba(255, 255, 255, 0.12);

    // Drop shadow
    drop-shadow-blur: 24px;
    drop-shadow-color: rgba(0, 0, 0, 0.5);
    drop-shadow-offset-x: 0px;
    drop-shadow-offset-y: 8px;

    // Stack cards vertically
    VerticalLayout {
        padding: 10px;
        spacing: 8px;
        alignment: center;

        // Cards are rendered here by Rust via model
        for package in packages : Card {
            name: package.name;
            color: package.color;
            item-count: package.item-count;
        }
    }
}
```

---

## 11. Known Traps — Read Every One

### Trap 1: Slint build.rs is mandatory
Without `build.rs`, Slint `.slint` files won't compile.
Always have:
```rust
// build.rs
fn main() {
    slint::compile("ui/app.slint").unwrap();
}
```

### Trap 2: Transparent window requires specific Slint backend
```rust
// In main.rs — enable software renderer for transparency
// OR use the default renderer with proper window flags
slint::platform::set_platform(
    Box::new(slint::platform::software_renderer::MinimalSoftwareWindow::new(
        slint::platform::software_renderer::RepaintBufferType::NewBuffer,
    ))
).ok();
```

### Trap 3: Always-on-top must be set AFTER window is shown
Setting HWND_TOPMOST before the window is visible has no effect.
Always call `setup_window()` after `window.show()`.

### Trap 4: Slint animations require property changes
Animations only trigger when a property VALUE changes.
If the value is already at the target — animation won't play.
Always set the initial state explicitly.

### Trap 5: serde_json with enums needs tags
```rust
// ❌ WRONG — fails to deserialize
#[derive(Serialize, Deserialize)]
enum ItemType { Folder, File }

// ✅ CORRECT
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ItemType { Folder, File }
```

### Trap 6: windows-rs unsafe blocks
All Windows API calls require `unsafe {}` blocks.
Always wrap in a function that handles the unsafe boundary:
```rust
pub fn set_topmost(hwnd: HWND) {
    unsafe {
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE);
    }
}
```

### Trap 7: Desktop path on Windows
Never hardcode `C:\Users\username\Desktop`.
Always use the `directories` crate:
```rust
let dirs = directories::UserDirs::new().unwrap();
let desktop = dirs.desktop_dir().unwrap();
```

### Trap 8: Drag and drop items into packages
To accept drag-and-drop in Slint — use the `DragDrop` area.
But for adding items to packages — use a right-click context menu
via Windows API instead. Much simpler to implement correctly.

---

## 12. Build and Run

```bash
# Install Rust if not installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and run
cargo run

# Build release (smaller, faster, no console)
cargo build --release
# Binary: target/release/clutch.exe

# Build without console window on Windows
# Add to Cargo.toml:
# [profile.release]
# windows_subsystem = "windows"  ← hides console
```

---

*Clutch SKILL.md — verified April 2026.*
*When in doubt — check here first. Search second. Guess never.*
