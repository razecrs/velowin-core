use std::collections::HashMap;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOZORDER, SWP_NOACTIVATE};
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
        
        let config = KB.lock().unwrap().config.clone();
        for rule in &config.window_rules {
            if state.title.contains(&rule.regex) || state.class_name.contains(&rule.regex) {
                match rule.rule.as_str() {
                    "float" => state.is_floating = true,
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
                            let mn = MN.lock().unwrap();
                            let monitor = mn.get_monitor_for_window(hwnd).cloned().unwrap_or_else(|| {
                                mn.monitors.iter().find(|m| m.is_primary).cloned().unwrap_or_else(|| {
                                    crate::managers::MonitorManager::Monitor {
                                        hmonitor: crate::managers::MonitorManager::SendHMONITOR(windows::Win32::Graphics::Gdi::HMONITOR(std::ptr::null_mut())),
                                        rect: windows::Win32::Foundation::RECT::default(),
                                        work_rect: windows::Win32::Foundation::RECT { left: 0, top: 0, right: 1920, bottom: 1080 },
                                        dpi: 96,
                                        is_primary: true,
                                    }
                                })
                            });

                            let wr = monitor.work_rect;
                            ws.root = Some(DwindleNode::new_leaf(send_hwnd, Rect { 
                                x: wr.left, 
                                y: wr.top, 
                                width: wr.right - wr.left, 
                                height: wr.bottom - wr.top 
                            }));
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
        let send_hwnd = SendHWND(hwnd);
        self.active_windows.remove(&(hwnd.0 as isize));
        
        if let Some(ws) = self.workspaces.iter_mut().find(|w| w.id == self.active_workspace) {
            if let Some(mut root) = ws.root.take() {
                if root.remove(send_hwnd) {
                    crate::velowin_log!("[Tiling] Successfully pruned window from tree: {:?}", hwnd);
                } else {
                    // if remove returned false, it means root itself was the target and was pruned
                    // or the window wasn't in this workspace.
                    ws.root = Some(root);
                }
            }
        }

        self.recalculate_layout();
    }

    pub fn recalculate_layout(&self) {
        let config = KB.lock().unwrap().config.clone();
        let gaps_in = config.get_int("general", "gaps_in", 0);
        let gaps_out = config.get_int("general", "gaps_out", 0);

        if let Some(ws) = self.workspaces.iter().find(|w| w.id == self.active_workspace) {
            crate::velowin_log!("--- Recalculating Layout (Workspace: {}, Mode: {:?}) ---", ws.id, ws.layout);
            match ws.layout {
                LayoutType::Dwindle => {
                    if let Some(root) = &ws.root {
                        let mut results = Vec::new();
                        root.get_layout_results(&mut results, gaps_in, gaps_out);
                        crate::velowin_log!("[Tiling] Arranging {} windows", results.len());
                        self.apply_layout_results(results);
                    }
                }
                _ => {}
            }
        }
    }

    fn apply_layout_results(&self, results: Vec<(SendHWND, Rect)>) {
        for (send_hwnd, rect) in results {
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(send_hwnd.0, windows::Win32::UI::WindowsAndMessaging::SW_RESTORE);

                let mut style = windows::Win32::UI::WindowsAndMessaging::GetWindowLongW(send_hwnd.0, windows::Win32::UI::WindowsAndMessaging::GWL_STYLE) as u32;
                if (style & windows::Win32::UI::WindowsAndMessaging::WS_CAPTION.0) != 0 {
                    style &= !(windows::Win32::UI::WindowsAndMessaging::WS_CAPTION.0 
                             | windows::Win32::UI::WindowsAndMessaging::WS_THICKFRAME.0 
                             | windows::Win32::UI::WindowsAndMessaging::WS_MINIMIZEBOX.0 
                             | windows::Win32::UI::WindowsAndMessaging::WS_MAXIMIZEBOX.0 
                             | windows::Win32::UI::WindowsAndMessaging::WS_SYSMENU.0);
                    let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowLongW(send_hwnd.0, windows::Win32::UI::WindowsAndMessaging::GWL_STYLE, style as i32);
                    let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(send_hwnd.0, HWND(std::ptr::null_mut()), 0, 0, 0, 0, 
                        windows::Win32::UI::WindowsAndMessaging::SWP_NOMOVE | windows::Win32::UI::WindowsAndMessaging::SWP_NOSIZE | windows::Win32::UI::WindowsAndMessaging::SWP_FRAMECHANGED);
                }

                let mut window_rect = windows::Win32::Foundation::RECT::default();
                let mut frame_rect = windows::Win32::Foundation::RECT::default();
                let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowRect(send_hwnd.0, &mut window_rect);
                let _ = windows::Win32::Graphics::Dwm::DwmGetWindowAttribute(
                    send_hwnd.0, 
                    windows::Win32::Graphics::Dwm::DWMWA_EXTENDED_FRAME_BOUNDS, 
                    &mut frame_rect as *mut _ as *mut std::ffi::c_void, 
                    std::mem::size_of::<windows::Win32::Foundation::RECT>() as u32
                );

                let offset_x = frame_rect.left - window_rect.left;
                let offset_y = frame_rect.top - window_rect.top;
                let offset_w = (window_rect.right - window_rect.left) - (frame_rect.right - frame_rect.left);
                let offset_h = (window_rect.bottom - window_rect.top) - (frame_rect.bottom - frame_rect.top);

                let final_x = rect.x - offset_x;
                let final_y = rect.y - offset_y;
                let final_w = rect.width + offset_w;
                let final_h = rect.height + offset_h;

                if let Some(window) = self.active_windows.get(&(send_hwnd.0.0 as isize)) {
                    crate::velowin_log!("  -> Window: '{}', Rect: (x:{}, y:{}, w:{}, h:{})", 
                        window.title, rect.x, rect.y, rect.width, rect.height);
                    
                    crate::render::Renderer::CreateBorderWrapper(send_hwnd, 2, 10.0);
                    crate::render::Renderer::SetBorderAngleWrapper(send_hwnd, window.border_angle.value);
                    crate::render::Renderer::UpdateBorderPositionWrapper(send_hwnd, rect.x, rect.y, rect.width, rect.height);
                }

                let _ = SetWindowPos(
                    send_hwnd.0,
                    HWND(std::ptr::null_mut()),
                    final_x,
                    final_y,
                    final_w,
                    final_h,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
        crate::velowin_log!("--- Layout Applied ---");
    }
}
