use std::collections::HashMap;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOZORDER, SWP_NOACTIVATE, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use crate::layout::algorithm::tiled::dwindle::DwindleAlgorithm::{DwindleNode, Rect};
use crate::helpers::Types::SendHWND;
use crate::desktop::Window::WindowState;
use crate::desktop::Workspace::{Workspace, LayoutType};
use crate::Compositor::KB;

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
                root: None,
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
        let send_hwnd = SendHWND(hwnd);
        let state = WindowState {
            hwnd: send_hwnd,
            title,
            class_name,
            is_floating: false,
            workspace_id: self.active_workspace,
        };
        
        self.active_windows.insert(hwnd.0 as isize, state);
        
        if let Some(ws) = self.workspaces.iter_mut().find(|w| w.id == self.active_workspace) {
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
            if let Some(root) = &ws.root {
                let mut results = Vec::new();
                root.get_layout_results(&mut results, gaps_in, gaps_out);

                for (send_hwnd, rect) in results {
                    unsafe {
                        crate::render::Renderer::CreateBorderWrapper(send_hwnd, 2, 10.0);
                        crate::render::Renderer::UpdateBorderPositionWrapper(send_hwnd, rect.x, rect.y, rect.width, rect.height);

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
    }
}
