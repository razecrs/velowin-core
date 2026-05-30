use std::collections::HashMap;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOZORDER, SWP_NOACTIVATE, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use crate::layout::algorithm::tiled::dwindle::DwindleAlgorithm::{DwindleNode, Rect};
use crate::layout::algorithm::tiled::master::MasterLayout::MasterLayout;
use crate::helpers::Types::SendHWND;
use crate::desktop::Window::WindowState;
use crate::desktop::Workspace::{Workspace, LayoutType};
use crate::Compositor::KB;

pub struct WindowManager {
    pub active_windows: HashMap<isize, WindowState>,
    pub workspaces: Vec<Workspace>,
    pub active_workspace: u32,
    pub master_layout: MasterLayout, // temporary global for testing
}

impl WindowManager {
    pub fn new() -> Self {
        let mut workspaces = Vec::new();
        for i in 1..=10 {
            workspaces.push(Workspace {
                id: i,
                root: None,
                layout: LayoutType::Dwindle,
            });
        }
        
        Self {
            active_windows: HashMap::new(),
            workspaces,
            active_workspace: 1,
            master_layout: MasterLayout::new(),
        }
    }

    pub fn add_window(&mut self, hwnd: HWND, title: String, class_name: String) {
        let send_hwnd = SendHWND(hwnd);
        let mut state = WindowState::new(send_hwnd, title, class_name, self.active_workspace);
        
        // Apply window rules (Hyprland style)
        let config = KB.lock().unwrap().config.clone();
        for rule in &config.window_rules {
            if state.title.contains(&rule.regex) || state.class_name.contains(&rule.regex) {
                match rule.rule.as_str() {
                    "float" => state.is_floating = true,
                    "workspace 4" => state.workspace_id = 4, // placeholder
                    _ => {}
                }
            }
        }

        state.border_angle.set(360.0); 
        self.active_windows.insert(hwnd.0 as isize, state);
        
        if let Some(ws) = self.workspaces.iter_mut().find(|w| w.id == self.active_workspace) {
            match ws.layout {
                LayoutType::Dwindle => {
                    match &mut ws.root {
                        None => {
                            unsafe {
                                let sw = GetSystemMetrics(SM_CXSCREEN);
                                let sh = GetSystemMetrics(SM_CYSCREEN);
                                ws.root = Some(DwindleNode::new_leaf(send_hwnd, Rect { x: 0, y: 0, width: sw, height: sh }));
                            }
                        }
                        Some(root) => {
                            root.split(send_hwnd);
                        }
                    }
                }
                LayoutType::Master => {
                    self.master_layout.add_window(send_hwnd);
                }
            }
        }
        
        self.recalculate_layout();
    }

    pub fn remove_window(&mut self, hwnd: HWND) {
        self.active_windows.remove(&(hwnd.0 as isize));
        self.recalculate_layout();
    }

    pub fn recalculate_layout(&self) {
        let config = KB.lock().unwrap().config.clone();
        let gaps_in = config.get_int("general", "gaps_in", 5);
        let gaps_out = config.get_int("general", "gaps_out", 20);

        if let Some(ws) = self.workspaces.iter().find(|w| w.id == self.active_workspace) {
            match ws.layout {
                LayoutType::Dwindle => {
                    if let Some(root) = &ws.root {
                        let mut results = Vec::new();
                        root.get_layout_results(&mut results, gaps_in, gaps_out);
                        self.apply_layout_results(results);
                    }
                }
                LayoutType::Master => {
                    // TODO: integrate master layout properly into workspaces
                }
            }
        }
    }

    fn apply_layout_results(&self, results: Vec<(SendHWND, Rect)>) {
        for (send_hwnd, rect) in results {
            unsafe {
                if let Some(window) = self.active_windows.get(&(send_hwnd.0.0 as isize)) {
                    crate::render::Renderer::CreateBorderWrapper(send_hwnd, 2, 10.0);
                    crate::render::Renderer::SetBorderAngleWrapper(send_hwnd, window.border_angle.value);
                    crate::render::Renderer::UpdateBorderPositionWrapper(send_hwnd, rect.x, rect.y, rect.width, rect.height);
                }

                let _ = SetWindowPos(
                    send_hwnd.0,
                    HWND(std::ptr::null_mut()),
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }
}
