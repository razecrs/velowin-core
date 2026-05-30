use crate::helpers::Types::SendHWND;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

// Link to the C++ library compiled by build.rs
#[link(name = "velowin_renderer", kind = "static")]
unsafe extern "C" {
    pub fn InitCompositor(overlayHwnd: windows::Win32::Foundation::HWND) -> bool;
    pub fn CreateBorder(targetHwnd: windows::Win32::Foundation::HWND, borderSize: i32, rounding: f32);
    pub fn SetBorderColors(targetHwnd: windows::Win32::Foundation::HWND, colors: *const Color, count: i32);
    pub fn SetBorderAngle(targetHwnd: windows::Win32::Foundation::HWND, angle: f32);
    pub fn UpdateBorderPosition(targetHwnd: windows::Win32::Foundation::HWND, x: i32, y: i32, width: i32, height: i32);
}

// Wrapper for SendHWND
pub fn CreateBorderWrapper(hwnd: SendHWND, borderSize: i32, rounding: f32) {
    unsafe { CreateBorder(hwnd.0, borderSize, rounding) };
}

pub fn SetBorderColorsWrapper(hwnd: SendHWND, colors: &[Color]) {
    unsafe { SetBorderColors(hwnd.0, colors.as_ptr(), colors.len() as i32) };
}

pub fn SetBorderAngleWrapper(hwnd: SendHWND, angle: f32) {
    unsafe { SetBorderAngle(hwnd.0, angle) };
}

pub fn UpdateBorderPositionWrapper(hwnd: SendHWND, x: i32, y: i32, width: i32, height: i32) {
    unsafe { UpdateBorderPosition(hwnd.0, x, y, width, height) };
}
