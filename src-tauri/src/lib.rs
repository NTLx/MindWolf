mod error;
mod types;
mod config;
mod llm;
mod commands;
mod utils;
mod game_engine;
mod game_manager;
mod ai;
mod database;
mod voice;
mod replay;

use commands::*;
use std::sync::Arc;
use log::info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    
    // 只在主函数中未初始化日志时才初始化
    if env_logger::try_init().is_ok() {
        info!("智狼 (MindWolf) 启动中...");
    }
    
    // 创建应用状态
    let app_state = match commands::AppState::new() {
        Ok(state) => {
            info!("应用状态初始化成功");
            state
        },
        Err(e) => {
            let error_msg = format!("初始化应用状态失败: {}", e);
            eprintln!("{}", error_msg);
            
            // 在 Windows 上显示消息框
            #[cfg(windows)]
            {
                use std::ffi::CString;
                use std::ptr;
                
                unsafe {
                    let title = CString::new("智狼 (MindWolf) - 错误").unwrap_or_default();
                    let message = CString::new(error_msg).unwrap_or_default();
                    
                    winapi::um::winuser::MessageBoxA(
                        ptr::null_mut(),
                        message.as_ptr(),
                        title.as_ptr(),
                        winapi::um::winuser::MB_OK | winapi::um::winuser::MB_ICONERROR,
                    );
                }
            }
            
            return;
        }
    };
    
    // 启动 Tauri 应用
    match tauri::Builder::default()
        .plugin(tauri_plugin_opener::init::<tauri::Wry>())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            update_llm_config,
            test_llm_connection,
            generate_ai_response,
            update_game_config,
            start_new_game,
            launch_game,
            get_game_state,
            player_vote,
            player_speech,
            generate_ai_speech,
            end_game,
            export_config,
            import_config,
            get_app_version
        ])
        .run(tauri::generate_context!()) {
        Ok(_) => {
            info!("应用正常退出");
        },
        Err(e) => {
            let error_msg = format!("启动 Tauri 应用失败: {}", e);
            eprintln!("{}", error_msg);
            
            // 在 Windows 上显示消息框
            #[cfg(windows)]
            {
                use std::ffi::CString;
                use std::ptr;
                
                unsafe {
                    let title = CString::new("智狼 (MindWolf) - 错误").unwrap_or_default();
                    let message = CString::new(error_msg).unwrap_or_default();
                    
                    winapi::um::winuser::MessageBoxA(
                        ptr::null_mut(),
                        message.as_ptr(),
                        title.as_ptr(),
                        winapi::um::winuser::MB_OK | winapi::um::winuser::MB_ICONERROR,
                    );
                }
            }
        }
    }
}
