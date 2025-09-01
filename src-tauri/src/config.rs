use crate::error::{AppError, AppResult};
use crate::types::{LLMConfig, GameConfig, LLMProvider};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use log::{info, warn};

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LLMConfig,
    pub game: GameConfig,
    pub voice: VoiceConfig,
    pub app: GeneralConfig,
}

/// 语音配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enable_asr: bool,
    pub enable_tts: bool,
    pub speech_rate: f32,
    pub volume: u8,
}

/// 通用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub auto_save_replay: bool,
    pub show_ai_thinking: bool,
    pub theme: String,
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LLMConfig {
                provider: LLMProvider::OpenAI,
                api_key: String::new(),
                base_url: "https://api.openai.com".to_string(),
                model: "gpt-4o-realtime-preview-2024-12-17".to_string(),
                max_tokens: 2000,
                temperature: 0.7,
                timeout: 60,
                use_realtime_api: false, // 默认使用传统API
                voice: Some("alloy".to_string()),
                input_audio_format: Some("pcm16".to_string()),
                output_audio_format: Some("pcm16".to_string()),
                modalities: vec!["text".to_string(), "audio".to_string()],
                instructions: Some("你是一个狼人杀游戏AI助手".to_string()),
                turn_detection: Some(crate::types::TurnDetectionConfig {
                    detection_type: "server_vad".to_string(),
                    threshold: Some(0.5),
                    prefix_padding_ms: Some(300),
                    silence_duration_ms: Some(200),
                }),
            },
            game: GameConfig {
                total_players: 8,
                role_distribution: std::collections::HashMap::new(),
                discussion_time: 300,
                voting_time: 60,
                enable_voice: false,
            },
            voice: VoiceConfig {
                enable_asr: false,
                enable_tts: true,
                speech_rate: 1.0,
                volume: 80,
            },
            app: GeneralConfig {
                auto_save_replay: true,
                show_ai_thinking: true,
                theme: "auto".to_string(),
                language: "zh-CN".to_string(),
            },
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> AppResult<Self> {
        let config_path = Self::get_config_path()?;
        let config = Self::load_or_create_config(&config_path)?;
        
        Ok(Self {
            config_path,
            config,
        })
    }
    
    /// 获取配置文件路径
    fn get_config_path() -> AppResult<PathBuf> {
        // 尝试便携式模式：优先使用可执行文件目录
        let portable_path = Self::get_portable_config_path();
        if let Ok(path) = portable_path {
            return Ok(path);
        }
        
        // 回退到系统配置目录
        let mut path = dirs::config_dir()
            .ok_or_else(|| AppError::Config("无法获取配置目录".to_string()))?;
        
        path.push("MindWolf");
        
        // 确保目录存在
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .map_err(|e| AppError::Config(format!("创建配置目录失败: {}", e)))?;
        }
        
        path.push("config.json");
        Ok(path)
    }
    
    /// 获取便携式配置路径
    fn get_portable_config_path() -> AppResult<PathBuf> {
        let exe_path = std::env::current_exe()
            .map_err(|e| AppError::Config(format!("无法获取可执行文件路径: {}", e)))?;
        
        let exe_dir = exe_path.parent()
            .ok_or_else(|| AppError::Config("无法获取可执行文件目录".to_string()))?;
        
        let config_dir = exe_dir.join("config");
        
        // 确保配置目录存在
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .map_err(|e| AppError::Config(format!("创建便携式配置目录失败: {}", e)))?;
        }
        
        Ok(config_dir.join("config.json"))
    }
    
    /// 加载或创建配置
    fn load_or_create_config(config_path: &PathBuf) -> AppResult<AppConfig> {
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)
                .map_err(|e| AppError::Config(format!("读取配置文件失败: {}", e)))?;
            
            let config: AppConfig = serde_json::from_str(&content)
                .map_err(|e| AppError::Config(format!("解析配置文件失败: {}", e)))?;
            
            info!("已加载配置文件: {:?}", config_path);
            Ok(config)
        } else {
            let config = AppConfig::default();
            
            let content = serde_json::to_string_pretty(&config)
                .map_err(|e| AppError::Config(format!("序列化默认配置失败: {}", e)))?;
            
            std::fs::write(config_path, content)
                .map_err(|e| AppError::Config(format!("写入默认配置失败: {}", e)))?;
            
            info!("已创建默认配置文件: {:?}", config_path);
            Ok(config)
        }
    }
    
    /// 获取当前配置
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
    
    /// 更新LLM配置
    pub async fn update_llm_config(&mut self, llm_config: LLMConfig) -> AppResult<()> {
        self.config.llm = llm_config;
        self.save_config().await
    }
    
    /// 更新游戏配置
    pub async fn update_game_config(&mut self, game_config: GameConfig) -> AppResult<()> {
        self.config.game = game_config;
        self.save_config().await
    }
    
    /// 更新语音配置
    pub async fn update_voice_config(&mut self, voice_config: VoiceConfig) -> AppResult<()> {
        self.config.voice = voice_config;
        self.save_config().await
    }
    
    /// 保存配置
    async fn save_config(&self) -> AppResult<()> {
        let content = serde_json::to_string_pretty(&self.config)
            .map_err(|e| AppError::Config(format!("序列化配置失败: {}", e)))?;
        
        fs::write(&self.config_path, content).await
            .map_err(|e| AppError::Config(format!("保存配置失败: {}", e)))?;
        
        info!("配置已保存: {:?}", self.config_path);
        Ok(())
    }
    
    /// 重置为默认配置
    pub async fn reset_to_default(&mut self) -> AppResult<()> {
        self.config = AppConfig::default();
        self.save_config().await
    }
    
    /// 导出配置
    pub fn export_config(&self) -> AppResult<String> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| AppError::Config(format!("导出配置失败: {}", e)))
    }
    
    /// 导入配置
    pub async fn import_config(&mut self, config_json: &str) -> AppResult<()> {
        let config: AppConfig = serde_json::from_str(config_json)
            .map_err(|e| AppError::Config(format!("解析导入配置失败: {}", e)))?;
        
        self.config = config;
        self.save_config().await
    }
}
