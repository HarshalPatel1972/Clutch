# 🗂️ Clutch — Complete Build Prompt
# Your desktop. Finally organized.

---

## Before You Write A Single Line — Read This Fully

You have already read the SKILL.md. That is your technical ground truth.
This is your mission.

Clutch was born from a real frustration — a desktop with 50+ scattered
folders, files, and shortcuts with no structure. Clutch does NOT move
or delete anything. It creates a beautiful floating overlay that groups
desktop items visually. The actual files stay exactly where they are.

One floating pill on the right edge of the screen.
Colored cards inside it — each one a package.
Hover a card — it expands showing the items inside.
Click an item — it opens directly.
Clean desktop. Zero mess. Everything one hover away.

---

## 📌 Project Identity

**Name:** Clutch
**Tagline:** Your desktop. Finally organized.
**Stack:** Rust + Slint 1.9
**Repo:** https://github.com/HarshalPatel1972/Clutch.git
**RAM target:** Under 5MB idle — runs 24/7 in background
**Platform:** Windows 11 first, Windows 10 compatible

---

## 🎨 Design System

### Colors
```
Pill background:    rgba(18, 18, 24, 0.92)   dark glass
Pill border:        rgba(255, 255, 255, 0.10) subtle rim
Card default:       see package colors below
Card hover:         brighter, slightly scaled up
Panel background:   rgba(22, 22, 30, 0.96)   darker glass
Panel border:       rgba(255, 255, 255, 0.08)
Text primary:       #F0EDE8                   warm white
Text secondary:     #8E8B82                   muted
Text dim:           #5A5750                   very muted
Shadow:             rgba(0, 0, 0, 0.6) 24px blur
```

### Package Card Colors (presets)
```
Mint green:   #A8E6A3  ← default for new packages
Coral:        #F4A896
Sky blue:     #96C8F4
Lavender:     #C8A8F4
Amber:        #F4D896
Rose:         #F4A8C8
```

### Animation Values — These Give The Apple Feel
```
Card hover scale:     1.08 with spring overshoot
Spring easing:        cubic-bezier(0.34, 1.56, 0.64, 1)
Smooth expand:        cubic-bezier(0.16, 1, 0.3, 1)
Duration hover:       200ms
Duration expand:      280ms
Duration fade:        180ms
```

---

## 🏗️ Phase 1 — Project Setup

### Step 1A — Initialize

```bash
cargo new clutch
cd clutch
```

### Step 1B — Cargo.toml

Replace entire `Cargo.toml` with the exact version from SKILL.md Section 3.
Do not modify it.

### Step 1C — build.rs

Create `build.rs` in project root:
```rust
fn main() {
    slint::compile("ui/app.slint").unwrap();
}
```

### Step 1D — Create folder structure

```bash
mkdir ui assets src
```

Create empty placeholder files:
- `ui/app.slint`
- `ui/pill.slint`
- `ui/card.slint`
- `ui/panel.slint`
- `assets/icon.ico` (any valid ICO file for now)

### Step 1E — Minimal main.rs to verify setup

```rust
// src/main.rs
slint::slint! {
    export component MainWindow inherits Window {
        width: 200px;
        height: 400px;
        background: #1a1a2e;

        Text {
            text: "Clutch";
            color: white;
            font-size: 24px;
            horizontal-alignment: center;
            vertical-alignment: center;
        }
    }
}

fn main() {
    let window = MainWindow::new().unwrap();
    window.run().unwrap();
}
```

**Test:** `cargo run` — a dark window with "Clutch" text must appear.

**Commit:**
```
feat: clutch project setup — rust, slint, cargo configuration
```

---

## 🗃️ Phase 2 — Data Layer

### Step 2A — Config structs

Create `src/config.rs` with the exact content from SKILL.md Section 8.
This is the complete data model for Clutch.

Key structures:
- `ClutchConfig` — root config with packages + settings
- `Package` — a named group with color, icon, and list of items
- `Item` — a path to a file or folder with display name
- `PillPosition` — where the pill sits on screen
- `Theme` — colors and opacity

All structs derive `Serialize`, `Deserialize`, `Clone`.

Config saves to: `%APPDATA%\clutch\config.json`

### Step 2B — App state

Create `src/state.rs`:

```rust
// src/state.rs
use crate::config::{ClutchConfig, Package, Item};

pub struct AppState {
    pub config: ClutchConfig,
    pub expanded_package: Option<String>, // ID of currently expanded package
    pub hovered_package: Option<String>,  // ID of hovered package
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: ClutchConfig::load(),
            expanded_package: None,
            hovered_package: None,
        }
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
```

