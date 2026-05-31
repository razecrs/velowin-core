use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::{MONITORINFOF_PRIMARY, SystemParametersInfoW, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS};

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

impl Default for Monitor {
    fn default() -> Self {
        Self {
            hmonitor: SendHMONITOR(HMONITOR(std::ptr::null_mut())),
            rect: RECT::default(),
            work_rect: RECT { left: 0, top: 0, right: 1920, bottom: 1080 },
            dpi: 96,
            is_primary: true,
        }
    }
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
        crate::velowin_log!("--- Refreshing Monitor List ---");
        unsafe {
            let _ = EnumDisplayMonitors(HDC(std::ptr::null_mut()), None, Some(Self::enum_monitor_callback), LPARAM(self as *mut Self as isize));
        }
        crate::velowin_log!("--- Total Monitors Found: {} ---", self.monitors.len());
    }

    unsafe extern "system" fn enum_monitor_callback(hmonitor: HMONITOR, _: HDC, rect: *mut RECT, lparam: LPARAM) -> BOOL {
        let manager = unsafe { &mut *(lparam.0 as *mut Self) };
        
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        
        if unsafe { GetMonitorInfoW(hmonitor, &mut info.monitorInfo as *mut _ as *mut MONITORINFO) }.as_bool() {
            let mut dpi_x = 0;
            let mut dpi_y = 0;
            let _ = unsafe { GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };
            
            let mut work_rect = info.monitorInfo.rcWork;

            // DOC-DRIVEN: Secondary check for WorkArea using SPI_GETWORKAREA for primary monitor
            if (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0 {
                let mut spi_rect = RECT::default();
                let _ = unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, Some(&mut spi_rect as *mut _ as *mut std::ffi::c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)) };
                if spi_rect.bottom != 0 {
                    work_rect = spi_rect;
                }
            }
            
            let m = Monitor {
                hmonitor: SendHMONITOR(hmonitor),
                rect: unsafe { *rect },
                work_rect,
                dpi: dpi_x,
                is_primary: (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0,
            };

            crate::velowin_log!("[Monitor] Primary: {}, DPI: {}, WorkArea: ({}, {}) to ({}, {})", 
                m.is_primary, m.dpi, m.work_rect.left, m.work_rect.top, m.work_rect.right, m.work_rect.bottom);

            manager.monitors.push(m);
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
