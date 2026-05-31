use std::collections::HashMap;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::layout::algorithm::tiled::dwindle::DwindleAlgorithm::{DwindleNode, Rect};
use crate::layout::algorithm::tiled::master::MasterLayout::MasterLayout;
use crate::helpers::Types::SendHWND;
use crate::desktop::Window::WindowState;
use crate::desktop::Workspace::{Workspace, LayoutType};
use crate::Compositor::{KB, MN};

pub struct WindowManager {
    pub active_windows: HashMap<isize, WindowState>,
    pub workspaces: Vec<Workspace>,
    pub active_workspace: u32,
    pub master_layout: MasterLayout,
    pub gaps_in: i32,
    pub gaps_out: i32,
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
            gaps_in: 0,
            gaps_out: 0,
        }
    }

    pub fn set_gaps(&mut self, r#in: i32, out: i32) {
        self.gaps_in = r#in;
        self.gaps_out = out;
    }

    pub fn add_window(&mut self, hwnd: HWND, title: String, class_name: String) {
        let send_hwnd = SendHWND(hwnd);
        let mut state = WindowState::new(send_hwnd, title, class_name, self.active_workspace);
        state.is_floating = true;
        self.active_windows.insert(hwnd.0 as isize, state);
        self.recalculate_layout();
    }

    fn tile_window(&mut self, hwnd: HWND) {
        let send_hwnd = SendHWND(hwnd);
        if let Some(ws) = self.workspaces.iter_mut().find(|w| w.id == self.active_workspace) {
            match &mut ws.root {
                None => {
                    let mn = MN.lock().unwrap();
                    let monitor = mn.get_monitor_for_window(hwnd).cloned().unwrap_or_default();
                    let wr = monitor.work_rect;
                    ws.root = Some(DwindleNode::new_leaf(send_hwnd, Rect { 
                        x: wr.left, y: wr.top, width: wr.right - wr.left, height: wr.bottom - wr.top 
                    }));
                }
                Some(root) => {
                    root.split(send_hwnd);
                }
            }
        }
    }

    pub fn toggle_tiling(&mut self, hwnd: HWND) {
        let top_hwnd = unsafe { GetAncestor(hwnd, GA_ROOTOWNER) };
        let target = if top_hwnd.0.is_null() { hwnd } else { top_hwnd };

        if let Some(state) = self.active_windows.get_mut(&(target.0 as isize)) {
            state.is_floating = !state.is_floating;
            let is_now_floating = state.is_floating;
            
            if is_now_floating {
                crate::velowin_log!("[WM] Window '{}' is now FLOATING", state.title);
                self.remove_from_layout(target);
                self.restore_decorations(target);
            } else {
                crate::velowin_log!("[WM] Window '{}' is now TILING", state.title);
                self.tile_window(target);
            }
            self.recalculate_layout();
        }
    }

    fn remove_from_layout(&mut self, hwnd: HWND) {
        let send_hwnd = SendHWND(hwnd);
        if let Some(ws) = self.workspaces.iter_mut().find(|w| w.id == self.active_workspace) {
            if let Some(mut root) = ws.root.take() {
                if !root.remove(send_hwnd) {
                    ws.root = Some(root);
                }
            }
        }
    }

    fn restore_decorations(&self, hwnd: HWND) {
        unsafe {
            let mut style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            style |= WS_CAPTION.0 | WS_THICKFRAME.0 | WS_SYSMENU.0;
            let _ = SetWindowLongW(hwnd, GWL_STYLE, style as i32);
            let _ = SetWindowPos(hwnd, HWND(std::ptr::null_mut()), 0, 0, 0, 0, 
                SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED);
        }
    }

    pub fn remove_window(&mut self, hwnd: HWND) {
        self.restore_decorations(hwnd);
        self.active_windows.remove(&(hwnd.0 as isize));
        self.remove_from_layout(hwnd);
        self.recalculate_layout();
    }

    pub fn recalculate_layout(&mut self) {
        if let Some(ws) = self.workspaces.iter().find(|w| w.id == self.active_workspace) {
            if let Some(root) = &ws.root {
                let mut results = Vec::new();
                root.get_layout_results(&mut results, self.gaps_in, self.gaps_out);
                self.apply_layout_results(results);
            }
        }
    }

    fn apply_layout_results(&mut self, results: Vec<(SendHWND, Rect)>) {
        unsafe {
            let hdwp = BeginDeferWindowPos(results.len() as i32).expect("BeginDeferWindowPos failed");
            let mut current_hdwp = hdwp;

            for (send_hwnd, rect) in results {
                // 1. DWM Policy & Animation Control
                let rendering_policy = windows::Win32::Graphics::Dwm::DWMNCRP_DISABLED;
                let _ = windows::Win32::Graphics::Dwm::DwmSetWindowAttribute(
                    send_hwnd.0,
                    windows::Win32::Graphics::Dwm::DWMWA_NCRENDERING_POLICY,
                    &rendering_policy as *const _ as *const std::ffi::c_void,
                    4,
                );

                let disable_anim: i32 = 1;
                let _ = windows::Win32::Graphics::Dwm::DwmSetWindowAttribute(
                    send_hwnd.0,
                    windows::Win32::Graphics::Dwm::DWMWA_TRANSITIONS_FORCEDISABLED,
                    &disable_anim as *const _ as *const std::ffi::c_void,
                    4,
                );

                // 2. Strip Decorations & Redraw
                let mut style = GetWindowLongW(send_hwnd.0, GWL_STYLE) as u32;
                if (style & WS_CAPTION.0) != 0 {
                    style &= !(WS_CAPTION.0 | WS_THICKFRAME.0 | WS_SYSMENU.0);
                    let _ = SetWindowLongW(send_hwnd.0, GWL_STYLE, style as i32);
                }

                let _ = ShowWindow(send_hwnd.0, SW_RESTORE);

                // 3. Extended Frame Bounds Alignment (Tight Fit)
                let mut w_rect = windows::Win32::Foundation::RECT::default();
                let mut f_rect = windows::Win32::Foundation::RECT::default();
                let _ = GetWindowRect(send_hwnd.0, &mut w_rect);
                let _ = windows::Win32::Graphics::Dwm::DwmGetWindowAttribute(
                    send_hwnd.0, 
                    windows::Win32::Graphics::Dwm::DWMWA_EXTENDED_FRAME_BOUNDS, 
                    &mut f_rect as *mut _ as *mut std::ffi::c_void, 
                    std::mem::size_of::<windows::Win32::Foundation::RECT>() as u32
                );

                let off_x = f_rect.left - w_rect.left;
                let off_y = f_rect.top - w_rect.top;
                let off_w = (w_rect.right - w_rect.left) - (f_rect.right - f_rect.left);
                let off_h = (w_rect.bottom - w_rect.top) - (f_rect.bottom - f_rect.top);

                if let Some(window) = self.active_windows.get_mut(&(send_hwnd.0.0 as isize)) {
                    window.x.set(rect.x - off_x);
                    window.y.set(rect.y - off_y);
                    window.width.set(rect.width + off_w);
                    window.height.set(rect.height + off_h);
                    
                    crate::render::Renderer::CreateBorderWrapper(send_hwnd, 2, 10.0);
                    crate::render::Renderer::SetBorderAngleWrapper(send_hwnd, window.border_angle.value);
                    crate::render::Renderer::UpdateBorderPositionWrapper(send_hwnd, rect.x, rect.y, rect.width, rect.height);
                }

                // Use SWP_FRAMECHANGED to force non-client area update (fixes taskbar overlap)
                current_hdwp = DeferWindowPos(current_hdwp, send_hwnd.0, HWND(std::ptr::null_mut()), 
                    rect.x - off_x, rect.y - off_y, rect.width + off_w, rect.height + off_h, 
                    SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED).expect("DeferWindowPos failed");
            }
            let _ = EndDeferWindowPos(current_hdwp);
        }
    }
}
