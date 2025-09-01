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
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("智狼 (MindWolf) 启动中...");
    
    // 创建应用状态
    let app_state = match commands::AppState::new() {
        Ok(state) => state,
        Err(e) => {
            eprintln!("初始化应用状态失败: {}", e);
            return;
        }
    };
    
    tauri::Builder::default()
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
