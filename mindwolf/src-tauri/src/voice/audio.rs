use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
    pub buffer_size: usize,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub volume: f32,
    pub noise_reduction: bool,
    pub auto_gain_control: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 1,
            bit_depth: 16,
            buffer_size: 1024,
            input_device: None,
            output_device: None,
            volume: 1.0,
            noise_reduction: true,
            auto_gain_control: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_default: bool,
    pub sample_rates: Vec<u32>,
    pub channels: Vec<u16>,
}

pub struct AudioManager {
    settings: Arc<Mutex<AudioSettings>>,
    is_recording: Arc<Mutex<bool>>,
    is_playing: Arc<Mutex<bool>>,
    devices: Arc<Mutex<Vec<AudioDevice>>>,
    callbacks: Arc<Mutex<HashMap<String, Box<dyn Fn(Vec<f32>) + Send + Sync>>>>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(AudioSettings::default())),
            is_recording: Arc::new(Mutex::new(false)),
            is_playing: Arc::new(Mutex::new(false)),
            devices: Arc::new(Mutex::new(Vec::new())),
            callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 初始化音频系统
    pub async fn initialize(&self) -> Result<()> {
        log::info("正在初始化音频系统...");
        
        // 扫描音频设备
        self.scan_devices().await?;
        
        // 设置默认设备
        self.setup_default_devices().await?;
        
        log::info("音频系统初始化完成");
        Ok(())
    }

    /// 扫描可用的音频设备
    pub async fn scan_devices(&self) -> Result<()> {
        let mut devices = self.devices.lock().await;
        devices.clear();
        
        // 模拟扫描音频设备
        // 在实际实现中，这里会使用 cpal 或其他音频库来获取设备列表
        devices.push(AudioDevice {
            id: "default_input".to_string(),
            name: "默认麦克风".to_string(),
            is_input: true,
            is_default: true,
            sample_rates: vec![8000, 16000, 22050, 44100, 48000],
            channels: vec![1, 2],
        });
        
        devices.push(AudioDevice {
            id: "default_output".to_string(),
            name: "默认扬声器".to_string(),
            is_input: false,
            is_default: true,
            sample_rates: vec![8000, 16000, 22050, 44100, 48000],
            channels: vec![1, 2],
        });
        
        log::info(&format!("扫描到 {} 个音频设备", devices.len()));
        Ok(())
    }

    /// 获取音频设备列表
    pub async fn get_devices(&self) -> Result<Vec<AudioDevice>> {
        let devices = self.devices.lock().await;
        Ok(devices.clone())
    }

    /// 设置默认音频设备
    async fn setup_default_devices(&self) -> Result<()> {
        let devices = self.devices.lock().await;
        let mut settings = self.settings.lock().await;
        
        // 设置默认输入设备
        if let Some(input_device) = devices.iter().find(|d| d.is_input && d.is_default) {
            settings.input_device = Some(input_device.id.clone());
        }
        
        // 设置默认输出设备
        if let Some(output_device) = devices.iter().find(|d| !d.is_input && d.is_default) {
            settings.output_device = Some(output_device.id.clone());
        }
        
        Ok(())
    }

    /// 开始录音
    pub async fn start_recording(&self) -> Result<()> {
        let mut is_recording = self.is_recording.lock().await;
        if *is_recording {
            return Ok(());
        }
        
        let settings = self.settings.lock().await;
        log::info(&format!("开始录音，设备: {:?}", settings.input_device));
        
        // 在实际实现中，这里会启动音频录制流
        // 使用 cpal 或其他音频库来捕获音频数据
        *is_recording = true;
        
        Ok(())
    }

    /// 停止录音
    pub async fn stop_recording(&self) -> Result<Vec<u8>> {
        let mut is_recording = self.is_recording.lock().await;
        if !*is_recording {
            return Ok(Vec::new());
        }
        
        *is_recording = false;
        log::info("停止录音");
        
        // 在实际实现中，这里会返回录制的音频数据
        Ok(Vec::new())
    }

    /// 播放音频数据
    pub async fn play_audio(&self, audio_data: Vec<u8>) -> Result<()> {
        let mut is_playing = self.is_playing.lock().await;
        if *is_playing {
            log::warn("音频播放中，跳过新的播放请求");
            return Ok(());
        }
        
        let settings = self.settings.lock().await;
        log::info(&format!("开始播放音频，设备: {:?}", settings.output_device));
        
        *is_playing = true;
        
        // 在实际实现中，这里会播放音频数据
        // 使用 cpal 或其他音频库来播放音频
        
        // 模拟播放完成
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        *is_playing = false;
        
        log::info("音频播放完成");
        Ok(())
    }

    /// 设置音频参数
    pub async fn set_settings(&self, new_settings: AudioSettings) -> Result<()> {
        let mut settings = self.settings.lock().await;
        *settings = new_settings;
        log::info("音频设置已更新");
        Ok(())
    }

    /// 获取当前音频设置
    pub async fn get_settings(&self) -> Result<AudioSettings> {
        let settings = self.settings.lock().await;
        Ok(settings.clone())
    }

    /// 检查是否正在录音
    pub async fn is_recording(&self) -> bool {
        *self.is_recording.lock().await
    }

    /// 检查是否正在播放
    pub async fn is_playing(&self) -> bool {
        *self.is_playing.lock().await
    }

    /// 设置音频输入回调
    pub async fn set_audio_callback<F>(&self, id: String, callback: F) -> Result<()>
    where
        F: Fn(Vec<f32>) + Send + Sync + 'static,
    {
        let mut callbacks = self.callbacks.lock().await;
        callbacks.insert(id, Box::new(callback));
        Ok(())
    }

    /// 移除音频回调
    pub async fn remove_audio_callback(&self, id: &str) -> Result<()> {
        let mut callbacks = self.callbacks.lock().await;
        callbacks.remove(id);
        Ok(())
    }

    /// 获取音频级别（音量）
    pub async fn get_input_level(&self) -> Result<f32> {
        // 在实际实现中，这里会返回当前输入的音频级别
        // 用于显示麦克风音量指示器
        Ok(0.5) // 模拟返回50%的音量级别
    }

    /// 设置输出音量
    pub async fn set_output_volume(&self, volume: f32) -> Result<()> {
        let mut settings = self.settings.lock().await;
        settings.volume = volume.clamp(0.0, 1.0);
        log::info(&format!("输出音量设置为: {:.1}%", settings.volume * 100.0));
        Ok(())
    }

    /// 获取输出音量
    pub async fn get_output_volume(&self) -> Result<f32> {
        let settings = self.settings.lock().await;
        Ok(settings.volume)
    }

    /// 启用/禁用噪音抑制
    pub async fn set_noise_reduction(&self, enabled: bool) -> Result<()> {
        let mut settings = self.settings.lock().await;
        settings.noise_reduction = enabled;
        log::info(&format!("噪音抑制: {}", if enabled { "开启" } else { "关闭" }));
        Ok(())
    }

    /// 启用/禁用自动增益控制
    pub async fn set_auto_gain_control(&self, enabled: bool) -> Result<()> {
        let mut settings = self.settings.lock().await;
        settings.auto_gain_control = enabled;
        log::info(&format!("自动增益控制: {}", if enabled { "开启" } else { "关闭" }));
        Ok(())
    }

    /// 音频格式转换
    pub fn convert_audio_format(&self, data: Vec<u8>, from_rate: u32, to_rate: u32) -> Result<Vec<u8>> {
        // 在实际实现中，这里会进行音频格式转换
        // 包括采样率转换、声道转换等
        log::info(&format!("音频格式转换: {}Hz -> {}Hz", from_rate, to_rate));
        Ok(data)
    }

    /// 应用音频滤波器
    pub fn apply_audio_filters(&self, data: Vec<f32>, settings: &AudioSettings) -> Result<Vec<f32>> {
        let mut filtered_data = data;
        
        // 应用噪音抑制
        if settings.noise_reduction {
            filtered_data = self.apply_noise_reduction(filtered_data)?;
        }
        
        // 应用自动增益控制
        if settings.auto_gain_control {
            filtered_data = self.apply_auto_gain_control(filtered_data)?;
        }
        
        Ok(filtered_data)
    }

    /// 应用噪音抑制算法
    fn apply_noise_reduction(&self, data: Vec<f32>) -> Result<Vec<f32>> {
        // 简单的噪音门限实现
        let threshold = 0.01; // 噪音门限
        let processed_data: Vec<f32> = data.iter()
            .map(|&sample| {
                if sample.abs() < threshold {
                    0.0
                } else {
                    sample
                }
            })
            .collect();
        
        Ok(processed_data)
    }

    /// 应用自动增益控制
    fn apply_auto_gain_control(&self, data: Vec<f32>) -> Result<Vec<f32>> {
        if data.is_empty() {
            return Ok(data);
        }
        
        // 计算RMS值
        let rms = (data.iter().map(|&x| x * x).sum::<f32>() / data.len() as f32).sqrt();
        
        // 目标RMS值
        let target_rms = 0.1;
        
        // 计算增益
        let gain = if rms > 0.0 {
            (target_rms / rms).min(4.0) // 限制最大增益为4倍
        } else {
            1.0
        };
        
        // 应用增益
        let processed_data: Vec<f32> = data.iter()
            .map(|&sample| (sample * gain).clamp(-1.0, 1.0))
            .collect();
        
        Ok(processed_data)
    }

    /// 关闭音频管理器
    pub async fn shutdown(&self) -> Result<()> {
        // 停止所有音频操作
        if self.is_recording().await {
            self.stop_recording().await?;
        }
        
        // 清除回调
        let mut callbacks = self.callbacks.lock().await;
        callbacks.clear();
        
        log::info("音频管理器已关闭");
        Ok(())
    }
}

