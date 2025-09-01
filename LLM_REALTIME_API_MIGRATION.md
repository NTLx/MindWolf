# MindWolf LLM集成改写：支持OpenAI Realtime API

## 改写概述

本次改写将MindWolf软件中的LLM推理服务接口从传统的OpenAI Chat Completions API全面升级为支持OpenAI Realtime API规范，实现实时语音和文本交互能力。

## 技术变更详情

### 1. 后端架构变更

#### 依赖添加
- 添加 `tokio-tungstenite` WebSocket客户端库
- 添加 `futures-util` 异步流处理库
- 启用WebSocket连接功能

#### 类型系统扩展 (`src-tauri/src/types.rs`)

**新增配置结构：**
- `TurnDetectionConfig` - 语音转向检测配置
- `RealtimeEvent` - 实时事件结构
- `RealtimeSessionConfig` - 实时会话配置
- `TranscriptionConfig` - 转写配置

**扩展LLMConfig：**
```rust
pub struct LLMConfig {
    // 原有字段
    pub provider: LLMProvider,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout: u64,
    
    // 新增实时API字段
    pub use_realtime_api: bool,
    pub voice: Option<String>,
    pub input_audio_format: Option<String>,
    pub output_audio_format: Option<String>,
    pub modalities: Vec<String>,
    pub instructions: Option<String>,
    pub turn_detection: Option<TurnDetectionConfig>,
}
```

#### LLM客户端重写 (`src-tauri/src/llm.rs`)

**核心功能：**
1. **双模式支持**：传统HTTP API + WebSocket实时API
2. **实时会话管理**：自动创建临时令牌、建立WebSocket连接
3. **事件驱动通信**：支持实时事件收发
4. **语音交互**：支持音频流和文本混合对话

**关键方法：**
- `chat_completion()` - 统一入口，根据配置选择API模式
- `traditional_completion()` - 传统HTTP API实现
- `realtime_completion()` - 实时WebSocket API实现
- `create_realtime_session()` - 创建实时会话

**WebSocket事件处理：**
- `session.update` - 更新会话配置
- `conversation.item.create` - 创建对话项
- `response.create` - 触发AI响应
- `response.content_part.added` - 接收响应内容
- `response.done` - 响应完成

### 2. 前端界面增强

#### 类型定义扩展 (`src/types/index.ts`)
```typescript
interface LLMConfig {
  // 原有字段
  provider: string;
  apiKey: string;
  baseUrl: string;
  model: string;
  maxTokens: number;
  temperature: number;
  timeout: number;
  
  // 实时API新增字段
  useRealtimeApi?: boolean;
  voice?: string;
  inputAudioFormat?: string;
  outputAudioFormat?: string;
  modalities?: string[];
  instructions?: string;
  turnDetection?: TurnDetectionConfig;
}
```

#### 设置界面重构 (`src/views/Settings.vue`)

**新增配置选项：**
- 实时API开关
- 语音类型选择（Alloy, Echo, Shimmer等）
- 音频格式配置（PCM16, G711等）
- 支持模态选择（文本/音频）
- 系统指令自定义
- 语音检测配置
- 连接测试功能

### 3. 配置管理更新

#### 默认配置 (`src-tauri/src/config.rs`)
```rust
LLMConfig {
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
    turn_detection: Some(TurnDetectionConfig {
        detection_type: "server_vad".to_string(),
        threshold: Some(0.5),
        prefix_padding_ms: Some(300),
        silence_duration_ms: Some(200),
    }),
}
```

## API规范对接详情

### WebSocket连接流程
1. **会话创建**：`POST /v1/realtime/sessions` 获取临时令牌
2. **WebSocket连接**：`wss://{base_url}/v1/realtime?model={model}`
3. **认证头设置**：
   - `Authorization: Bearer {api_key}`
   - `OpenAI-Beta: realtime=v1`

### 实时事件交互
```json
// 会话更新
{
  "type": "session.update",
  "session": {
    "modalities": ["text", "audio"],
    "instructions": "你是一个狼人杀游戏AI助手",
    "voice": "alloy",
    "input_audio_format": "pcm16",
    "output_audio_format": "pcm16",
    "temperature": 0.7
  }
}

// 创建对话项
{
  "type": "conversation.item.create",
  "item": {
    "type": "message",
    "role": "user",
    "content": [{
      "type": "input_text",
      "text": "用户输入内容"
    }]
  }
}

// 创建响应
{
  "type": "response.create",
  "response": {
    "modalities": ["text"],
    "instructions": "请简洁回答用户的问题"
  }
}
```

## 兼容性保证

### 向后兼容
- 保持原有传统API接口不变
- 通过`use_realtime_api`配置开关控制
- 默认使用传统API，确保现有功能正常

### 渐进式迁移
- 用户可在设置中选择启用实时API
- 支持运行时动态切换API模式
- 提供连接测试功能验证配置

## 使用指南

### 启用实时API
1. 进入设置 -> AI设置
2. 开启"使用实时API"开关
3. 配置语音类型、音频格式等参数
4. 点击"测试连接"验证配置
5. 保存设置

### 配置说明
- **语音类型**：选择AI使用的语音风格
- **音频格式**：设置输入输出音频编码格式
- **支持模态**：选择支持文本和/或音频交互
- **系统指令**：自定义AI的行为指令
- **语音检测**：配置语音活动检测算法

## 技术优势

### 实时交互
- WebSocket长连接，低延迟
- 支持语音和文本混合对话
- 流式响应，更好的用户体验

### 灵活配置
- 支持多种语音风格
- 可调节音频格式和质量
- 自定义AI行为指令

### 稳定可靠
- 保持传统API作为后备方案
- 完善的错误处理和重试机制
- 连接状态监控和自动恢复

## 未来扩展

### 语音功能
- 集成实时语音识别(ASR)
- 实时语音合成(TTS)
- 音频流处理和优化

### 多模态交互
- 支持图像输入
- 实时屏幕共享
- 文档处理能力

### 性能优化
- WebSocket连接池
- 音频压缩算法
- 网络自适应调节

---

本次改写完全按照 https://docs.newapi.pro/api/openai-realtime/ 文档规范实现，确保与NewAPI服务完全兼容。