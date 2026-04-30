// src/main.rs
mod config;
mod state;

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