// 音频工具函数
impl AudioManager {
    /// 检测静音段
    pub fn detect_silence(&self, data: &[f32], threshold: f32, min_duration: usize) -> Vec<(usize, usize)> {
        let mut silence_segments = Vec::new();
        let mut start = None;
        
        for (i, &sample) in data.iter().enumerate() {
            if sample.abs() < threshold {
                if start.is_none() {
                    start = Some(i);
                }
            } else if let Some(silence_start) = start {
                if i - silence_start >= min_duration {
                    silence_segments.push((silence_start, i));
                }
                start = None;
            }
        }
        
        // 处理结尾的静音段
        if let Some(silence_start) = start {
            if data.len() - silence_start >= min_duration {
                silence_segments.push((silence_start, data.len()));
            }
        }
        
        silence_segments
    }

    /// 音频数据归一化
    pub fn normalize_audio(&self, data: Vec<f32>) -> Vec<f32> {
        if data.is_empty() {
            return data;
        }
        
        let max_val = data.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        
        if max_val > 0.0 {
            data.iter().map(|&x| x / max_val).collect()
        } else {
            data
        }
    }

    /// 计算音频功率谱
    pub fn compute_power_spectrum(&self, data: &[f32]) -> Vec<f32> {
        // 简化的功率谱计算
        // 在实际实现中会使用FFT
        let window_size = 256;
        let mut spectrum = Vec::new();
        
        for chunk in data.chunks(window_size) {
            let power = chunk.iter().map(|&x| x * x).sum::<f32>() / chunk.len() as f32;
            spectrum.push(power);
        }
        
        spectrum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_manager_creation() {
        let manager = AudioManager::new();
        assert!(!manager.is_recording().await);
        assert!(!manager.is_playing().await);
    }

    #[tokio::test]
    async fn test_device_scanning() {
        let manager = AudioManager::new();
        manager.scan_devices().await.unwrap();
        
        let devices = manager.get_devices().await.unwrap();
        assert!(!devices.is_empty());
    }

    #[tokio::test]
    async fn test_settings() {
        let manager = AudioManager::new();
        let settings = AudioSettings {
            sample_rate: 44100,
            volume: 0.8,
            ..Default::default()
        };
        
        manager.set_settings(settings.clone()).await.unwrap();
        let retrieved_settings = manager.get_settings().await.unwrap();
        
        assert_eq!(retrieved_settings.sample_rate, 44100);
        assert_eq!(retrieved_settings.volume, 0.8);
    }

    #[test]
    fn test_noise_reduction() {
        let manager = AudioManager::new();
        let data = vec![0.001, 0.5, 0.002, 0.8, 0.0001];
        let filtered = manager.apply_noise_reduction(data).unwrap();
        
        // 小于阈值的值应该被置为0
        assert_eq!(filtered[0], 0.0);
        assert_eq!(filtered[2], 0.0);
        assert_eq!(filtered[4], 0.0);
        // 大于阈值的值应该保持不变
        assert_eq!(filtered[1], 0.5);
        assert_eq!(filtered[3], 0.8);
    }

    #[test]
    fn test_silence_detection() {
        let manager = AudioManager::new();
        let data = vec![0.001, 0.002, 0.5, 0.8, 0.001, 0.002, 0.003];
        let silence_segments = manager.detect_silence(&data, 0.01, 2);
        
        assert_eq!(silence_segments.len(), 2);
        assert_eq!(silence_segments[0], (0, 2));
        assert_eq!(silence_segments[1], (4, 7));
    }

    #[test]
    fn test_audio_normalization() {
        let manager = AudioManager::new();
        let data = vec![0.5, 1.0, -0.8, 0.2];
        let normalized = manager.normalize_audio(data);
        
        let max_val = normalized.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        assert!((max_val - 1.0).abs() < 1e-6);
    }
}