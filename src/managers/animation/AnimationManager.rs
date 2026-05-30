use crate::Compositor::WM;
use std::time::Duration;

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
        let mut needs_redraw = false;

        for window in wm.active_windows.values_mut() {
            let angle_anim = window.border_angle.update();
            let opacity_anim = window.opacity.update();
            
            if angle_anim || opacity_anim {
                needs_redraw = true;
                // Update the GPU with new values
                crate::render::Renderer::SetBorderAngleWrapper(window.hwnd, window.border_angle.value);
            }
        }

        if needs_redraw {
            wm.recalculate_layout();
        }
    }
}
