use crate::error::{AppError, AppResult};
use crate::voice::VoiceConfig;
use std::process::Command;
use tokio::fs;
use log::{info, debug};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use chrono::Utc;

/// 语音合成引擎
pub struct TTSEngine {
    config: VoiceConfig,
    voice_config: TTSVoiceConfig,
}

/// TTS语音配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSVoiceConfig {
    pub voice_name: String,
    pub speed: f32,
    pub pitch: f32,
    pub volume: f32,
    pub use_edge_tts: bool,
}

impl Default for TTSVoiceConfig {
    fn default() -> Self {
        Self {
            voice_name: "zh-CN-XiaoxiaoNeural".to_string(),
            speed: 1.0,
            pitch: 1.0,
            volume: 0.8,
            use_edge_tts: true,
        }
    }
}

impl TTSEngine {
    /// 创建TTS引擎
    pub fn new(config: &VoiceConfig) -> AppResult<Self> {
        Ok(Self {
            config: config.clone(),
            voice_config: TTSVoiceConfig::default(),
        })
    }
    
    /// 初始化TTS引擎
    pub async fn initialize(&mut self) -> AppResult<()> {
        // 检查TTS可用性
        if !self.is_available() {
            return Err(AppError::Config("语音合成不可用".to_string()));
        }
        
        info!("语音合成引擎初始化完成");
        Ok(())
    }
    
    /// 语音合成
    pub async fn synthesize(&self, text: &str) -> AppResult<Vec<u8>> {
        if self.voice_config.use_edge_tts {
            self.edge_tts_synthesize(text).await
        } else {
            self.mock_synthesize(text).await
        }
    }
    
    /// 使用Edge TTS进行语音合成
    async fn edge_tts_synthesize(&self, text: &str) -> AppResult<Vec<u8>> {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("mindwolf_tts_{}.wav", Utc::now().timestamp()));
        
        // 构建edge-tts命令
        let output = Command::new("edge-tts")
            .arg("--voice")
            .arg(&self.voice_config.voice_name)
            .arg("--text")
            .arg(text)
            .arg("--write-media")
            .arg(&output_path)
            .arg("--write-subtitles")
            .arg("/dev/null") // 忽略字幕文件
            .output()
            .map_err(|e| AppError::Io(format!("执行edge-tts失败: {}", e)))?;
        
