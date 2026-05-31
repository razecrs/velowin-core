use crate::helpers::Types::SendHWND;
use crate::helpers::AnimatedVariable::AnimatedVariable;

pub struct WindowState {
    pub hwnd: SendHWND,
    pub title: String,
    pub class_name: String,
    pub is_floating: bool,
    pub workspace_id: u32,
    
    // Smooth Animations (1:1 Hyprland Style)
    pub x: AnimatedVariable<i32>,
    pub y: AnimatedVariable<i32>,
    pub width: AnimatedVariable<i32>,
    pub height: AnimatedVariable<i32>,
    
    pub border_angle: AnimatedVariable<f32>,
    pub opacity: AnimatedVariable<f32>,
}

impl WindowState {
    pub fn new(hwnd: SendHWND, title: String, class_name: String, workspace_id: u32) -> Self {
        let mut rect = windows::Win32::Foundation::RECT::default();
        unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd.0, &mut rect); }

        Self {
            hwnd,
            title,
            class_name,
            is_floating: true, 
            workspace_id,
            x: AnimatedVariable::new_i32(rect.left),
            y: AnimatedVariable::new_i32(rect.top),
            width: AnimatedVariable::new_i32(rect.right - rect.left),
            height: AnimatedVariable::new_i32(rect.bottom - rect.top),
            border_angle: AnimatedVariable::new_f32(0.0),
            opacity: AnimatedVariable::new_f32(1.0),
        }
    }
}
