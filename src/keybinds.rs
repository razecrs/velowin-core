use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::*;
use std::process::Command;
use crate::config::Config;

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
                return true; // handled
            }
        }
        false
    }

    fn match_key(&self, key_str: &str, vk_code: u32) -> bool {
        // basic mapping for now, should be expanded to full 1:1 hyprland keys
        match key_str.to_uppercase().as_str() {
            "RETURN" | "ENTER" => vk_code == VK_RETURN.0 as u32,
            "Q" => vk_code == 'Q' as u32,
            "SPACE" => vk_code == VK_SPACE.0 as u32,
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
                println!("[Exec] Running: {}", arg);
                let _ = Command::new("cmd").args(&["/C", arg]).spawn();
            }
            "killactive" => {
                println!("[Dispatch] killactive");
                // TODO: wm.kill_active()
            }
            _ => println!("[Dispatch] Unknown: {}", dispatcher),
        }
    }
}
