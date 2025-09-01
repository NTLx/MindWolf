use crate::types::LLMConfig;
use crate::error::{AppResult, AppError};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
use log::{info, warn, error};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// LLM客户端管理器
#[derive(Clone)]
pub struct LLMClient {
    client: Client,
    config: LLMConfig,
}

impl LLMClient {
    /// 创建新的LLM客户端
    pub fn new(config: LLMConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }
    
    /// 发送聊天补全请求（传统API）
    pub async fn chat_completion(&self, messages: Vec<ChatMessage>) -> AppResult<String> {
        if self.config.use_realtime_api {
            // 使用实时API
            self.realtime_completion(messages).await
        } else {
            // 使用传统API
            self.traditional_completion(messages).await
        }
    }
    
    /// 传统聊天补全请求
    async fn traditional_completion(&self, messages: Vec<ChatMessage>) -> AppResult<String> {
        let request_body = json!({
            "model": self.config.model,
            "messages": messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature
        });
        
        let response = self.client
            .post(&format!("{}/v1/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
        
        let response_json: Value = response.json().await?;
        
        // 检查API错误
        if let Some(error) = response_json.get("error") {
            return Err(AppError::LlmApi(
                error.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown API error")
                    .to_string()
            ));
        }
        
        // 提取响应内容
        let content = response_json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| AppError::LlmApi(
                "响应中未找到内容".to_string()
            ))?;
        
        Ok(content.to_string())
    }
    
    /// 实时API聊天补全请求
    async fn realtime_completion(&self, messages: Vec<ChatMessage>) -> AppResult<String> {
        // 1. 创建会话获取临时令牌
        let session_response = self.create_realtime_session().await?;
        
        // 2. 建立WebSocket连接
        let ws_url = format!("wss://{}/v1/realtime?model={}", 
            self.config.base_url.replace("https://", "").replace("http://", ""),
            self.config.model
        );
        
        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&ws_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("OpenAI-Beta", "realtime=v1")
            .body(())?;
        
        let (ws_stream, _) = connect_async(request).await
            .map_err(|e| AppError::LlmApi(format!("WebSocket连接失败: {}", e)))?;
        
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        
        // 3. 发送会话更新事件
        let session_update = json!({
            "type": "session.update",
            "session": {
                "modalities": self.config.modalities,
                "instructions": self.config.instructions,
                "voice": self.config.voice,
                "input_audio_format": self.config.input_audio_format,
                "output_audio_format": self.config.output_audio_format,
                "turn_detection": self.config.turn_detection,
                "temperature": self.config.temperature,
                "max_response_output_tokens": self.config.max_tokens
            }
        });
        
        ws_sender.send(Message::Text(session_update.to_string())).await
            .map_err(|e| AppError::LlmApi(format!("发送会话更新失败: {}", e)))?;
        
        // 4. 发送对话内容
        let conversation_item = json!({
            "type": "conversation.item.create",
            "item": {
                "type": "message",
                "role": "user",
                "content": [{
                    "type": "input_text",
                    "text": messages.last().map(|m| m.content.as_str()).unwrap_or("")
                }]
            }
        });
        
        ws_sender.send(Message::Text(conversation_item.to_string())).await
            .map_err(|e| AppError::LlmApi(format!("发送对话项失败: {}", e)))?;
        
        // 5. 创建响应
        let response_create = json!({
            "type": "response.create",
            "response": {
                "modalities": ["text"],
                "instructions": "请简洁回答用户的问题"
            }
        });
        
        ws_sender.send(Message::Text(response_create.to_string())).await
            .map_err(|e| AppError::LlmApi(format!("创建响应失败: {}", e)))?;
        
        // 6. 接收响应
        let mut response_content = String::new();
        
        while let Some(message) = ws_receiver.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(event) = serde_json::from_str::<Value>(&text) {
                        if let Some(event_type) = event.get("type").and_then(|t| t.as_str()) {
                            match event_type {
                                "response.content_part.added" => {
                                    if let Some(part) = event.get("part") {
                                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                            response_content.push_str(text);
                                        }
                                    }
                                }
                                "response.done" => {
                                    break;
                                }
                                "error" => {
                                    let error_msg = event.get("error")
                                        .and_then(|e| e.get("message"))
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("Unknown error");
                                    return Err(AppError::LlmApi(format!("实时API错误: {}", error_msg)));
                                }
                                _ => {
                                    // 处理其他事件类型
                                    info!("收到事件: {}", event_type);
                                }
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket连接已关闭");
                    break;
                }
                Err(e) => {
                    error!("WebSocket接收错误: {}", e);
                    return Err(AppError::LlmApi(format!("WebSocket接收错误: {}", e)));
                }
                _ => {}
            }
        }
        
        if response_content.is_empty() {
            Err(AppError::LlmApi("未收到有效响应".to_string()))
        } else {
            Ok(response_content)
        }
    }
    
