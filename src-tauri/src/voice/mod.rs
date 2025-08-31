pub mod asr;
pub mod tts;
pub mod audio;

pub use asr::*;
pub use tts::*;
pub use audio::*;

use crate::error::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

/// 语音配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enable_asr: bool,
    pub enable_tts: bool,
    pub language: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub chunk_duration_ms: u32,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enable_asr: false,
            enable_tts: true,
            language: "zh-CN".to_string(),
            sample_rate: 16000,
            channels: 1,
            chunk_duration_ms: 1000,
        }
    }
}

/// 语音管理器
pub struct VoiceManager {
    config: VoiceConfig,
    asr_engine: Arc<Mutex<ASREngine>>,
    tts_engine: Arc<Mutex<TTSEngine>>,
    audio_manager: Arc<AudioManager>,
    is_enabled: Arc<Mutex<bool>>,
}

impl VoiceManager {
    /// 创建语音管理器
    pub fn new(config: VoiceConfig) -> Self {
        Self {
            config,
            asr_engine: Arc::new(Mutex::new(ASREngine::new())),
            tts_engine: Arc::new(Mutex::new(TTSEngine::new())),
            audio_manager: Arc::new(AudioManager::new()),
            is_enabled: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 初始化语音系统
    pub async fn initialize(&self) -> Result<()> {
        log::info("正在初始化语音系统...");
        
        // 初始化音频管理器
        self.audio_manager.initialize().await?;
        
        // 初始化ASR引擎
        if self.config.enable_asr {
            self.asr_engine.lock().await.initialize().await?;
        }
        
        // 初始化TTS引擎
        if self.config.enable_tts {
            self.tts_engine.lock().await.initialize().await?;
        }
        
        // 启用语音功能
        *self.is_enabled.lock().await = true;
        
        log::info("语音系统初始化完成");
        Ok(())
    }
    
    /// 开始录音
    pub async fn start_recording(&self) -> Result<()> {
        if !self.config.enable_asr {
            return Err(crate::error::AppError::Config("语音识别未启用".to_string()).into());
        }
        
        self.audio_manager.start_recording().await
    }
    
    /// 停止录音并识别
    pub async fn stop_recording_and_recognize(&self) -> Result<String> {
        if !self.config.enable_asr {
            return Err(crate::error::AppError::Config("语音识别未启用".to_string()).into());
        }
        
        let audio_data = self.audio_manager.stop_recording().await?;
        self.asr_engine.lock().await.recognize(&audio_data).await
    }
    
    /// 文本转语音
    pub async fn text_to_speech(&self, text: &str) -> Result<Vec<u8>> {
        if !self.config.enable_tts {
            return Err(crate::error::AppError::Config("语音合成未启用".to_string()).into());
        }
        
        self.tts_engine.lock().await.synthesize(text).await
    }
    
    /// 播放语音
    pub async fn play_audio(&self, audio_data: &[u8]) -> Result<()> {
        self.audio_manager.play_audio(audio_data.to_vec()).await
    }
    
    /// 更新配置
    pub fn update_config(&mut self, config: VoiceConfig) {
        self.config = config;
    }
    
    /// 检查语音功能可用性
    pub async fn check_availability(&self) -> VoiceAvailability {
        VoiceAvailability {
            asr_available: self.config.enable_asr,
            tts_available: self.config.enable_tts,
            audio_input_available: !self.audio_manager.get_devices().await.unwrap_or_default().is_empty(),
            audio_output_available: true,
        }
    }
    
    /// 获取语音设置
    pub async fn get_audio_settings(&self) -> Result<AudioSettings> {
        self.audio_manager.get_settings().await
    }
    
    /// 设置语音参数
    pub async fn set_audio_settings(&self, settings: AudioSettings) -> Result<()> {
        self.audio_manager.set_settings(settings).await
    }
    
    /// 获取音频设备列表
    pub async fn get_audio_devices(&self) -> Result<Vec<AudioDevice>> {
        self.audio_manager.get_devices().await
    }
    
    /// 设置输出音量
    pub async fn set_output_volume(&self, volume: f32) -> Result<()> {
        self.audio_manager.set_output_volume(volume).await
    }
    
    /// 获取输出音量
    pub async fn get_output_volume(&self) -> Result<f32> {
        self.audio_manager.get_output_volume().await
    }
    
    /// 检查是否正在录音
    pub async fn is_recording(&self) -> bool {
        self.audio_manager.is_recording().await
    }
    
    /// 检查是否正在播放
    pub async fn is_playing(&self) -> bool {
        self.audio_manager.is_playing().await
    }
    
    /// 关闭语音管理器
    pub async fn shutdown(&self) -> Result<()> {
        *self.is_enabled.lock().await = false;
        self.audio_manager.shutdown().await?;
        log::info("语音管理器已关闭");
        Ok(())
    }
}

/// 语音功能可用性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceAvailability {
    pub asr_available: bool,
    pub tts_available: bool,
    pub audio_input_available: bool,
    pub audio_output_available: bool,
}