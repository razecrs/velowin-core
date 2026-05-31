use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::MONITORINFOF_PRIMARY;
use windows::Win32::UI::HiDpi::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendHMONITOR(pub HMONITOR);
unsafe impl Send for SendHMONITOR {}
unsafe impl Sync for SendHMONITOR {}

#[derive(Debug, Clone)]
pub struct Monitor {
    pub hmonitor: SendHMONITOR,
    pub rect: RECT,
    pub work_rect: RECT,
    pub dpi: u32,
    pub is_primary: bool,
}

pub struct MonitorManager {
    pub monitors: Vec<Monitor>,
}

impl MonitorManager {
    pub fn new() -> Self {
        let mut manager = Self { monitors: Vec::new() };
        manager.refresh();
        manager
    }

    pub fn refresh(&mut self) {
        self.monitors.clear();
        unsafe {
            let _ = EnumDisplayMonitors(HDC(std::ptr::null_mut()), None, Some(Self::enum_monitor_callback), LPARAM(self as *mut Self as isize));
        }
    }

    unsafe extern "system" fn enum_monitor_callback(hmonitor: HMONITOR, _: HDC, rect: *mut RECT, lparam: LPARAM) -> BOOL {
        let manager = unsafe { &mut *(lparam.0 as *mut Self) };
        
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        
        if unsafe { GetMonitorInfoW(hmonitor, &mut info.monitorInfo as *mut _ as *mut MONITORINFO) }.as_bool() {
            let mut dpi_x = 0;
            let mut dpi_y = 0;
            
            let _ = unsafe { GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };
            
            manager.monitors.push(Monitor {
                hmonitor: SendHMONITOR(hmonitor),
                rect: unsafe { *rect },
                work_rect: info.monitorInfo.rcWork,
                dpi: dpi_x,
                is_primary: (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0,
            });
        }
        
        true.into()
    }

    pub fn get_monitor_for_window(&self, hwnd: HWND) -> Option<&Monitor> {
        unsafe {
            let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            self.monitors.iter().find(|m| m.hmonitor.0 == hmonitor)
        }
    }
}
