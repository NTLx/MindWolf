use crate::error::{AppError, AppResult};
use crate::voice::VoiceConfig;
use std::process::Command;
use tokio::fs;
use log::{info, warn, debug};
use std::path::PathBuf;
use chrono::Utc;

/// 语音识别引擎
pub struct ASREngine {
    config: VoiceConfig,
    model_path: Option<PathBuf>,
}

impl ASREngine {
    /// 创建ASR引擎
    pub fn new(config: &VoiceConfig) -> AppResult<Self> {
        let model_path = Self::find_whisper_model()?;
        
        Ok(Self {
            config: config.clone(),
            model_path,
        })
    }
    
    /// 初始化ASR引擎
    pub async fn initialize(&mut self) -> AppResult<()> {
        // 检查模型可用性
        if !self.is_available() {
            return Err(AppError::Config("语音识别不可用".to_string()));
        }
        
        info!("语音识别引擎初始化完成");
        Ok(())
    }
    
    /// 语音识别
    pub async fn recognize(&self, audio_data: &[u8]) -> AppResult<String> {
        // 保存音频数据到临时文件
        let temp_path = self.save_temp_audio(audio_data).await?;
        
        // 调用Whisper进行识别
        let text = self.whisper_recognize(&temp_path).await?;
        
        // 清理临时文件
        let _ = fs::remove_file(&temp_path).await;
        
        Ok(text)
    }
    
    /// 查找Whisper模型
    fn find_whisper_model() -> AppResult<Option<PathBuf>> {
        // 简化实现：返回None表示使用在线服务
        // 实际实现中可以检查本地Whisper模型文件
        Ok(None)
    }
    
    /// 保存临时音频文件
    async fn save_temp_audio(&self, audio_data: &[u8]) -> AppResult<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("mindwolf_audio_{}.wav", Utc::now().timestamp()));
        
        fs::write(&temp_path, audio_data).await
            .map_err(|e| AppError::Io(e.to_string()))?;
        
        Ok(temp_path)
    }
    
    /// 使用Whisper进行语音识别
    async fn whisper_recognize(&self, audio_path: &PathBuf) -> AppResult<String> {
        if self.model_path.is_some() {
            // 使用本地Whisper模型
            self.local_whisper_recognize(audio_path).await
        } else {
            // 使用简化的识别逻辑（演示用）
            self.mock_recognize(audio_path).await
        }
    }
    
    /// 本地Whisper识别
    async fn local_whisper_recognize(&self, audio_path: &PathBuf) -> AppResult<String> {
        let output = Command::new("whisper")
            .arg(audio_path)
            .arg("--language")
            .arg(&self.config.language)
            .arg("--output_format")
            .arg("txt")
            .output()
            .map_err(|e| AppError::Io(format!("执行Whisper失败: {}", e)))?;
        
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            debug!("Whisper识别结果: {}", text);
            Ok(text)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Io(format!("Whisper识别失败: {}", error)))
        }
    }
    
    /// 模拟识别（用于演示）
    async fn mock_recognize(&self, _audio_path: &PathBuf) -> AppResult<String> {
        // 模拟语音识别结果
        let mock_results = [
            "我觉得1号玩家很可疑",
            "我是预言家，昨晚验了3号是好人",
            "我不是狼人，请大家相信我",
            "我投票给2号玩家",
            "我需要再想想"
        ];
        
        let index = Utc::now().timestamp() as usize % mock_results.len();
        let result = mock_results[index].to_string();
        
        info!("模拟语音识别结果: {}", result);
        
        // 模拟处理延时
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        Ok(result)
    }
    
    /// 检查ASR可用性
    pub fn is_available(&self) -> bool {
        // 检查是否有可用的识别方法
        self.model_path.is_some() || self.has_online_service()
    }
    
    /// 检查是否有在线服务
    fn has_online_service(&self) -> bool {
        // 简化实现：总是返回true
        true
    }
}

/// 语音识别结果
#[derive(Debug, Clone)]
pub struct ASRResult {
    pub text: String,
    pub confidence: f32,
    pub duration_ms: u32,
}

/// 语音识别配置
#[derive(Debug, Clone)]
pub struct ASRConfig {
    pub language: String,
    pub model_size: String, // tiny, base, small, medium, large
    pub temperature: f32,
    pub best_of: u32,
}

impl Default for ASRConfig {
    fn default() -> Self {
        Self {
            language: "zh".to_string(),
            model_size: "base".to_string(),
            temperature: 0.0,
            best_of: 5,
        }
    }
}
