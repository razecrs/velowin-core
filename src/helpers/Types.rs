use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::HHOOK;
use windows::Win32::UI::Accessibility::HWINEVENTHOOK;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendHWND(pub HWND);

impl std::hash::Hash for SendHWND {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.0.hash(state);
    }
}

unsafe impl Send for SendHWND {}
unsafe impl Sync for SendHWND {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendHHOOK(pub HHOOK);
unsafe impl Send for SendHHOOK {}
unsafe impl Sync for SendHHOOK {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendHWINEVENTHOOK(pub HWINEVENTHOOK);
unsafe impl Send for SendHWINEVENTHOOK {}
unsafe impl Sync for SendHWINEVENTHOOK {}
