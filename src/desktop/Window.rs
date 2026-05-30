use crate::helpers::Types::SendHWND;
use crate::helpers::AnimatedVariable::AnimatedVariable;

pub struct WindowState {
    pub hwnd: SendHWND,
    pub title: String,
    pub class_name: String,
    pub is_floating: bool,
    pub workspace_id: u32,
    
    // Animations
    pub border_angle: AnimatedVariable<f32>,
    pub opacity: AnimatedVariable<f32>,
}

impl WindowState {
    pub fn new(hwnd: SendHWND, title: String, class_name: String, workspace_id: u32) -> Self {
        Self {
            hwnd,
            title,
            class_name,
            is_floating: false,
            workspace_id,
            border_angle: AnimatedVariable::new(0.0),
            opacity: AnimatedVariable::new(1.0),
        }
    }
}
