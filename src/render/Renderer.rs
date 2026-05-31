use crate::helpers::Types::SendHWND;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::*;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[link(name = "velowin_renderer", kind = "static")]
unsafe extern "C" {
    pub fn InitCompositor(overlayHwnd: HWND) -> bool;
    pub fn CreateBorder(targetHwnd: HWND, borderSize: i32, rounding: f32);
    pub fn SetBorderColors(targetHwnd: HWND, colors: *const Color, count: i32);
    pub fn SetBorderAngle(targetHwnd: HWND, angle: f32);
    pub fn UpdateBorderPosition(targetHwnd: HWND, x: i32, y: i32, width: i32, height: i32);
}

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

pub fn create_overlay_window() -> Result<HWND> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(overlay_wnd_proc),
            hInstance: instance.into(),
            lpszClassName: w!("VelowinOverlay"),
            style: CS_HREDRAW | CS_VREDRAW,
            hbrBackground: HBRUSH(GetStockObject(HOLLOW_BRUSH).0),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            return Err(Error::from_win32());
        }

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            w!("VelowinOverlay"),
            w!("Velowin Overlay"),
            WS_POPUP,
            0, 0, GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN),
            None,
            None,
            instance,
            None,
        )?;

        SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)?;
        let _ = ShowWindow(hwnd, SW_SHOW);

        Ok(hwnd)
    }
}

unsafe extern "system" fn overlay_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
