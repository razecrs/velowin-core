use windows::Win32::Foundation::HWND;

// Link to the C++ library compiled by build.rs
#[link(name = "velowin_renderer", kind = "static")]
unsafe extern "C" {
    pub fn InitCompositor(overlayHwnd: HWND) -> bool;
    pub fn CreateBorder(targetHwnd: HWND, borderSize: i32, rounding: f32);
    pub fn UpdateBorderPosition(targetHwnd: HWND, x: i32, y: i32, width: i32, height: i32);
}
