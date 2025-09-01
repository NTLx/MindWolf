// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod windows;

fn main() {
    // Windows便携式配置
    #[cfg(windows)]
    {
        if let Err(e) = windows::configure_portable() {
            eprintln!("Failed to configure portable mode: {}", e);
        }
    }
    
    mindwolf_lib::run()
}
