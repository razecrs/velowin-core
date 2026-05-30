use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Keybind {
    pub mods: u32,
    pub key: String,
    pub dispatcher: String,
    pub arg: String,
}

#[derive(Debug, Clone)]
pub struct WindowRule {
    pub rule: String,
    pub regex: String,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub general: HashMap<String, String>,
    pub decoration: HashMap<String, String>,
    pub animations: HashMap<String, String>,
    pub binds: Vec<Keybind>,
    pub window_rules: Vec<WindowRule>,
    pub exec_once: Vec<String>,
}

impl Config {
    pub fn get_int(&self, section: &str, key: &str, default: i32) -> i32 {
        let map = match section {
            "general" => &self.general,
            "decoration" => &self.decoration,
            _ => return default,
        };
        map.get(key).and_then(|s| s.parse().ok()).unwrap_or(default)
    }
}

pub fn parse_config(content: &str) -> Config {
    let mut config = Config::default();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((directive, body)) = line.split_once('=') {
            let directive = directive.trim();
            let body = body.trim();

            match directive {
                "bind" => {
                    let parts: Vec<&str> = body.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 3 {
                        config.binds.push(Keybind {
                            mods: parse_mods(parts[0]),
                            key: parts[1].to_string(),
                            dispatcher: parts[2].to_string(),
                            arg: parts.get(3).unwrap_or(&"").to_string(),
                        });
                    }
                }
                "windowrule" => {
                    let parts: Vec<&str> = body.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 2 {
                        config.window_rules.push(WindowRule {
                            rule: parts[0].to_string(),
                            regex: parts[1].to_string(),
                        });
                    }
                }
                "exec-once" => {
                    config.exec_once.push(body.to_string());
                }
                _ => {
                    // basic section handling (not nested for simplicity here)
                }
            }
        }
    }

    config
}

fn parse_mods(input: &str) -> u32 {
    let mut mods = 0;
    for part in input.split('&') {
        match part.trim().to_uppercase().as_str() {
            "SUPER" | "MOD4" | "WIN" => mods |= 0x0008,
            "SHIFT" => mods |= 0x0004,
            "CTRL" | "CONTROL" => mods |= 0x0002,
            "ALT" => mods |= 0x0001,
            _ => {}
        }
    }
    mods
}
