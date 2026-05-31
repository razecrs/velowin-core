use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::Accessibility::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Input::KeyboardAndMouse::*,
    Win32::UI::HiDpi::*,
    Win32::Graphics::Gdi::*,
    Win32::Graphics::Dwm::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
};
use crate::managers::WindowManager::WindowManager;
use crate::managers::KeybindManager::KeybindManager;
use crate::managers::MonitorManager::MonitorManager;
use crate::managers::animation::AnimationManager::AnimationManager;
use crate::config::ConfigManager::parse_config;
use crate::render::Renderer;
use crate::helpers::Types::{SendHHOOK, SendHWINEVENTHOOK};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

pub static WM: Lazy<Arc<Mutex<WindowManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(WindowManager::new()))
});

pub static KB: Lazy<Arc<Mutex<KeybindManager>>> = Lazy::new(|| {
    let dummy_conf = "
        bind = SUPER, Return, exec, powershell
        bind = SUPER, Q, killactive,
        bind = SUPER, V, togglefloating,
        exec-once = start velowin-bar
    ";
    Arc::new(Mutex::new(KeybindManager::new(parse_config(dummy_conf))))
});

pub static MN: Lazy<Arc<Mutex<MonitorManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(MonitorManager::new()))
});

pub static ANIMATION_MANAGER: Lazy<Arc<AnimationManager>> = Lazy::new(|| {
    Arc::new(AnimationManager::new())
});

static KB_HOOK: Mutex<Option<SendHHOOK>> = Mutex::new(None);
static EVENT_HOOK: Mutex<Option<SendHWINEVENTHOOK>> = Mutex::new(None);
static FOREGROUND_HOOK: Mutex<Option<SendHWINEVENTHOOK>> = Mutex::new(None);

pub fn init() -> Result<()> {
    unsafe {
        crate::helpers::Logger::init();
        crate::velowin_log!("Velowin: Starting 1:1 Hyprland-like WM...");

        // DOC-DRIVEN: Set Process DPI Awareness correctly for multi-monitor V2
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let overlay_hwnd = create_overlay_window()?;
        
        if !Renderer::InitCompositor(overlay_hwnd) {
            crate::velowin_log!("Failed to initialize DirectX/DirectComposition renderer.");
            return Err(Error::from_win32());
        }
        crate::velowin_log!("DirectComposition Renderer Initialized.");

        MN.lock().unwrap().refresh();
        scan_existing_windows();

        std::thread::spawn(|| {
            loop {
                ANIMATION_MANAGER.tick();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        });

        // LOCKING IN THE HOOKS (Keeping them in static memory)
        let hmodule = GetModuleHandleW(None).unwrap_or_default();
        let kb_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), HINSTANCE(hmodule.0), 0)
            .map_err(|_| Error::from_win32())?;
        *KB_HOOK.lock().unwrap() = Some(SendHHOOK(kb_hook));

        let event_hook = SetWinEventHook(EVENT_OBJECT_CREATE, EVENT_OBJECT_LOCATIONCHANGE, None, Some(win_event_proc), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS);
        if !event_hook.0.is_null() { *EVENT_HOOK.lock().unwrap() = Some(SendHWINEVENTHOOK(event_hook)); }

        let fg_hook = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, None, Some(win_event_proc), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS);
        if !fg_hook.0.is_null() { *FOREGROUND_HOOK.lock().unwrap() = Some(SendHWINEVENTHOOK(fg_hook)); }

        crate::velowin_log!("Hooks installed and persistence locked.");
    }
    Ok(())
}

unsafe fn scan_existing_windows() {
    crate::velowin_log!("Scanning existing windows...");
    let _ = unsafe { EnumWindows(Some(enum_windows_proc), LPARAM(0)) };
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
    if is_top_level_window(hwnd) {
        let title = get_window_title(hwnd);
        let class_name = get_window_class(hwnd);
        crate::velowin_log!("[Found] {} ({})", title, class_name);
        let mut wm = WM.lock().unwrap();
        wm.add_window(hwnd, title, class_name);
    }
    true.into()
}

unsafe extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kbd = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
        
        if w_param.0 == WM_KEYDOWN as usize || w_param.0 == WM_SYSKEYDOWN as usize {
            let mut mods = 0;
            unsafe {
                if (GetAsyncKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0 || (GetAsyncKeyState(VK_RWIN.0 as i32) as u16 & 0x8000) != 0 {
                    mods |= 0x0008; 
                }
                if (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0 {
                    mods |= 0x0004;
                }
                if (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0 {
                    mods |= 0x0002;
                }
                if (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0 {
                    mods |= 0x0001; 
                }
            }

            if mods != 0 {
                // Heartbeat log: see if the keypress is detected
                // crate::velowin_log!("[Input] Key: {:X}, Mods: {}", kbd.vkCode, mods);
                
                let kb = KB.lock().unwrap();
                if kb.handle_key(kbd.vkCode, mods) {
                    return LRESULT(1); 
                }
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

fn is_top_level_window(hwnd: HWND) -> bool {
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

        // DOC-DRIVEN: Check if window is cloaked (on another desktop or suspended)
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

unsafe extern "system" fn overlay_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn create_overlay_window() -> Result<HWND> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(overlay_wnd_proc),
            hInstance: instance.into(),
            lpszClassName: w!("VelowinOverlay"),
            style: CS_HREDRAW | CS_VREDRAW,
            hbrBackground: HBRUSH(GetStockObject(HOLLOW_BRUSH).0),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            return Err(Error::from_win32());
        }

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            w!("VelowinOverlay"),
            w!("Velowin Overlay"),
            WS_POPUP,
            0, 0, GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN),
            None,
            None,
            instance,
            None,
        )?;

        SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)?;
        let _ = ShowWindow(hwnd, SW_SHOW);

        Ok(hwnd)
    }
}