        if output.status.success() {
            // 读取生成的音频文件
            let audio_data = fs::read(&output_path).await
                .map_err(|e| AppError::Io(format!("读取TTS音频文件失败: {}", e)))?;
            
            // 清理临时文件
            let _ = fs::remove_file(&output_path).await;
            
            debug!("TTS合成成功，音频大小: {} 字节", audio_data.len());
            Ok(audio_data)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Io(format!("TTS合成失败: {}", error)))
        }
    }
    
    /// 模拟语音合成（用于演示）
    async fn mock_synthesize(&self, text: &str) -> AppResult<Vec<u8>> {
        info!("模拟语音合成: {}", text);
        
        // 模拟处理延时
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        // 返回模拟的音频数据（实际上是空的WAV文件头）
        let mock_wav_header = vec![
            // WAV文件头（44字节）
            0x52, 0x49, 0x46, 0x46, // "RIFF"
            0x24, 0x00, 0x00, 0x00, // 文件大小-8
            0x57, 0x41, 0x56, 0x45, // "WAVE"
            0x66, 0x6D, 0x74, 0x20, // "fmt "
            0x10, 0x00, 0x00, 0x00, // fmt chunk大小
            0x01, 0x00,             // 音频格式(PCM)
            0x01, 0x00,             // 声道数
            0x40, 0x1F, 0x00, 0x00, // 采样率(8000)
            0x80, 0x3E, 0x00, 0x00, // 字节率
            0x02, 0x00,             // 块对齐
            0x10, 0x00,             // 位深度
            0x64, 0x61, 0x74, 0x61, // "data"
            0x00, 0x00, 0x00, 0x00, // 数据大小
        ];
        
        Ok(mock_wav_header)
    }
    
    /// 设置语音参数
    pub fn set_voice_config(&mut self, config: TTSVoiceConfig) {
        self.voice_config = config;
    }
    
    /// 获取可用的语音列表
    pub async fn get_available_voices(&self) -> AppResult<Vec<VoiceInfo>> {
        if self.voice_config.use_edge_tts {
            self.get_edge_tts_voices().await
        } else {
            Ok(self.get_mock_voices())
        }
    }
    
    /// 获取Edge TTS可用语音
    async fn get_edge_tts_voices(&self) -> AppResult<Vec<VoiceInfo>> {
        let output = Command::new("edge-tts")
            .arg("--list-voices")
            .output()
            .map_err(|e| AppError::Io(format!("获取Edge TTS语音列表失败: {}", e)))?;
        
        if output.status.success() {
            let voices_text = String::from_utf8_lossy(&output.stdout);
            Ok(self.parse_edge_tts_voices(&voices_text))
        } else {
            Ok(self.get_mock_voices())
        }
    }
    
    /// 解析Edge TTS语音列表
    fn parse_edge_tts_voices(&self, voices_text: &str) -> Vec<VoiceInfo> {
        let mut voices = Vec::new();
        
        // 简化的解析逻辑
        for line in voices_text.lines() {
            if line.contains("zh-CN") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(name) = parts.get(1) {
                    voices.push(VoiceInfo {
                        name: name.to_string(),
                        language: "zh-CN".to_string(),
                        gender: if name.contains("Male") { "Male" } else { "Female" }.to_string(),
                        description: line.to_string(),
                    });
                }
            }
        }
        
        voices
    }
    
    /// 获取模拟语音列表
    fn get_mock_voices(&self) -> Vec<VoiceInfo> {
        vec![
            VoiceInfo {
                name: "zh-CN-XiaoxiaoNeural".to_string(),
                language: "zh-CN".to_string(),
                gender: "Female".to_string(),
                description: "中文女声（晓晓）".to_string(),
            },
            VoiceInfo {
                name: "zh-CN-YunxiNeural".to_string(),
                language: "zh-CN".to_string(),
                gender: "Male".to_string(),
                description: "中文男声（云希）".to_string(),
            },
            VoiceInfo {
                name: "zh-CN-YunyangNeural".to_string(),
                language: "zh-CN".to_string(),
                gender: "Male".to_string(),
                description: "中文男声（云扬）".to_string(),
            },
        ]
    }
    
    /// 检查TTS可用性
    pub fn is_available(&self) -> bool {
        if self.voice_config.use_edge_tts {
            self.check_edge_tts_available()
        } else {
            true // 模拟模式总是可用
        }
    }
    
    /// 检查Edge TTS是否可用
    fn check_edge_tts_available(&self) -> bool {
        Command::new("edge-tts")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// 语音信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub name: String,
    pub language: String,
    pub gender: String,
    pub description: String,
}

/// TTS合成结果
#[derive(Debug, Clone)]
pub struct TTSResult {
    pub audio_data: Vec<u8>,
    pub duration_ms: u32,
    pub format: AudioFormat,
}

/// 音频格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    Wav,
    Mp3,
    Ogg,
}

/// TTS合成选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSOptions {
    pub voice_name: String,
    pub speed: f32,     // 0.5 - 2.0
    pub pitch: f32,    // 0.5 - 2.0
    pub volume: f32,   // 0.0 - 1.0
    pub format: AudioFormat,
}

impl Default for TTSOptions {
    fn default() -> Self {
        Self {
            voice_name: "zh-CN-XiaoxiaoNeural".to_string(),
            speed: 1.0,
            pitch: 1.0,
            volume: 0.8,
            format: AudioFormat::Wav,
        }
    }
}
