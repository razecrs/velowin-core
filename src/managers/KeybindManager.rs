use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::*;
use std::process::Command;
use crate::config::ConfigManager::Config;
use crate::Compositor::WM;

pub struct KeybindManager {
    pub config: Config,
}

impl KeybindManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn handle_key(&self, vk_code: u32, modifiers: u32) -> bool {
        for bind in &self.config.binds {
            if bind.mods == modifiers && self.match_key(&bind.key, vk_code) {
                self.dispatch(&bind.dispatcher, &bind.arg);
                return true; 
            }
        }
        false
    }

    fn match_key(&self, key_str: &str, vk_code: u32) -> bool {
        match key_str.to_uppercase().as_str() {
            "RETURN" | "ENTER" => vk_code == VK_RETURN.0 as u32,
            "Q" => vk_code == 'Q' as u32,
            "SPACE" => vk_code == VK_SPACE.0 as u32,
            "V" => vk_code == 'V' as u32,
            "F" => vk_code == 'F' as u32,
            _ => {
                if key_str.len() == 1 {
                    vk_code == key_str.chars().next().unwrap().to_ascii_uppercase() as u32
                } else {
                    false
                }
            }
        }
    }

    fn dispatch(&self, dispatcher: &str, arg: &str) {
        match dispatcher {
            "exec" => {
                crate::velowin_log!("[Exec] Running: {}", arg);
                let _ = Command::new("cmd").args(&["/C", arg]).spawn();
            }
            "togglefloating" => {
                crate::velowin_log!("[Dispatch] togglefloating");
                unsafe {
                    let hwnd = GetForegroundWindow();
                    if !hwnd.0.is_null() {
                        let mut wm = WM.lock().unwrap();
                        wm.toggle_tiling(hwnd);
                    }
                }
            }
            "killactive" => {
                crate::velowin_log!("[Dispatch] killactive");
                unsafe {
                    let hwnd = GetForegroundWindow();
                    if !hwnd.0.is_null() {
                        let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                    }
                }
            }
            _ => crate::velowin_log!("[Dispatch] Unknown: {}", dispatcher),
        }
    }
}