**Commit:**
```
feat: config structs, app state, package and item management
```

---

## 🖥️ Phase 3 — Slint UI Components

Build UI components in this exact order.
Each component builds on the previous one.

### Step 3A — card.slint

The individual colored card inside the pill.
This is the most important animation component.

```slint
// ui/card.slint
export component Card inherits Rectangle {
    // --- Properties ---
    in property <string> card-name: "Package";
    in property <string> card-color: "#A8E6A3";
    in property <int> item-count: 0;
    in property <bool> is-expanded: false;

    // --- Callbacks ---
    callback clicked();
    callback right-clicked();

    // --- Size with spring animation ---
    width: touch.has-hover ? 54px : 48px;
    height: touch.has-hover ? 54px : 48px;

    animate width {
        duration: 200ms;
        easing: cubic-bezier(0.34, 1.56, 0.64, 1);
    }
    animate height {
        duration: 200ms;
        easing: cubic-bezier(0.34, 1.56, 0.64, 1);
    }

    // --- Visual ---
    background: @linear-gradient(145deg,
        Colors.from-argb-encoded(card-color.to-argb()),
        Colors.from-argb-encoded(card-color.to-argb()).darker(0.15)
    );
    border-radius: 12px;
    border-width: is-expanded ? 2px : 1px;
    border-color: is-expanded
        ? Colors.white.with-alpha(0.6)
        : Colors.white.with-alpha(0.2);

    drop-shadow-blur: touch.has-hover ? 12px : 6px;
    drop-shadow-color: Colors.black.with-alpha(0.4);
    drop-shadow-offset-y: touch.has-hover ? 6px : 3px;

    animate drop-shadow-blur {
        duration: 200ms;
        easing: ease-out;
    }

    // --- Item count badge ---
    if item-count > 0 : Rectangle {
        x: parent.width - 14px;
        y: -4px;
        width: 18px;
        height: 18px;
        border-radius: 9px;
        background: Colors.white.with-alpha(0.9);

        Text {
            text: item-count > 9 ? "9+" : item-count;
            color: #333333;
            font-size: 9px;
            font-weight: 700;
            horizontal-alignment: center;
            vertical-alignment: center;
        }
    }

    // --- Active indicator dot ---
    if is-expanded : Rectangle {
        x: (parent.width - 6px) / 2;
        y: parent.height + 4px;
        width: 6px;
        height: 6px;
        border-radius: 3px;
        background: Colors.white.with-alpha(0.8);
    }

    // --- Touch area ---
    touch := TouchArea {
        clicked => { root.clicked(); }
        pointer-event(event) => {
            if event.button == PointerEventButton.right {
                root.right-clicked();
            }
        }
    }
}
```

### Step 3B — panel.slint

The expanded panel that appears when a card is clicked.
This is where the folder items are shown.

