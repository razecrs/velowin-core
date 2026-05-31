use windows::Win32::Foundation::*;
use windows::Win32::UI::Accessibility::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Dwm::*;
use crate::Compositor::WM;
use crate::managers::WindowManager;

pub unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if id_object != OBJID_WINDOW.0 || id_child != CHILDID_SELF as i32 || hwnd.0.is_null() {
        return;
    }

    match event {
        EVENT_OBJECT_CREATE | EVENT_OBJECT_SHOW => {
            if is_top_level_window(hwnd) {
                let mut wm = WM.lock().unwrap();
                if !wm.active_windows.contains_key(&(hwnd.0 as isize)) {
                    let title = get_window_title(hwnd);
                    let class_name = get_window_class(hwnd);
                    crate::velowin_log!("[Created/Shown] {} ({})", title, class_name);
                    wm.add_window(hwnd, title, class_name);
                }
            }
        }
        EVENT_OBJECT_DESTROY => {
            let mut wm = WM.lock().unwrap();
            wm.remove_window(hwnd);
        }
        EVENT_SYSTEM_FOREGROUND => {
            let mut wm = WM.lock().unwrap();
            if let Some(window) = wm.active_windows.get_mut(&(hwnd.0 as isize)) {
                window.opacity.set(1.0);
            }
        }
        _ => {}
    }
}

pub fn is_top_level_window(hwnd: HWND) -> bool {
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        let is_visible = IsWindowVisible(hwnd).as_bool();
        
        let mut class_text = [0u16; 512];
        let len = GetClassNameW(hwnd, &mut class_text);
        let class_name = String::from_utf16_lossy(&class_text[..len as usize]);
        let title = get_window_title(hwnd);

        if !is_visible || (style & WS_CHILD.0) != 0 { return false; }
        if (ex_style & WS_EX_TOOLWINDOW.0) != 0 { return false; }

        let mut cloaked: i32 = 0;
        let _ = DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut std::ffi::c_void, 4);
        if cloaked != 0 { return false; }

        let ignored_classes = [
            "Xaml_WindowedPopupClass",
            "tooltips_class32",
            "DroppyClass",
            "ApplicationFrameTitleBarWindow",
            "GhostWindow",
            "Windows.UI.Core.CoreWindow",
            "Shell_TrayWnd",
            "Progman",
        ];

        if ignored_classes.iter().any(|&c| class_name.contains(c)) {
            return false;
        }

        if class_name.contains("ApplicationFrameWindow") && title.is_empty() {
            return false;
        }

        if title.is_empty() || title == "Windows Input Experience" || title == "Realtek Audio Console" {
            return false;
        }

        true
    }
}

pub fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let mut text = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut text);
        String::from_utf16_lossy(&text[..len as usize])
    }
}

pub fn get_window_class(hwnd: HWND) -> String {
    unsafe {
        let mut text = [0u16; 512];
        let len = GetClassNameW(hwnd, &mut text);
        String::from_utf16_lossy(&text[..len as usize])
    }
}
