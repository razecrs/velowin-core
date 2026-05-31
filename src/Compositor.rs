use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::Accessibility::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::HiDpi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
};
use crate::managers::WindowManager::WindowManager;
use crate::managers::KeybindManager::{KeybindManager, keyboard_proc};
use crate::managers::EventManager::win_event_proc;
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

        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let overlay_hwnd = Renderer::create_overlay_window()?;
        
        if !Renderer::InitCompositor(overlay_hwnd) {
            crate::velowin_log!("Failed to initialize DirectX/DirectComposition renderer.");
            return Err(Error::from_win32());
        }
        crate::velowin_log!("DirectComposition Renderer Initialized.");

        // HIDE NATIVE WINDOWS TASKBAR
        if let Ok(tray) = FindWindowW(w!("Shell_TrayWnd"), None) {
            if !tray.0.is_null() {
                let _ = ShowWindow(tray, SW_HIDE);
                crate::velowin_log!("Native Windows Taskbar Hidden.");
            }
        }

        MN.lock().unwrap().refresh();
        scan_existing_windows();

        std::thread::spawn(|| {
            loop {
                ANIMATION_MANAGER.tick();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        });

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
    if crate::managers::EventManager::is_top_level_window(hwnd) {
        let title = crate::managers::EventManager::get_window_title(hwnd);
        let class_name = crate::managers::EventManager::get_window_class(hwnd);
        crate::velowin_log!("[Found] {} ({})", title, class_name);
        let mut wm = WM.lock().unwrap();
        wm.add_window(hwnd, title, class_name);
    }
    true.into()
}
