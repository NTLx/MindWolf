use crate::config::ConfigManager;
use crate::error::{AppError, AppResult};
use crate::llm::{LLMManager, LLMClient};
use crate::game_manager::GameManager;
use crate::types::{LLMConfig, GameConfig, GameState};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error};

/// 应用状态
pub struct AppState {
    pub config_manager: Arc<RwLock<ConfigManager>>,
    pub llm_manager: Arc<RwLock<Option<LLMManager>>>,
    pub game_manager: Arc<RwLock<GameManager>>,
}

impl AppState {
    pub fn new() -> AppResult<Self> {
        let config_manager = ConfigManager::new()?;
        
        Ok(Self {
            config_manager: Arc::new(RwLock::new(config_manager)),
            llm_manager: Arc::new(RwLock::new(None)),
            game_manager: Arc::new(RwLock::new(GameManager::new())),
        })
    }
}

/// 获取应用配置
#[tauri::command]
pub async fn get_app_config(
    state: tauri::State<'_, AppState>
) -> Result<crate::config::AppConfig, String> {
    let config_manager = state.config_manager.read().await;
    Ok(config_manager.get_config().clone())
}

/// 更新LLM配置
#[tauri::command]
pub async fn update_llm_config(
    state: tauri::State<'_, AppState>,
    config: LLMConfig
) -> Result<(), String> {
    let mut config_manager = state.config_manager.write().await;
    
    config_manager.update_llm_config(config.clone()).await
        .map_err(|e| e.to_string())?;
    
    // 重新创建LLM管理器
    let llm_manager = Arc::new(LLMManager::new(config, vec![]));
    let mut llm_state = state.llm_manager.write().await;
    *llm_state = Some(llm_manager.as_ref().clone());
    
    // 更新游戏管理器的LLM管理器
    let mut game_manager = state.game_manager.write().await;
    game_manager.set_llm_manager(llm_manager);
    
    info!("LLM配置已更新");
    Ok(())
}

/// 测试LLM连接
#[tauri::command]
pub async fn test_llm_connection(
    state: tauri::State<'_, AppState>
) -> Result<bool, String> {
    let llm_manager_guard = state.llm_manager.read().await;
    
    if let Some(llm_manager) = llm_manager_guard.as_ref() {
        let results = llm_manager.test_all_connections().await
            .map_err(|e| e.to_string())?;
        
        // 如果至少有一个连接成功，返回true
        Ok(results.iter().any(|&success| success))
    } else {
        Err("LLM管理器未初始化".to_string())
    }
}

/// 生成AI响应
#[tauri::command]
pub async fn generate_ai_response(
    state: tauri::State<'_, AppState>,
    prompt: String
) -> Result<String, String> {
    let llm_manager_guard = state.llm_manager.read().await;
    
    if let Some(llm_manager) = llm_manager_guard.as_ref() {
        llm_manager.generate_with_fallback(prompt).await
            .map_err(|e| e.to_string())
    } else {
        Err("LLM管理器未初始化".to_string())
    }
}

/// 更新游戏配置
#[tauri::command]
pub async fn update_game_config(
    state: tauri::State<'_, AppState>,
    config: GameConfig
) -> Result<(), String> {
    let mut config_manager = state.config_manager.write().await;
    
    config_manager.update_game_config(config).await
        .map_err(|e| e.to_string())?;
    
    info!("游戏配置已更新");
    Ok(())
}

/// 开始新游戏
#[tauri::command]
pub async fn start_new_game(
    state: tauri::State<'_, AppState>,
    config: GameConfig
) -> Result<GameState, String> {
    info!("开始新游戏: {:?}", config);
    
    let mut game_manager = state.game_manager.write().await;
    let game_state = game_manager.create_game(config).await
        .map_err(|e| e.to_string())?;
    
    Ok(game_state)
}

/// 启动游戏
#[tauri::command]
pub async fn launch_game(
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let mut game_manager = state.game_manager.write().await;
    game_manager.start_game().await
        .map_err(|e| e.to_string())
}

/// 获取当前游戏状态
#[tauri::command]
pub async fn get_game_state(
    state: tauri::State<'_, AppState>
) -> Result<Option<GameState>, String> {
    let game_manager = state.game_manager.read().await;
    Ok(game_manager.get_game_state())
}

/// 玩家投票
#[tauri::command]
pub async fn player_vote(
    state: tauri::State<'_, AppState>,
    voter_id: String,
    target_id: String
) -> Result<(), String> {
    let mut game_manager = state.game_manager.write().await;
    game_manager.player_vote(voter_id, target_id).await
        .map_err(|e| e.to_string())
}

/// 玩家发言
#[tauri::command]
pub async fn player_speech(
    state: tauri::State<'_, AppState>,
    player_id: String,
    content: String
) -> Result<(), String> {
    let mut game_manager = state.game_manager.write().await;
    game_manager.handle_player_speech(player_id, content).await
        .map_err(|e| e.to_string())
}

/// 生成AI发言
#[tauri::command]
pub async fn generate_ai_speech(
    state: tauri::State<'_, AppState>,
    player_id: String
) -> Result<String, String> {
    let mut game_manager = state.game_manager.write().await;
    game_manager.generate_ai_speech(player_id).await
        .map_err(|e| e.to_string())
}

/// 结束游戏
#[tauri::command]
pub async fn end_game(
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let mut game_manager = state.game_manager.write().await;
    game_manager.end_game().await
        .map_err(|e| e.to_string())
}

/// 导出配置
#[tauri::command]
pub async fn export_config(
    state: tauri::State<'_, AppState>
) -> Result<String, String> {
    let config_manager = state.config_manager.read().await;
    config_manager.export_config().map_err(|e| e.to_string())
}

/// 导入配置
#[tauri::command]
pub async fn import_config(
    state: tauri::State<'_, AppState>,
    config_json: String
) -> Result<(), String> {
    let mut config_manager = state.config_manager.write().await;
    config_manager.import_config(&config_json).await
        .map_err(|e| e.to_string())
}

/// 获取应用版本
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}