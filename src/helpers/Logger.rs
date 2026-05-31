use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static LOG_FILE: Lazy<Mutex<Option<std::fs::File>>> = Lazy::new(|| Mutex::new(None));

pub fn init() {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("velowin.log")
        .expect("Failed to create log file");
    
    let mut log_lock = LOG_FILE.lock().unwrap();
    *log_lock = Some(file);
    
    drop(log_lock);
    log("Velowin Log Initialized");
}

pub fn log(msg: &str) {
    let mut log_lock = LOG_FILE.lock().unwrap();
    if let Some(file) = log_lock.as_mut() {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!("[{}] {}\n", timestamp, msg);
        print!("{}", line); 
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}

pub fn cleanup() {
    log("Shutting down Velowin, restoring all windows...");
    let mut wm = crate::Compositor::WM.lock().unwrap();
    let hwnds: Vec<windows::Win32::Foundation::HWND> = wm.active_windows.values().map(|w| w.hwnd.0).collect();
    for hwnd in hwnds {
        wm.remove_window(hwnd);
    }
    log("Restoration complete.");
}

#[macro_export]
macro_rules! velowin_log {
    ($($arg:tt)*) => {
        $crate::helpers::Logger::log(&format!($($arg)*))
    };
}
