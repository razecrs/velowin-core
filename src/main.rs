pub mod manager;
pub mod config;
pub mod dwindle;
pub mod keybinds;

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::Accessibility::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Input::KeyboardAndMouse::*,
};
use crate::manager::WindowManager;
use crate::keybinds::KeybindManager;
use crate::config::parse_config;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

static WM: Lazy<Arc<Mutex<WindowManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(WindowManager::new()))
});

static KB: Lazy<Arc<Mutex<KeybindManager>>> = Lazy::new(|| {
    // dummy config for now, in a real app we'd read ~/.config/hypr/hyprland.conf
    let dummy_conf = "
        bind = SUPER, Return, exec, start alacritty
        bind = SUPER, Q, killactive,
        exec-once = start velowin-bar
    ";
    Arc::new(Mutex::new(KeybindManager::new(parse_config(dummy_conf))))
});

unsafe extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kbd = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
        
        if w_param.0 == WM_KEYDOWN as usize || w_param.0 == WM_SYSKEYDOWN as usize {
            let mut mods = 0;
            unsafe {
                if (GetKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0 || (GetKeyState(VK_RWIN.0 as i32) as u16 & 0x8000) != 0 {
                    mods |= 0x0008; // Super
                }
                if (GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0 {
                    mods |= 0x0004;
                }
                if (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0 {
                    mods |= 0x0002;
                }
                if (GetKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0 {
                    mods |= 0x0001; // Alt
                }
            }

            let kb = KB.lock().unwrap();
            if kb.handle_key(kbd.vkCode, mods) {
                return LRESULT(1); // consume the key
            }
        }
    }
    unsafe { CallNextHookEx(HHOOK(std::ptr::null_mut()), code, w_param, l_param) }
}

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

        let kb_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_proc),
            HINSTANCE(std::ptr::null_mut()),
            0,
        ).map_err(|_| Error::from_win32())?;

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
