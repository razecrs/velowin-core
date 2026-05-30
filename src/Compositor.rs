use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::Accessibility::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Input::KeyboardAndMouse::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
};
use crate::managers::WindowManager::WindowManager;
use crate::managers::KeybindManager::KeybindManager;
use crate::managers::animation::AnimationManager::AnimationManager;
use crate::config::ConfigManager::parse_config;
use crate::render::Renderer;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

pub static WM: Lazy<Arc<Mutex<WindowManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(WindowManager::new()))
});

pub static KB: Lazy<Arc<Mutex<KeybindManager>>> = Lazy::new(|| {
    let dummy_conf = "
        bind = SUPER, Return, exec, start alacritty
        bind = SUPER, Q, killactive,
        exec-once = start velowin-bar
    ";
    Arc::new(Mutex::new(KeybindManager::new(parse_config(dummy_conf))))
});

pub static ANIMATION_MANAGER: Lazy<Arc<AnimationManager>> = Lazy::new(|| {
    Arc::new(AnimationManager::new())
});

pub fn init() -> Result<()> {
    unsafe {
        let overlay_hwnd = create_overlay_window()?;
        
        if !Renderer::InitCompositor(overlay_hwnd) {
            println!("Failed to initialize DirectX/DirectComposition renderer.");
            return Err(Error::from_win32());
        }
        println!("DirectComposition Renderer Initialized.");

        // Start Animation Tick Thread
        std::thread::spawn(|| {
            loop {
                ANIMATION_MANAGER.tick();
                std::thread::sleep(std::time::Duration::from_millis(16)); // ~60fps
            }
        });

        let _kb_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_proc),
            GetModuleHandleW(None)?,
            0,
        ).map_err(|_| Error::from_win32())?;

        let _event_hook = SetWinEventHook(
            EVENT_OBJECT_CREATE,
            EVENT_OBJECT_DESTROY,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );

        let _foreground_hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
    }
    Ok(())
}

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
    if id_object != OBJID_WINDOW.0 || id_child != CHILDID_SELF as i32 || hwnd.0.is_null() {
        return;
    }

    let mut wm = WM.lock().unwrap();

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
            let mut wm = WM.lock().unwrap();
            // TODO: set active window, trigger focus animations
            if let Some(window) = wm.active_windows.get_mut(&(hwnd.0 as isize)) {
                // fade in active window (just a test animation)
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

unsafe extern "system" fn overlay_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
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