```slint
// ui/panel.slint
import { VerticalBox } from "std-widgets.slint";

export struct ItemData {
    name: string,
    path: string,
    is-folder: bool,
    exists: bool,
}

export component ExpandedPanel inherits Rectangle {
    // --- Properties ---
    in property <string> package-name: "Package";
    in property <string> package-color: "#A8E6A3";
    in property <[ItemData]> items: [];
    in property <bool> visible-state: false;

    // --- Callbacks ---
    callback item-clicked(string);        // path
    callback item-right-clicked(string);  // path
    callback close-panel();

    // --- Size ---
    width: 280px;
    min-height: 80px;

    // --- Animations ---
    opacity: visible-state ? 1.0 : 0.0;
    x: visible-state ? -290px : -260px;  // slides in from right

    animate opacity {
        duration: 200ms;
        easing: ease-out;
    }
    animate x {
        duration: 280ms;
        easing: cubic-bezier(0.16, 1, 0.3, 1);
    }

    // --- Background ---
    background: rgba(18, 18, 26, 0.96);
    border-radius: 16px;
    border-width: 1px;
    border-color: rgba(255, 255, 255, 0.10);
    drop-shadow-blur: 32px;
    drop-shadow-color: rgba(0, 0, 0, 0.6);
    drop-shadow-offset-x: -4px;

    VerticalLayout {
        padding: 14px;
        spacing: 8px;

        // --- Panel header ---
        HorizontalLayout {
            spacing: 8px;
            alignment: center;

            // Color dot
            Rectangle {
                width: 10px;
                height: 10px;
                border-radius: 5px;
                background: Colors.from-argb-encoded(package-color.to-argb());
            }

            // Package name
            Text {
                text: package-name;
                color: #F0EDE8;
                font-size: 13px;
                font-weight: 600;
                vertical-alignment: center;
                horizontal-stretch: 1;
            }

            // Close button
            close-btn := Rectangle {
                width: 22px;
                height: 22px;
                border-radius: 11px;
                background: close-touch.has-hover
                    ? rgba(255, 255, 255, 0.15)
                    : transparent;

                Text {
                    text: "×";
                    color: #8E8B82;
                    font-size: 16px;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }

                close-touch := TouchArea {
                    clicked => { root.close-panel(); }
                }

                animate background {
                    duration: 120ms;
                    easing: ease-out;
                }
            }
        }

        // --- Divider ---
        Rectangle {
            height: 1px;
            background: rgba(255, 255, 255, 0.08);
        }

        // --- Items list ---
        if items.length == 0 : Text {
            text: "Drop items here\nor right-click to add";
            color: #5A5750;
            font-size: 12px;
            horizontal-alignment: center;
            wrap: word-wrap;
        }

        for item in items : item-row := HorizontalLayout {
            spacing: 10px;
            padding-left: 4px;
            padding-right: 4px;

            item-bg := Rectangle {
                border-radius: 8px;
                background: item-touch.has-hover
                    ? rgba(255, 255, 255, 0.08)
                    : transparent;
                padding: 8px;

                animate background {
                    duration: 120ms;
                    easing: ease-out;
                }

                HorizontalLayout {
                    spacing: 10px;
                    alignment: center;

                    // Icon
                    Text {
                        text: item.is-folder ? "📁" : "📄";
                        font-size: 16px;
                        vertical-alignment: center;
                    }

                    // Name
                    Text {
                        text: item.name;
                        color: item.exists ? #F0EDE8 : #5A5750;
                        font-size: 13px;
                        vertical-alignment: center;
                        horizontal-stretch: 1;
                        overflow: elide;
                    }

                    // Missing indicator
                    if !item.exists : Text {
                        text: "✕";
                        color: #F4A896;
                        font-size: 11px;
                        vertical-alignment: center;
                    }
                }

                item-touch := TouchArea {
                    clicked => {
                        if item.exists {
                            root.item-clicked(item.path);
                        }
                    }
                    pointer-event(event) => {
                        if event.button == PointerEventButton.right {
                            root.item-right-clicked(item.path);
                        }
                    }
                }
            }
        }

        // --- Add item hint ---
        Rectangle {
            height: 32px;
            border-radius: 8px;
            border-width: 1px;
            border-color: rgba(255, 255, 255, 0.08);
            background: add-touch.has-hover
                ? rgba(255, 255, 255, 0.05)
                : transparent;

            Text {
                text: "+ Add from Desktop";
                color: #5A5750;
                font-size: 11px;
                horizontal-alignment: center;
                vertical-alignment: center;
            }

            add-touch := TouchArea {}

            animate background {
                duration: 120ms;
                easing: ease-out;
            }
        }
    }
}
```

### Step 3C — pill.slint

The main pill container with all cards.

```slint
// ui/pill.slint
import { Card } from "card.slint";
import { ExpandedPanel, ItemData } from "panel.slint";

export struct PackageData {
    id: string,
    name: string,
    color: string,
    item-count: int,
    items: [ItemData],
}

export component Pill inherits Rectangle {
    // --- Properties ---
    in property <[PackageData]> packages: [];
    in property <string> expanded-id: "";

    // --- Callbacks ---
    callback package-clicked(string);          // package id
    callback package-right-clicked(string);    // package id
    callback item-opened(string);              // file path
    callback item-right-clicked(string);       // file path

    // --- Size ---
    width: 68px;
    height: packages.length * 58px + 24px;

    // --- Visual ---
    background: rgba(18, 18, 24, 0.92);
    border-radius: 22px;
    border-width: 1px;
    border-color: rgba(255, 255, 255, 0.10);
    drop-shadow-blur: 28px;
    drop-shadow-color: rgba(0, 0, 0, 0.55);
    drop-shadow-offset-x: -4px;
    drop-shadow-offset-y: 8px;

    // --- Cards ---
    VerticalLayout {
        padding: 12px;
        spacing: 8px;
        alignment: center;

        for pkg in packages : Card {
            card-name: pkg.name;
            card-color: pkg.color;
            item-count: pkg.item-count;
            is-expanded: root.expanded-id == pkg.id;

            clicked => { root.package-clicked(pkg.id); }
            right-clicked => { root.package-right-clicked(pkg.id); }
        }

        // Add package button
        add-btn := Rectangle {
            width: 36px;
            height: 36px;
            border-radius: 10px;
            border-width: 1px;
            border-color: rgba(255, 255, 255, 0.12);
            background: add-touch.has-hover
                ? rgba(255, 255, 255, 0.08)
                : rgba(255, 255, 255, 0.04);

            Text {
                text: "+";
                color: #8E8B82;
                font-size: 20px;
                horizontal-alignment: center;
                vertical-alignment: center;
            }

            add-touch := TouchArea {
                clicked => {
                    // Signal Rust to show add package dialog
                    root.package-clicked("__add__");
                }
            }

            animate background {
                duration: 150ms;
                easing: ease-out;
            }
        }
    }
}
```

