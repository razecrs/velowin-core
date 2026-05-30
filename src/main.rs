pub mod manager;
pub mod config;
pub mod dwindle;

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::Accessibility::*,
    Win32::UI::WindowsAndMessaging::*,
};
use crate::manager::WindowManager;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

static WM: Lazy<Arc<Mutex<WindowManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(WindowManager::new()))
});

unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    // ignore random child elements and ghosts, we only care about actual windows
    if id_object != OBJID_WINDOW.0 || id_child != CHILDID_SELF as i32 || hwnd.0.is_null() {
        return;
    }

    let mut wm = WM.lock().unwrap(); // TODO: maybe don't block the whole thread here later

    match event {
        EVENT_OBJECT_CREATE => {
            if is_top_level_window(hwnd) {
                let title = get_window_title(hwnd);
                let class_name = get_window_class(hwnd);
                println!("[Created] {} ({})", title, class_name);
                wm.add_window(hwnd, title, class_name);
            }
        }
        EVENT_OBJECT_DESTROY => {
            wm.remove_window(hwnd);
        }
        EVENT_SYSTEM_FOREGROUND => {
            println!("[Foreground] {:?}", hwnd);
        }
        _ => {}
    }
}

fn is_top_level_window(hwnd: HWND) -> bool {
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        let is_visible = IsWindowVisible(hwnd).as_bool();
        let is_child = (style & WS_CHILD.0) != 0;
        let is_tool_window = (ex_style & WS_EX_TOOLWINDOW.0) != 0;
        is_visible && !is_child && !is_tool_window
    }
}

fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let mut text = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut text);
        String::from_utf16_lossy(&text[..len as usize])
    }
}

fn get_window_class(hwnd: HWND) -> String {
    unsafe {
        let mut text = [0u16; 512];
        let len = GetClassNameW(hwnd, &mut text);
        String::from_utf16_lossy(&text[..len as usize])
    }
}

fn main() -> Result<()> {
    unsafe {
        println!("Velowin: Starting 1:1 Hyprland-like WM...");

        let hook = SetWinEventHook(
            EVENT_OBJECT_CREATE,
            EVENT_OBJECT_DESTROY,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );

        let foreground_hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }

        let _ = UnhookWinEvent(hook);
        let _ = UnhookWinEvent(foreground_hook);
    }

    Ok(())
}
