#![windows_subsystem = "windows"]
#[cfg(target_os="windows")]
mod win32;
#[cfg(target_os="windows")]
include!("win32.rs");
