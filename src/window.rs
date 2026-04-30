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

pub fn enable_autostart() {
    use windows::Win32::System::Registry::*;
    use windows::core::HSTRING;
    
    unsafe {
        let mut hkey = HKEY::default();
        let subkey = HSTRING::from("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            &subkey,
            0,
            KEY_WRITE,
            &mut hkey
        ).is_ok() {
            let exe_path = std::env::current_exe().unwrap();
            let path_str = HSTRING::from(exe_path.to_str().unwrap());
            let path_wide = path_str.as_wide();
            let data = unsafe { 
                std::slice::from_raw_parts(
                    path_wide.as_ptr() as *const u8,
                    path_wide.len() * 2
                )
            };
            
            let _ = RegSetValueExW(
                hkey,
                &HSTRING::from("Clutch"),
                0,
                REG_SZ,
                Some(data)
            );
            
            let _ = RegCloseKey(hkey);
        }
    }
}