    /// 创建实时会话
    async fn create_realtime_session(&self) -> AppResult<Value> {
        let session_body = json!({
            "model": self.config.model,
            "voice": self.config.voice.as_ref().unwrap_or(&"alloy".to_string())
        });
        
        let response = self.client
            .post(&format!("{}/v1/realtime/sessions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&session_body)
            .send()
            .await?;
        
        let response_json: Value = response.json().await?;
        
        if let Some(error) = response_json.get("error") {
            return Err(AppError::LlmApi(
                error.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown session creation error")
                    .to_string()
            ));
        }
        
        Ok(response_json)
    }
    
    /// 测试连接
    pub async fn test_connection(&self) -> AppResult<bool> {
        let test_messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            id: Some(format!("msg_{}", chrono::Utc::now().timestamp_millis())),
            timestamp: Some(chrono::Utc::now()),
            content_type: Some("text".to_string()),
        }];
        
        match self.chat_completion(test_messages).await {
            Ok(_) => {
                info!("LLM连接测试成功");
                Ok(true)
            }
            Err(e) => {
                warn!("LLM连接测试失败: {}", e);
                Err(e)
            }
        }
    }
}

/// 聊天消息结构
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    // 兼容实时API的附加字段
    pub id: Option<String>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub content_type: Option<String>, // "text", "audio", "function_call"
}

/// 重试配置
#[derive(Clone, Debug)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
        }
    }
}

/// LLM管理器，支持主备和重试机制
#[derive(Clone)]
pub struct LLMManager {
    primary_client: LLMClient,
    fallback_clients: Vec<LLMClient>,
    retry_config: RetryConfig,
}

impl LLMManager {
    /// 创建新的LLM管理器
    pub fn new(
        primary_config: LLMConfig,
        fallback_configs: Vec<LLMConfig>,
    ) -> Self {
        let primary_client = LLMClient::new(primary_config);
        let fallback_clients = fallback_configs
            .into_iter()
            .map(LLMClient::new)
            .collect();
        
        Self {
            primary_client,
            fallback_clients,
            retry_config: RetryConfig::default(),
        }
    }
    
    /// 生成文本，支持重试和备用
    pub async fn generate_with_fallback(&self, prompt: String) -> AppResult<String> {
        // 尝试主要API
        match self.try_generate_with_retry(&self.primary_client, &prompt).await {
            Ok(result) => {
                info!("主要LLM API调用成功");
                return Ok(result);
            }
            Err(e) => {
                warn!("主要LLM API调用失败: {}", e);
            }
        }
        
        // 尝试备用API
        for (index, fallback_client) in self.fallback_clients.iter().enumerate() {
            match self.try_generate_with_retry(fallback_client, &prompt).await {
                Ok(result) => {
                    info!("备用LLM API {} 调用成功", index);
                    return Ok(result);
                }
                Err(e) => {
                    warn!("备用LLM API {} 调用失败: {}", index, e);
                }
            }
        }
        
        Err(AppError::LlmApi("所有LLM API都失败了".to_string()))
    }
    
    /// 带重试的生成
    async fn try_generate_with_retry(
        &self, 
        client: &LLMClient, 
        prompt: &str
    ) -> AppResult<String> {
        for attempt in 1..=self.retry_config.max_attempts {
            match self.generate_single(client, prompt).await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < self.retry_config.max_attempts => {
                    let delay = std::cmp::min(
                        self.retry_config.base_delay_ms * 2_u64.pow(attempt - 1),
                        self.retry_config.max_delay_ms
                    );
                    
                    warn!(
                        "尝试 {}/{} 失败: {}, {}ms后重试...", 
                        attempt, 
                        self.retry_config.max_attempts,
                        e,
                        delay
                    );
                    
                    sleep(Duration::from_millis(delay)).await;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }
    
    /// 单次生成调用
    async fn generate_single(
        &self,
        client: &LLMClient,
        prompt: &str
    ) -> AppResult<String> {
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                id: Some(format!("msg_{}", chrono::Utc::now().timestamp_millis())),
                timestamp: Some(chrono::Utc::now()),
                content_type: Some("text".to_string()),
            }
        ];
        
        client.chat_completion(messages).await
    }
    
    /// 测试所有LLM连接
    pub async fn test_all_connections(&self) -> AppResult<Vec<bool>> {
        let mut results = Vec::new();
        
        // 测试主要连接
        match self.primary_client.test_connection().await {
            Ok(success) => results.push(success),
            Err(_) => results.push(false),
        }
        
        // 测试备用连接
        for client in &self.fallback_clients {
            match client.test_connection().await {
                Ok(success) => results.push(success),
                Err(_) => results.push(false),
            }
        }
        
        Ok(results)
    }
}
