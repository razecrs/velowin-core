use windows::Win32::Foundation::HWND;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendHWND(pub HWND);

impl std::hash::Hash for SendHWND {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.0.hash(state);
    }
}

unsafe impl Send for SendHWND {}
unsafe impl Sync for SendHWND {}
