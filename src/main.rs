#![allow(non_snake_case)]

pub mod config;
pub mod desktop;
pub mod helpers;
pub mod layout;
pub mod managers;
pub mod render;
pub mod Compositor;

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
};

fn main() -> Result<()> {
    unsafe {
        println!("Velowin: Starting 1:1 Hyprland-like WM...");

        Compositor::init()?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }
    }

    Ok(())
}
