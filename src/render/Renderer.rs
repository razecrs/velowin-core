use crate::helpers::Types::SendHWND;

// Link to the C++ library compiled by build.rs
#[link(name = "velowin_renderer", kind = "static")]
unsafe extern "C" {
    pub fn InitCompositor(overlayHwnd: windows::Win32::Foundation::HWND) -> bool;
    pub fn CreateBorder(targetHwnd: windows::Win32::Foundation::HWND, borderSize: i32, rounding: f32);
    pub fn UpdateBorderPosition(targetHwnd: windows::Win32::Foundation::HWND, x: i32, y: i32, width: i32, height: i32);
}

// Wrapper for SendHWND
pub fn CreateBorderWrapper(hwnd: SendHWND, borderSize: i32, rounding: f32) {
    unsafe { CreateBorder(hwnd.0, borderSize, rounding) };
}

pub fn UpdateBorderPositionWrapper(hwnd: SendHWND, x: i32, y: i32, width: i32, height: i32) {
    unsafe { UpdateBorderPosition(hwnd.0, x, y, width, height) };
}
