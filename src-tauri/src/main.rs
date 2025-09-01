// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod windows;

fn main() {
    // 初始化日志到文件和控制台
    #[cfg(debug_assertions)]
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    #[cfg(not(debug_assertions))]
    {
        // 在发布模式下，尝试写入到可执行文件目录的日志文件
        let exe_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let log_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new(".")).join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }
    
    // Windows便携式配置
    #[cfg(windows)]
    {
        if let Err(e) = windows::configure_portable() {
            eprintln!("Failed to configure portable mode: {}", e);
        }
    }
    
    mindwolf_lib::run()
}
