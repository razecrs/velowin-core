use crate::Compositor::WM;
use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOZORDER, SWP_NOACTIVATE};
use windows::Win32::Foundation::HWND;

pub struct AnimationManager {
    pub enabled: bool,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn tick(&self) {
        if !self.enabled { return; }

        let mut wm = WM.lock().unwrap();
        
        for window in wm.active_windows.values_mut() {
            // Update animations
            let x_anim = window.x.update();
            let y_anim = window.y.update();
            let w_anim = window.width.update();
            let h_anim = window.height.update();
            let angle_anim = window.border_angle.update();
            let opacity_anim = window.opacity.update();
            
            // Only move the window if the coordinates actually changed
            if x_anim || y_anim || w_anim || h_anim {
                unsafe {
                    let _ = SetWindowPos(
                        window.hwnd.0,
                        HWND(std::ptr::null_mut()),
                        window.x.value,
                        window.y.value,
                        window.width.value,
                        window.height.value,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
            }

            // Always update the GPU visuals if anything changed
            if x_anim || y_anim || w_anim || h_anim || angle_anim || opacity_anim {
                crate::render::Renderer::SetBorderAngleWrapper(window.hwnd, window.border_angle.value);
                // We'll update border position separately or via a wrapper
                crate::render::Renderer::UpdateBorderPositionWrapper(window.hwnd, window.x.value, window.y.value, window.width.value, window.height.value);
            }
        }
    }
}
