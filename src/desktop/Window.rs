use crate::helpers::Types::SendHWND;

#[derive(Debug, Clone)]
pub struct WindowState {
    pub hwnd: SendHWND,
    pub title: String,
    pub class_name: String,
    pub is_floating: bool,
    pub workspace_id: u32,
}
