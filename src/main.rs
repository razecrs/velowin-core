#![allow(non_snake_case)]

pub mod config;
pub mod debug;
pub mod desktop;
pub mod devices;
pub mod errorOverlay;
pub mod event;
pub mod helpers;
pub mod i18n;
pub mod init;
pub mod layout;
pub mod managers;
pub mod notification;
pub mod pch;
pub mod plugins;
pub mod protocols;
pub mod render;
pub mod xwayland;

pub mod Compositor;
pub mod defines;
pub mod includes;
pub mod macros;
pub mod SharedDefs;
pub mod version;

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
};

fn main() -> Result<()> {
    unsafe {
        println!("Velowin: Starting 1:1 Hyprland-like WM...");

        ctrlc::set_handler(move || {
            crate::helpers::Logger::cleanup();
            std::process::exit(0);
        }).expect("Error setting Ctrl-C handler");

        Compositor::init()?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }
    }

    Ok(())
}
