// src/window.rs
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::Controls::*;
use std::ffi::c_void;

pub type DropCallback = Box<dyn Fn(Vec<String>) + Send + Sync>;

pub fn setup_window(hwnd: HWND, on_drop: DropCallback) {
    unsafe {
        // Make window always on top
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE,
        ).ok();

        // Remove window decorations
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        SetWindowLongW(
            hwnd,
            GWL_STYLE,
            style & !(WS_OVERLAPPEDWINDOW.0 as i32),
        );

        // Enable layering (required for transparency)
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            ex_style | WS_EX_LAYERED.0 as i32,
        );

        // Enable drag and drop
        DragAcceptFiles(hwnd, true);

        // Subclass window to handle WM_DROPFILES
        let callback_ptr = Box::into_raw(Box::new(on_drop));
        SetWindowSubclass(hwnd, Some(subclass_proc), 1, callback_ptr as usize);
    }
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    ref_data: usize,
) -> LRESULT {
    if msg == WM_DROPFILES {
        let hdrop = HDROP(wparam.0 as *mut c_void);
        let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
        let mut files = Vec::new();

        for i in 0..count {
            let mut buffer = [0u16; 512];
            let len = DragQueryFileW(hdrop, i, Some(&mut buffer));
            if len > 0 {
                let path = String::from_utf16_lossy(&buffer[..len as usize]);
                files.push(path);
            }
        }

        DragFinish(hdrop);

        // Call the callback
        let callback = &*(ref_data as *const DropCallback);
        callback(files);

        return LRESULT(0);
    }

    if msg == WM_NCDESTROY {
        // Cleanup callback
        let _ = Box::from_raw(ref_data as *mut DropCallback);
        RemoveWindowSubclass(hwnd, Some(subclass_proc), 1);
    }

    DefSubclassProc(hwnd, msg, wparam, lparam)
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
            let data = std::slice::from_raw_parts(
                path_wide.as_ptr() as *const u8,
                path_wide.len() * 2
            );
            
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