### Step 3D — app.slint

The root window that puts it all together.

```slint
// ui/app.slint
import { Pill, PackageData } from "pill.slint";
import { ExpandedPanel, ItemData } from "panel.slint";

export component MainWindow inherits Window {
    // No frame, transparent — the pill floats over desktop
    no-frame: true;
    background: transparent;

    // Window size — fits the pill + panel space
    width: 360px;
    height: 600px;

    // --- Properties exposed to Rust ---
    in property <[PackageData]> packages: [];
    in property <string> expanded-id: "";
    in property <string> expanded-name: "";
    in property <string> expanded-color: "#A8E6A3";
    in property <[ItemData]> expanded-items: [];

    // --- Callbacks exposed to Rust ---
    callback package-clicked(string);
    callback package-right-clicked(string);
    callback item-opened(string);
    callback item-right-clicked(string);
    callback panel-closed();

    // --- The pill dock (right side) ---
    Pill {
        x: parent.width - self.width - 4px;
        y: (parent.height - self.height) / 2;
        packages: root.packages;
        expanded-id: root.expanded-id;

        package-clicked(id) => { root.package-clicked(id); }
        package-right-clicked(id) => { root.package-right-clicked(id); }
        item-opened(path) => { root.item-opened(path); }
        item-right-clicked(path) => { root.item-right-clicked(path); }
    }

    // --- Expanded panel (appears to the left of pill) ---
    ExpandedPanel {
        x: 4px;
        y: (parent.height - self.height) / 2;
        package-name: root.expanded-name;
        package-color: root.expanded-color;
        items: root.expanded-items;
        visible-state: root.expanded-id != "";

        item-clicked(path) => { root.item-opened(path); }
        item-right-clicked(path) => { root.item-right-clicked(path); }
        close-panel => { root.panel-closed(); }
    }
}
```

**Commit:**
```
feat: slint UI components — card with spring animation, panel, pill dock, main window
```

---

## 🔧 Phase 4 — Rust Backend

### Step 4A — filesystem.rs

Create `src/filesystem.rs` with exact content from SKILL.md Section 9.

Functions needed:
- `open_folder(path)` — opens in Windows Explorer
- `open_file(path)` — opens with default app
- `path_exists(path)` — checks if still exists
- `get_desktop_items()` — lists all desktop items

### Step 4B — window.rs

Create `src/window.rs` with exact content from SKILL.md Section 7.

This handles:
- Making the window always-on-top
- Removing window decorations
- Setting transparent click-through on empty areas

### Step 4C — main.rs — Wire Everything Together

```rust
// src/main.rs
#![windows_subsystem = "windows"] // hide console on Windows

mod config;
mod state;
mod filesystem;
mod window;

use slint::{ModelRc, VecModel, SharedString};
use std::sync::{Arc, Mutex};
use std::rc::Rc;

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
                window.set_expanded_color(pkg.color.clone().into());
                
                // Convert items to Slint model
                let items_vec: Vec<ItemData> = pkg.items.iter().map(|i| {
                    ItemData {
                        name: i.name.clone().into(),
                        path: i.path.clone().into(),
                        is_folder: matches!(i.item_type, config::ItemType::Folder),
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
            color: p.color.clone().into(),
            item_count: p.items.length() as i32,
            items: ModelRc::new(VecModel::from(vec![])), // simplified for pill view
        }
    }).collect();
    
    window.set_packages(ModelRc::new(VecModel::from(pkgs)));
}

// Helper to get HWND from Slint window
fn get_hwnd(window: &MainWindow) -> Option<windows::Win32::Foundation::HWND> {
    use slint::platform::WindowAdapter;
    window.window().as_ref()
        .get_winit_window()
        .map(|w| windows::Win32::Foundation::HWND(w.hwnd() as isize))
}
```

---

## 🛠️ Phase 5 — Polish & Persistence

### Step 5A — File watcher
### Step 5B — System Tray
### Step 5C — Final Polish

*(Details to be provided in Phase 5 prompt)*
