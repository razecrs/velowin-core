use std::collections::HashMap;
use windows::Win32::Foundation::HWND;

#[derive(Debug, Clone)]
pub struct WindowState {
    pub hwnd: HWND,
    pub title: String,
    pub class_name: String,
    pub is_floating: bool,
    pub workspace_id: u32,
}

pub struct Workspace {
    pub id: u32,
    pub windows: Vec<HWND>,
    pub layout: LayoutType,
}

pub enum LayoutType {
    Dwindle, // Hyprland default
    Master,
}

pub struct WindowManager {
    pub active_windows: HashMap<isize, WindowState>,
    pub workspaces: Vec<Workspace>,
    pub active_workspace: u32,
}

impl WindowManager {
    pub fn new() -> Self {
        let mut workspaces = Vec::new();
        for i in 1..=10 {
            workspaces.push(Workspace {
                id: i,
                windows: Vec::new(),
                layout: LayoutType::Dwindle,
            });
        }
        
        Self {
            active_windows: HashMap::new(),
            workspaces,
            active_workspace: 1,
        }
    }

    pub fn add_window(&mut self, hwnd: HWND, title: String, class_name: String) {
        let state = WindowState {
            hwnd,
            title,
            class_name,
            is_floating: false,
            workspace_id: self.active_workspace,
        };
        
        self.active_windows.insert(hwnd.0 as isize, state);
        
        if let Some(ws) = self.workspaces.iter_mut().find(|w| w.id == self.active_workspace) {
            ws.windows.push(hwnd);
        }
        
        self.recalculate_layout();
    }

    pub fn remove_window(&mut self, hwnd: HWND) {
        self.active_windows.remove(&(hwnd.0 as isize));
        for ws in &mut self.workspaces {
            ws.windows.retain(|&h| h != hwnd);
        }
        self.recalculate_layout();
    }

    pub fn recalculate_layout(&self) {
        // TODO: Implement BSP (Dwindle) layout algorithm
        println!("Recalculating layout for workspace {}", self.active_workspace);
    }
}
