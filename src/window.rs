// src/window.rs
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
        ).ok();

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
