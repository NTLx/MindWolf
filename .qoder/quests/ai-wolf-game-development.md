# 智狼 (MindWolf) AI狼人杀软件技术设计文档

## 1. 项目概述

智狼 (MindWolf) 是一款基于Tauri+Vue3+TypeScript技术栈开发的AI狼人杀桌面应用。该软件允许单个人类玩家与多个AI玩家进行狼人杀游戏，AI能够进行逻辑推理、发言欺骗和策略博弈，提供真实的狼人杀游戏体验。

### 1.1 核心特性
- **智能AI对手**: AI能够扮演任何狼人杀角色，具备逻辑推理和语言欺骗能力
- **单人游戏体验**: 一位人类玩家即可开始游戏
- **跨平台桌面应用**: 基于Tauri框架，支持Windows、macOS、Linux
- **实时语音交互**: 支持语音输入和AI语音合成
- **复盘分析系统**: 详细的游戏复盘和AI决策分析

## 2. 技术栈选型

### 2.1 前端技术栈
- **框架**: Vue 3.x (Composition API)
- **语言**: TypeScript 5.x
- **构建工具**: Vite 4.x
- **UI组件库**: Element Plus / Naive UI
- **状态管理**: Pinia
- **路由**: Vue Router 4.x
- **样式**: SCSS + CSS Modules

### 2.2 后端技术栈 (Tauri Core)
- **框架**: Tauri 2.x (最新稳定版)
- **语言**: Rust 1.75+ (最新稳定版)
- **AI推理**: OpenAI兼容API + Python (推理引擎)
- **数据库**: SQLite (本地存储)
- **AI模型**: OpenAI兼容API接口 (主要) + 本地LLM备选

### 2.3 AI技术栈
- **自然语言处理**: OpenAI兼容API + 本地fallback
- **逻辑推理引擎**: 基于规则的专家系统
- **API客户端**: reqwest (Rust) + openai-python
- **语音处理**: 
  - ASR: whisper-cpp (本地语音识别)
  - TTS: edge-tts / 本地TTS引擎

## 3. 系统架构

### 3.1 整体架构图

```
graph TB
    subgraph "前端层 (Vue3 + TypeScript)"
        UI[用户界面]
        GM[游戏管理器]
        SM[状态管理 Pinia]
        Voice[语音模块]
    end
    
    subgraph "Tauri Core (Rust)"
        API[Tauri API]
        GameEngine[游戏引擎]
        DB[SQLite数据库]
        Process[进程管理]
    end
    
    subgraph "AI后端 (Python)"
        AIAgent[AI代理]
        NLP[自然语言处理]
        Logic[逻辑推理引擎]
        Strategy[策略模块]
    end
    
    subgraph "外部服务"
        OpenAI[OpenAI兼容API]
        LocalLLM[本地LLM备选]
        TTS[语音合成]
        ASR[语音识别]
    end
    
    UI --> GM
    GM --> SM
    UI --> Voice
    GM --> API
    API --> GameEngine
    GameEngine --> DB
    API --> Process
    Process --> AIAgent
    AIAgent --> NLP
    AIAgent --> Logic
    AIAgent --> Strategy
    NLP --> OpenAI
    NLP --> LocalLLM
    Voice --> ASR
    Voice --> TTS
```

### 3.2 模块分层

| 层级 | 技术栈 | 职责 |
|------|--------|------|
| 表现层 | Vue3 + TypeScript | 用户交互、游戏界面、状态展示 |
| 业务层 | Tauri (Rust) | 游戏逻辑、规则引擎、数据管理 |
| AI层 | Python | AI推理、自然语言处理、策略决策 |
| 数据层 | SQLite | 游戏数据、配置、历史记录 |

## 4. 核心模块设计

### 4.1 LLM配置与管理模块

#### 4.1.1 API配置管理
```typescript
interface LLMConfig {
  provider: 'openai' | 'anthropic' | 'azure' | 'custom';
  apiKey: string;
  baseUrl: string;
  model: string;
  maxTokens: number;
  temperature: number;
  timeout: number;
}

interface LLMSettings {
  primaryConfig: LLMConfig;
  fallbackConfigs: LLMConfig[];
  retryAttempts: number;
  enableLocalFallback: boolean;
  requestCaching: boolean;
}
```

#### 4.1.2 API客户端管理
``rust
// src-tauri/src/llm_client.rs
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct LLMClient {
    client: Client,
    config: LLMConfig,
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }
    
    pub async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<String, LLMError> {
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
        
        // 更健壮的响应解析（Tauri 2.x优化）
        if let Some(error) = response_json.get("error") {
            return Err(LLMError::ApiError(
                error.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown API error")
                    .to_string()
            ));
        }
        
        let content = response_json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| LLMError::InvalidResponse(
                "响应中未找到内容".to_string()
            ))?;
        
        Ok(content.to_string())
    }
}
```

#### 4.1.3 容错机制 (Tauri 2.x 增强版)
``rust
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use tokio::sync::RwLock;

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

pub struct LLMManager {
    primary_client: Arc<LLMClient>,
    fallback_clients: Vec<Arc<LLMClient>>,
    local_fallback: Option<Arc<dyn LocalLLM + Send + Sync>>,
    retry_config: RetryConfig,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
}

// 熔断器模式，防止连续失败
struct CircuitBreaker {
    failure_count: u32,
    last_failure_time: Option<std::time::Instant>,
    state: CircuitState,
}

#[derive(Debug, PartialEq)]
enum CircuitState {
    Closed,  // 正常状态
    Open,    // 熔断状态
    HalfOpen, // 半开状态
}

impl LLMManager {
    pub fn new(
        primary_config: LLMConfig,
        fallback_configs: Vec<LLMConfig>,
    ) -> Self {
        let primary_client = Arc::new(LLMClient::new(primary_config));
        let fallback_clients = fallback_configs
            .into_iter()
            .map(|config| Arc::new(LLMClient::new(config)))
            .collect();
        
        Self {
            primary_client,
            fallback_clients,
            local_fallback: None,
            retry_config: RetryConfig::default(),
            circuit_breaker: Arc::new(RwLock::new(CircuitBreaker::new())),
        }
    }
    
    pub async fn generate_with_fallback(&self, prompt: String) -> Result<String, LLMError> {
        // 检查熔断器状态
        if self.is_circuit_open().await {
            return self.try_local_fallback(&prompt).await;
        }
        
        // 主要API尝试
        match self.try_generate_with_retry(&self.primary_client, &prompt).await {
            Ok(result) => {
                self.reset_circuit_breaker().await;
                return Ok(result);
            }
            Err(e) => {
                log::warn!("Primary LLM failed: {}", e);
                self.record_failure().await;
            }
        }
        
        // 备用API尝试
        for (index, fallback_client) in self.fallback_clients.iter().enumerate() {
            match self.try_generate_with_retry(fallback_client, &prompt).await {
                Ok(result) => {
                    log::info!("Fallback {} succeeded", index);
                    return Ok(result);
                }
                Err(e) => {
                    log::warn!("Fallback {} failed: {}", index, e);
                }
            }
        }
        
        // 本地模型备用
        self.try_local_fallback(&prompt).await
    }
    
    async fn try_generate_with_retry(
        &self, 
        client: &LLMClient, 
        prompt: &str
    ) -> Result<String, LLMError> {
        for attempt in 1..=self.retry_config.max_attempts {
            match self.generate_single(client, prompt).await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < self.retry_config.max_attempts => {
                    let delay = std::cmp::min(
                        self.retry_config.base_delay_ms * 2_u64.pow(attempt - 1),
                        self.retry_config.max_delay_ms
                    );
                    
                    log::warn!(
                        "Attempt {}/{} failed: {}, retrying in {}ms...", 
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
    
    async fn generate_single(
        &self,
        client: &LLMClient,
        prompt: &str
    ) -> Result<String, LLMError> {
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }
        ];
        
        client.chat_completion(messages).await
    }
    
    async fn try_local_fallback(&self, prompt: &str) -> Result<String, LLMError> {
        if let Some(local_llm) = &self.local_fallback {
            log::info!("Using local fallback model");
            local_llm.generate(prompt).await
                .map_err(|e| LLMError::ApiError(format!("Local fallback failed: {}", e)))
        } else {
            Err(LLMError::ApiError("All providers failed and no local fallback available".to_string()))
        }
    }
    
    async fn is_circuit_open(&self) -> bool {
        let breaker = self.circuit_breaker.read().await;
        breaker.state == CircuitState::Open
    }
    
    async fn record_failure(&self) {
        let mut breaker = self.circuit_breaker.write().await;
        breaker.failure_count += 1;
        breaker.last_failure_time = Some(std::time::Instant::now());
        
        if breaker.failure_count >= 5 {
            breaker.state = CircuitState::Open;
            log::warn!("Circuit breaker opened due to consecutive failures");
        }
    }
    
    async fn reset_circuit_breaker(&self) {
        let mut breaker = self.circuit_breaker.write().await;
        breaker.failure_count = 0;
        breaker.state = CircuitState::Closed;
    }
}

impl CircuitBreaker {
    fn new() -> Self {
        Self {
            failure_count: 0,
            last_failure_time: None,
            state: CircuitState::Closed,
        }
    }
}

// 本地LLM接口
#[async_trait::async_trait]
pub trait LocalLLM {
    async fn generate(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    async fn is_available(&self) -> bool;
}
```

### 4.2 游戏引擎模块

#### 4.2.1 游戏状态管理
``typescript
interface GameState {
  phase: GamePhase;           // 游戏阶段
  day: number;               // 天数
  players: Player[];         // 玩家列表
  deadPlayers: Player[];     // 死亡玩家
  votes: VoteRecord[];       // 投票记录
  gameConfig: GameConfig;    // 游戏配置
  winner: Faction | null;    // 胜利方
}

enum GamePhase {
  PREPARATION = 'preparation',
  NIGHT = 'night',
  DAY_DISCUSSION = 'day_discussion',
  VOTING = 'voting',
  LAST_WORDS = 'last_words',
  GAME_OVER = 'game_over'
}
```

#### 4.2.2 角色系统
``typescript
abstract class Role {
  abstract roleType: RoleType;
  abstract faction: Faction;
  abstract nightAction(target?: Player): ActionResult;
  abstract canUseSkill(): boolean;
}

enum RoleType {
  WEREWOLF = 'werewolf',
  VILLAGER = 'villager',
  SEER = 'seer',
  WITCH = 'witch',
  HUNTER = 'hunter',
  GUARD = 'guard'
}
```

### 4.3 AI代理系统

#### 4.3.1 AI代理架构
```
graph TD
    subgraph "AI Agent"
        Memory[记忆模块]
        Reasoning[推理模块]
        Speech[发言生成]
        Strategy[策略决策]
        Personality[性格模块]
    end
    
    GameState[游戏状态] --> Memory
    Memory --> Reasoning
    Reasoning --> Strategy
    Strategy --> Speech
    Personality --> Speech
    Speech --> Action[行动输出]
```

#### 4.3.2 推理引擎设计
``python
class ReasoningEngine:
    def __init__(self):
        self.knowledge_base = KnowledgeBase()
        self.probability_model = BayesianNetwork()
        self.rule_engine = RuleEngine()
    
    def analyze_game_state(self, game_state: GameState) -> Analysis:
        """分析当前游戏状态，生成概率推断"""
        pass
    
    def update_player_suspicion(self, player_id: str, evidence: Evidence):
        """更新对玩家的怀疑度"""
        pass
    
    def generate_strategy(self, role: Role, game_state: GameState) -> Strategy:
        """根据角色和游戏状态生成策略"""
        pass
```

#### 4.3.3 自然语言处理模块
``python
class NLPModule:
    def __init__(self, llm_manager: LLMManager):
        self.llm_manager = llm_manager
        self.intent_classifier = IntentClassifier()
        self.entity_extractor = EntityExtractor()
        self.prompt_templates = self._load_templates()
    
    def understand_speech(self, text: str, context: GameContext) -> SpeechIntent:
        """理解玩家发言"""
        prompt = self._build_understanding_prompt(text, context)
        response = await self.llm_manager.generate_with_fallback(prompt)
        return self._parse_intent_response(response)
    
    def generate_speech(self, intent: SpeechIntent, context: GameContext, personality: AIPersonality) -> str:
        """生成AI发言"""
        prompt = self._build_generation_prompt(intent, context, personality)
        response = await self.llm_manager.generate_with_fallback(prompt)
        return self._post_process_speech(response)
    
    def _build_understanding_prompt(self, text: str, context: GameContext) -> str:
        return f"""
你是一个狼人杀游戏分析专家。请分析以下玩家发言的意图和关键信息。

游戏背景：
- 当前是第{context.day}天{context.phase}
- 存活玩家：{len(context.alive_players)}人
- 已知信息：{context.public_info}

玩家发言："{text}"

请提取以下信息（JSON格式返回）：
{{
  "intent": "站边/怀疑/辩解/信息分享/其他",
  "targets": ["被提及的玩家ID"],
  "key_claims": ["关键声明"],
  "emotion": "冷静/紧张/愤怒/其他",
  "credibility": 0.0-1.0
}}
        """
    
    def _build_generation_prompt(self, intent: SpeechIntent, context: GameContext, personality: AIPersonality) -> str:
        return f"""
你是一个狼人杀AI玩家，需要根据当前情况生成发言。

角色信息：
- 你的身份：{context.my_role}
- 你的阵营：{context.my_faction}
- 性格特点：{personality.description}

当前情况：
- 第{context.day}天{context.phase}
- 存活玩家：{context.alive_players}
- 场上信息：{context.situation_summary}

发言目标：{intent.objective}
发言策略：{intent.strategy}

要求：
1. 发言长度50-200字
2. 符合你的角色身份和性格
3. 自然流畅，像真人玩家
4. 不要透露真实身份（如果是狼人）
5. 体现逻辑推理过程

请生成发言：
        """
    
    def analyze_deception(self, text: str, speaker_info: PlayerInfo) -> DeceptionScore:
        """分析发言的欺骗性"""
        prompt = f"""
分析以下狼人杀发言的可信度：

发言者信息：{speaker_info}
发言内容："{text}"

请从以下角度分析（JSON格式）：
{{
  "logic_consistency": 0.0-1.0,
  "information_reliability": 0.0-1.0,
  "emotion_authenticity": 0.0-1.0,
  "overall_credibility": 0.0-1.0,
  "suspicious_points": ["可疑点列表"]
}}
        """
        
        response = await self.llm_manager.generate_with_fallback(prompt)
        return self._parse_deception_response(response)
```

### 4.4 用户界面模块

#### 4.4.1 组件架构
```
src/
├── components/
│   ├── Game/
│   │   ├── GameBoard.vue         # 游戏主界面
│   │   ├── PlayerCard.vue        # 玩家卡片
│   │   ├── ChatPanel.vue         # 聊天面板
│   │   ├── VotingPanel.vue       # 投票面板
│   │   └── PhaseIndicator.vue    # 阶段指示器
│   ├── AI/
│   │   ├── AIPlayerCard.vue      # AI玩家卡片
│   │   ├── AIThinkingBubble.vue  # AI思考气泡
│   │   └── AIPersonalitySet.vue  # AI性格设置
│   └── Common/
│       ├── VoiceRecorder.vue     # 语音录制
│       ├── AudioPlayer.vue       # 音频播放
│       └── GameTimer.vue         # 游戏计时器
```

#### 4.4.2 状态管理设计
``typescript
// stores/gameStore.ts
export const useGameStore = defineStore('game', () => {
  const gameState = ref<GameState>()
  const currentPlayer = ref<Player>()
  const chatHistory = ref<ChatMessage[]>([])
  const aiThoughts = ref<Map<string, AIThought>>()
  
  const startGame = async (config: GameConfig) => {
    // 初始化游戏
  }
  
  const processPlayerAction = async (action: PlayerAction) => {
    // 处理玩家行动
  }
  
  const getAIResponse = async (playerId: string) => {
    // 获取AI响应
  }
  
  return {
    gameState,
    currentPlayer,
    chatHistory,
    aiThoughts,
    startGame,
    processPlayerAction,
    getAIResponse
  }
})
```

#### 4.4.3 LLM配置界面
``vue
<!-- LLMSettings.vue -->
<template>
  <div class="llm-settings">
    <h3>智能AI配置</h3>
    
    <!-- 主要API配置 -->
    <div class="config-section">
      <h4>主要AI服务</h4>
      <el-form :model="primaryConfig" label-width="120px">
        <el-form-item label="服务提供商">
          <el-select v-model="primaryConfig.provider">
            <el-option label="OpenAI" value="openai" />
            <el-option label="Anthropic Claude" value="anthropic" />
            <el-option label="Azure OpenAI" value="azure" />
            <el-option label="自定义API" value="custom" />
          </el-select>
        </el-form-item>
        
        <el-form-item label="API地址">
          <el-input 
            v-model="primaryConfig.baseUrl" 
            placeholder="https://api.openai.com"
          />
        </el-form-item>
        
        <el-form-item label="API密钥">
          <el-input 
            v-model="primaryConfig.apiKey" 
            type="password" 
            show-password
            placeholder="sk-..."
          />
        </el-form-item>
        
        <el-form-item label="模型名称">
          <el-input 
            v-model="primaryConfig.model" 
            placeholder="gpt-4"
          />
        </el-form-item>
        
        <el-form-item label="最大令牌">
          <el-slider 
            v-model="primaryConfig.maxTokens" 
            :min="100" 
            :max="4000" 
            show-input
          />
        </el-form-item>
        
        <el-form-item label="创意度">
          <el-slider 
            v-model="primaryConfig.temperature" 
            :min="0" 
            :max="2" 
            :step="0.1" 
            show-input
          />
        </el-form-item>
      </el-form>
    </div>
    
    <!-- 备用配置 -->
    <div class="config-section">
      <h4>备用服务</h4>
      <el-switch 
        v-model="enableFallback"
        active-text="启用备用"
        inactive-text="禁用备用"
      />
      <!-- 备用配置表单... -->
    </div>
    
    <!-- 测试区域 -->
    <div class="test-section">
      <h4>连接测试</h4>
      <el-button @click="testConnection" :loading="testing">
        测试连接
      </el-button>
      <div v-if="testResult" class="test-result">
        <el-alert 
          :type="testResult.success ? 'success' : 'error'"
          :title="testResult.message"
          show-icon
        />
      </div>
    </div>
    
    <div class="action-buttons">
      <el-button type="primary" @click="saveSettings">保存配置</el-button>
      <el-button @click="resetToDefaults">恢复默认</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { useSettingsStore } from '@/stores/settingsStore';

const settingsStore = useSettingsStore();

const primaryConfig = reactive<LLMConfig>({
  provider: 'openai',
  baseUrl: 'https://api.openai.com',
  apiKey: '',
  model: 'gpt-4',
  maxTokens: 2000,
  temperature: 0.7,
  timeout: 30
});

const enableFallback = ref(false);
const testing = ref(false);
const testResult = ref<{success: boolean, message: string} | null>(null);

const testConnection = async () => {
  testing.value = true;
  testResult.value = null;
  
  try {
    const result = await invoke('test_llm_connection', {
      config: primaryConfig
    });
    
    testResult.value = {
      success: true,
      message: '连接成功！AI服务响应正常'
    };
  } catch (error) {
    testResult.value = {
      success: false,
      message: `连接失败：${error}`
    };
  } finally {
    testing.value = false;
  }
};

const saveSettings = async () => {
  await settingsStore.updateLLMConfig(primaryConfig);
  // 显示保存成功提示
};
</script>
```

### 4.5 设置存储模块
```typescript
// stores/settingsStore.ts
export const useSettingsStore = defineStore('settings', () => {
  const llmConfig = ref<LLMSettings>({
    primaryConfig: {
      provider: 'openai',
      baseUrl: 'https://api.openai.com',
      apiKey: '',
      model: 'gpt-4',
      maxTokens: 2000,
      temperature: 0.7,
      timeout: 30
    },
    fallbackConfigs: [],
    retryAttempts: 3,
    enableLocalFallback: false,
    requestCaching: true
  });
  
  const loadSettings = async () => {
    try {
      const saved = await invoke('load_settings');
      if (saved.llmConfig) {
        llmConfig.value = saved.llmConfig;
      }
    } catch (error) {
      console.warn('加载设置失败:', error);
    }
  };
  
  const saveSettings = async () => {
    try {
      await invoke('save_settings', {
        settings: {
          llmConfig: llmConfig.value
        }
      });
    } catch (error) {
      console.error('保存设置失败:', error);
      throw error;
    }
  };
  
  const updateLLMConfig = async (config: LLMConfig) => {
    llmConfig.value.primaryConfig = config;
    await saveSettings();
  };
  
  return {
    llmConfig,
    loadSettings,
    saveSettings,
    updateLLMConfig
  };
});
```

### 4.6 语音交互模块

#### 4.6.1 语音输入处理
```typescript
class VoiceInputHandler {
  private recorder: MediaRecorder | null = null;
  private audioChunks: Blob[] = [];
  
  async startRecording(): Promise<void> {
    // 开始录音
  }
  
  async stopRecording(): Promise<string> {
    // 停止录音并转换为文本
  }
  
  private async transcribeAudio(audioBlob: Blob): Promise<string> {
    // 调用Tauri后端进行语音识别
    return await invoke('transcribe_audio', { audioData: audioBlob });
  }
}
```

#### 4.6.2 语音合成
``rust
// src-tauri/src/tts.rs
use std::process::Command;

pub async fn synthesize_speech(text: String, voice_id: String) -> Result<Vec<u8>, String> {
    // 调用TTS引擎生成语音
    let output = Command::new("python")
        .arg("ai_backend/tts_service.py")
        .arg("--text")
        .arg(&text)
        .arg("--voice")
        .arg(&voice_id)
        .output()
        .map_err(|e| e.to_string())?;
    
    Ok(output.stdout)
}
```

## 5. 数据模型设计

### 5.1 数据库表结构

```
-- 游戏记录表
CREATE TABLE games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT UNIQUE NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    winner_faction TEXT,
    game_config TEXT, -- JSON格式的游戏配置
    final_state TEXT   -- JSON格式的最终游戏状态
);

-- 玩家表
CREATE TABLE players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL,
    player_id TEXT NOT NULL,
    player_name TEXT NOT NULL,
    is_ai BOOLEAN NOT NULL,
    role_type TEXT NOT NULL,
    faction TEXT NOT NULL,
    is_alive BOOLEAN DEFAULT TRUE,
    death_day INTEGER,
    FOREIGN KEY (game_id) REFERENCES games(game_id)
);

-- 游戏日志表
CREATE TABLE game_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL,
    day INTEGER NOT NULL,
    phase TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    event_type TEXT NOT NULL, -- 'speech', 'vote', 'skill', 'death'
    player_id TEXT,
    content TEXT, -- JSON格式的事件内容
    FOREIGN KEY (game_id) REFERENCES games(game_id)
);

-- 设置配置表
CREATE TABLE settings (
    id INTEGER PRIMARY KEY,
    key TEXT UNIQUE NOT NULL,
    value TEXT NOT NULL, -- JSON格式的配置值
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- LLM配置表
CREATE TABLE llm_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL, -- 配置名称
    provider TEXT NOT NULL, -- 'openai', 'anthropic', 'azure', 'custom'
    base_url TEXT NOT NULL,
    api_key TEXT NOT NULL,
    model TEXT NOT NULL,
    max_tokens INTEGER DEFAULT 2000,
    temperature REAL DEFAULT 0.7,
    timeout INTEGER DEFAULT 30,
    is_primary BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 5.2 核心数据结构

``typescript
interface Player {
  id: string;
  name: string;
  isAI: boolean;
  role: Role;
  faction: Faction;
  isAlive: boolean;
  position: number; // 座位号
  aiPersonality?: AIPersonality; // AI性格设置
}

interface ChatMessage {
  id: string;
  playerId: string;
  playerName: string;
  content: string;
  timestamp: Date;
  type: 'speech' | 'system' | 'action';
  isAI: boolean;
}

interface AIThought {
  playerId: string;
  day: number;
  phase: GamePhase;
  thoughtType: 'suspicion' | 'strategy' | 'deduction';
  content: string;
  confidence: number;
  targetPlayers?: string[]; // 相关的其他玩家
}

interface VoteRecord {
  voterId: string;
  targetId: string;
  day: number;
  phase: GamePhase;
  timestamp: Date;
}
```

## 6. AI策略与算法设计

### 6.1 概率推理模型

AI使用贝叶斯网络维护对每个玩家的怀疑度：

```
class SuspicionModel:
    def __init__(self):
        self.player_probabilities = {}  # {player_id: wolf_probability}
        self.evidence_weights = {
            'speech_contradiction': 0.3,
            'voting_pattern': 0.25,
            'night_action_result': 0.4,
            'role_claim_conflict': 0.5
        }
    
    def update_probability(self, player_id: str, evidence_type: str, evidence_strength: float):
        """根据证据更新玩家是狼人的概率"""
        current_prob = self.player_probabilities.get(player_id, 0.5)
        weight = self.evidence_weights[evidence_type]
        
        # 贝叶斯更新
        new_prob = self._bayesian_update(current_prob, evidence_strength, weight)
        self.player_probabilities[player_id] = new_prob
```

### 6.2 策略决策树

```
graph TD
    A[AI回合开始] --> B{我的角色？}
    B -->|狼人| C[狼人策略]
    B -->|村民| D[村民策略]
    B -->|神职| E[神职策略]
    
    C --> C1{是否已暴露？}
    C1 -->|是| C2[混淆视听/带节奏]
    C1 -->|否| C3[伪装身份/潜伏]
    
    D --> D1{是否有可靠信息？}
    D1 -->|是| D2[逻辑推理/站边]
    D1 -->|否| D3[观察分析/跟票]
    
    E --> E1{技能是否已用？}
    E1 -->|是| E2[引导推理/保护身份]
    E1 -->|否| E3[技能使用决策]
```

### 6.3 发言生成策略

```
class SpeechGenerator:
    def __init__(self, llm_client, personality):
        self.llm_client = llm_client
        self.personality = personality
        self.speech_templates = self._load_templates()
    
    def generate_speech(self, context: GameContext, intent: SpeechIntent) -> str:
        """生成AI发言"""
        # 1. 分析当前局势
        situation_analysis = self._analyze_situation(context)
        
        # 2. 确定发言策略
        strategy = self._determine_strategy(intent, situation_analysis)
        
        # 3. 构建提示词
        prompt = self._build_prompt(context, strategy, self.personality)
        
        # 4. 调用LLM生成发言
        raw_speech = self.llm_client.generate(prompt)
        
        # 5. 后处理和过滤
        final_speech = self._post_process(raw_speech, context)
        
        return final_speech
    
    def _build_prompt(self, context: GameContext, strategy: str, personality: AIPersonality) -> str:
        return f"""
        你是一个狼人杀AI玩家，性格特点：{personality.description}
        当前游戏状况：{context.summary}
        你的角色：{context.my_role}
        发言策略：{strategy}
        
        请生成一段符合你性格和策略的发言，要求：
        1. 发言长度50-150字
        2. 符合狼人杀游戏语境
        3. 体现你的性格特点
        4. 实现发言策略目标
        """
```

## 7. 用户体验设计

### 7.1 游戏流程设计

```
sequenceDiagram
    participant U as 人类玩家
    participant G as 游戏引擎
    participant AI as AI玩家们
    
    U->>G: 开始游戏
    G->>G: 分配角色
    G->>U: 显示角色信息
    G->>AI: 初始化AI代理
    
    loop 游戏主循环
        G->>G: 夜晚阶段
        G->>AI: AI夜间行动
        G->>G: 结算夜间结果
        
        G->>G: 白天讨论
        G->>U: 轮到你发言
        U->>G: 语音/文字发言
        G->>AI: AI依次发言
        
        G->>G: 投票阶段
        G->>U: 投票选择
        U->>G: 投票结果
        G->>AI: AI投票
        G->>G: 统计票数
        
        alt 游戏结束
            G->>U: 显示游戏结果
            G->>U: 复盘分析
        end
    end
```

### 7.2 界面交互设计

#### 7.2.1 主界面布局
```
┌─────────────────────────────────────────────────────────┐
│  智狼 MindWolf                               [设置] [退出] │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─玩家卡片区────┐    ┌─游戏状态────┐    ┌─AI思考区───┐  │
│  │ 1号 张三(我)   │    │ 第2天白天    │    │ AI-2正在思考 │  │
│  │ 🙋‍♂️ 存活      │    │ 讨论阶段     │    │ [思考气泡]   │  │
│  │              │    │ 剩余: 2:30   │    │              │  │
│  │ 2号 AI-李四   │    │              │    │ AI-3: 我觉得 │  │
│  │ 🤖 存活      │    │ 存活: 6人    │    │ 5号很可疑... │  │
│  │              │    │ 死亡: 2人    │    │              │  │
│  │ 3号 AI-王五   │    │              │    │              │  │
│  │ 🤖 存活      │    │              │    │              │  │
│  └─────────────┘    └─────────────┘    └─────────────┘  │
│                                                         │
│  ┌─聊天记录区─────────────────────────────────────────┐  │
│  │ 系统: 昨晚死亡的是7号玩家                            │  │
│  │ 2号: 我是预言家，昨晚验了3号，是好人                 │  │
│  │ 3号: 我不信2号，我觉得他是悍跳                      │  │
│  │ 4号: 让我们理性分析一下...                          │  │
│  └─────────────────────────────────────────────────┘  │
│                                                         │
│  ┌─操作区─────────────────────────────────────────────┐  │
│  │ [🎤开始录音] [💬文字输入] [🗳️投票] [📊复盘] [⚙️设置]    │  │
│  │ 输入框: [请输入您的发言...]                [发送]     │  │
│  └─────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 7.3 AI个性化设置

```typescript
interface AIPersonality {
  id: string;
  name: string;
  description: string;
  traits: {
    aggressiveness: number;     // 攻击性 0-100
    logicalness: number;        // 逻辑性 0-100
    verbosity: number;          // 话痨程度 0-100
    deceptionSkill: number;     // 欺骗技巧 0-100
    trustfulness: number;       // 信任他人程度 0-100
  };
  speechPatterns: {
    vocabulary: 'formal' | 'casual' | 'gaming';
    lengthPreference: 'short' | 'medium' | 'long';
    emotionLevel: 'calm' | 'moderate' | 'intense';
  };
}

const predefinedPersonalities: AIPersonality[] = [
  {
    id: 'logical_analyst',
    name: '逻辑分析师',
    description: '冷静理性，善于逻辑推理，发言简洁有力',
    traits: { aggressiveness: 30, logicalness: 95, verbosity: 40, deceptionSkill: 60, trustfulness: 70 }
  },
  {
    id: 'aggressive_leader',
    name: '强势领袖',
    description: '主动带节奏，发言强势，喜欢指挥他人',
    traits: { aggressiveness: 90, logicalness: 75, verbosity: 80, deceptionSkill: 70, trustfulness: 40 }
  },
  {
    id: 'cunning_manipulator',
    name: '狡猾操控者',
    description: '善于欺骗和误导，话多且具有迷惑性',
    traits: { aggressiveness: 60, logicalness: 60, verbosity: 85, deceptionSkill: 95, trustfulness: 20 }
  }
];
```

## 8. 测试策略

### 8.1 单元测试

```
// 游戏引擎测试
describe('GameEngine', () => {
  test('should initialize game with correct roles', () => {
    const config = { playerCount: 8, roles: ['werewolf', 'seer', 'witch'] };
    const game = new GameEngine(config);
    expect(game.players).toHaveLength(8);
    expect(game.getRole('werewolf')).toBeDefined();
  });
  
  test('should handle vote correctly', () => {
    const game = createTestGame();
    game.vote('player1', 'player2');
    expect(game.getVoteCount('player2')).toBe(1);
  });
});

// AI推理测试
describe('AIReasoningEngine', () => {
  test('should update suspicion based on contradiction', () => {
    const engine = new ReasoningEngine();
    engine.updateSuspicion('player1', 'contradiction', 0.8);
    expect(engine.getSuspicion('player1')).toBeGreaterThan(0.5);
  });
});
```

### 8.2 集成测试

```
describe('Game Integration', () => {
  test('should complete a full game cycle', async () => {
    const gameManager = new GameManager();
    const game = await gameManager.startGame({
      humanPlayers: 1,
      aiPlayers: 7,
      roles: 'classic'
    });
    
    // 模拟完整游戏流程
    await gameManager.playNightPhase();
    await gameManager.playDayPhase();
    
    expect(game.isGameOver()).toBe(false);
  });
});
```

### 8.3 AI性能测试

```
class AIPerformanceTest:
    def test_ai_response_time(self):
        """测试AI响应时间"""
        ai_agent = AIAgent(personality='logical_analyst')
        start_time = time.time()
        
        response = ai_agent.generate_speech(mock_game_context)
        
        response_time = time.time() - start_time
        assert response_time < 3.0  # AI响应应在3秒内
    
    def test_ai_consistency(self):
        """测试AI行为一致性"""
        ai_agent = AIAgent(personality='logical_analyst')
        
        # 在相同情况下，AI应该做出类似的决策
        decisions = []
        for _ in range(10):
            decision = ai_agent.make_decision(identical_game_state)
            decisions.append(decision)
        
        # 检查决策的一致性
        assert self.calculate_consistency(decisions) > 0.8
```

## 9. 性能优化策略

### 9.1 前端性能优化

```
// 使用虚拟滚动优化聊天记录
const ChatHistory = defineComponent({
  setup() {
    const { list, containerProps, wrapperProps } = useVirtualList(
      chatMessages,
      {
        itemHeight: 60,
        overscan: 5,
      }
    );
    
    return { list, containerProps, wrapperProps };
  }
});

// 使用防抖优化语音输入
const useVoiceInput = () => {
  const processVoice = useDebounceFn(async (audioData: Blob) => {
    const text = await invoke('transcribe_audio', { audioData });
    return text;
  }, 500);
  
  return { processVoice };
};
```

### 9.2 Tauri后端实现

#### 9.2.1 Tauri 2.x 项目配置
```
# src-tauri/Cargo.toml - 最新版本
[package]
name = "mindwolf"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[build-dependencies]
tauri-build = { version = "2.0", features = ["isolation"] }

[dependencies]
tauri = { version = "2.0", features = ["isolation", "protocol-asset", "macos-private-api"] }
tauri-plugin-shell = "2.0"
tauri-plugin-fs = "2.0"
tauri-plugin-http = "2.0"
tauri-plugin-sql = { version = "2.0", features = ["sqlite"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.35", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "stream"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono"] }
thiserror = "1.0"
anyhow = "1.0"
log = "0.4"
env_logger = "0.10"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
aes-gcm = "0.10"
sha2 = "0.10"
base64 = "0.21"
machine-uid = "0.3"
async-trait = "0.1"
futures = "0.3"

# 新增：Tauri 2.x 专用插件
tauri-plugin-window-state = "2.0"
tauri-plugin-notification = "2.0"
tauri-plugin-dialog = "2.0"
tauri-plugin-clipboard-manager = "2.0"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

```
// src-tauri/tauri.conf.json - Tauri 2.x 配置
{
  "productName": "智狼 MindWolf",
  "version": "0.1.0",
  "identifier": "com.mindwolf.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "fullscreen": false,
        "resizable": true,
        "title": "智狼 MindWolf",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "center": true,
        "decorations": true,
        "transparent": false,
        "alwaysOnTop": false,
        "skipTaskbar": false,
        "theme": "auto"
      }
    ],
    "security": {
      "csp": "default-src 'self' data: blob: https://api.openai.com https://*.openai.azure.com; script-src 'self' 'unsafe-eval'; style-src 'self' 'unsafe-inline'"
    },
    "macOSPrivateApi": true
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "category": "Game",
    "shortDescription": "AI狼人杀游戏",
    "longDescription": "智狼是一款基于AI的狼人杀游戏，支持单人与AI对战，提供真实的游戏体验。",
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    },
    "macOS": {
      "entitlements": null,
      "exceptionDomain": "",
      "frameworks": [],
      "providerShortName": null,
      "signingIdentity": null
    },
    "linux": {
      "deb": {
        "depends": []
      }
    }
  },
  "plugins": {
    "shell": {
      "all": false,
      "execute": true,
      "sidecar": false,
      "open": false
    },
    "fs": {
      "all": false,
      "readFile": true,
      "writeFile": true,
      "readDir": false,
      "copyFile": false,
      "createDir": true,
      "removeDir": false,
      "removeFile": false,
      "renameFile": false,
      "exists": true
    },
    "http": {
      "all": false,
      "request": true
    },
    "sql": {
      "preload": ["sqlite:mindwolf.db"]
    }
  }
}
```

#### 9.2.2 现代化的Rust错误处理
```
// src-tauri/src/errors.rs - Tauri 2.x 错误处理
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("LLM服务错误: {0}")]
    LLMError(#[from] LLMError),
    
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("游戏状态错误: {message}")]
    GameStateError { message: String },
    
    #[error("配置错误: {message}")]
    ConfigError { message: String },
    
    #[error("网络错误: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
}

// 为Tauri命令实现自动错误转换
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

// 统一的Result类型
pub type AppResult<T> = Result<T, AppError>;
```

#### 9.2.3 LLM连接测试
```
// src-tauri/src/commands.rs
use crate::llm_client::LLMClient;
use tauri::State;

#[tauri::command]
pub async fn test_llm_connection(config: LLMConfig) -> Result<String, String> {
    let client = LLMClient::new(config);
    
    let test_messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "你好，请简单回复一下。".to_string(),
        }
    ];
    
    match client.chat_completion(test_messages).await {
        Ok(response) => {
            if !response.trim().is_empty() {
                Ok("连接测试成功".to_string())
            } else {
                Err("响应为空".to_string())
            }
        }
        Err(e) => Err(format!("连接失败: {}", e))
    }
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    // 保存设置到数据库
    let db = get_database_connection().await?;
    
    let settings_json = serde_json::to_string(&settings)
        .map_err(|e| format!("序列化失败: {}", e))?;
    
    sqlx::query!("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        "app_settings",
        settings_json
    )
    .execute(&db)
    .await
    .map_err(|e| format!("数据库错误: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn load_settings() -> Result<AppSettings, String> {
    let db = get_database_connection().await?;
    
    let row = sqlx::query!("SELECT value FROM settings WHERE key = ?", "app_settings")
        .fetch_optional(&db)
        .await
        .map_err(|e| format!("数据库错误: {}", e))?;
    
    match row {
        Some(record) => {
            serde_json::from_str(&record.value)
                .map_err(|e| format!("反序列化失败: {}", e))
        }
        None => Ok(AppSettings::default())
    }
}
```

#### 9.2.4 AI代理调用
```
#[tauri::command]
pub async fn generate_ai_speech(
    player_id: String,
    game_context: GameContext,
    personality: AIPersonality
) -> Result<String, String> {
    let llm_manager = get_llm_manager().await?;
    
    // 构建提示词
    let prompt = build_ai_speech_prompt(&game_context, &personality);
    
    // 调用LLM生成发言
    match llm_manager.generate_with_fallback(prompt).await {
        Ok(speech) => {
            // 记录AI思考过程
            log_ai_thought(&player_id, &game_context, &speech).await?;
            Ok(speech)
        }
        Err(e) => Err(format!("AI发言生成失败: {}", e))
    }
}

fn build_ai_speech_prompt(context: &GameContext, personality: &AIPersonality) -> String {
    format!("""
你是狼人杀游戏AI玩家，性格特点：{}
当前情况：
- 你的身份：{}
- 游戏阶段：第{}天{}
- 存活玩家：{}人
- 场上信息：{}

请根据以上情况生成一段符合你性格和身份的发言：
    """,
        personality.description,
        context.my_role,
        context.day,
        context.phase,
        context.alive_players.len(),
        context.situation_summary
    )
}
```

### 9.3 AI推理优化

```
class OptimizedAIAgent:
    def __init__(self):
        self.reasoning_cache = {}  # 缓存推理结果
        self.llm_pool = LLMPool(size=3)  # LLM连接池
    
    @lru_cache(maxsize=128)
    def analyze_game_state(self, game_state_hash: str) -> Analysis:
        """缓存游戏状态分析结果"""
        pass
    
    async def generate_speech_async(self, context: GameContext) -> str:
        """异步生成发言，避免阻塞"""
        return await self.llm_pool.generate(context)
```

### 9.4 内存管理

```
// Tauri后端内存优化
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct GameStateManager {
    games: Arc<RwLock<HashMap<String, GameState>>>,
    max_games: usize,
}

impl GameStateManager {
    pub async fn cleanup_old_games(&self) {
        let mut games = self.games.write().await;
        if games.len() > self.max_games {
            // 清理最旧的游戏状态
            let oldest_games: Vec<_> = games
                .iter()
                .sorted_by_key(|(_, state)| state.last_updated)
                .take(games.len() - self.max_games)
                .map(|(id, _)| id.clone())
                .collect();
            
            for game_id in oldest_games {
                games.remove(&game_id);
            }
        }
    }
}
```

## 10. OpenAI兼容API集成详解

### 10.1 支持的API提供商

智狼软件支持多种OpenAI兼容的API服务商，用户可以根据需求选择：

| 提供商 | 优势 | 推荐模型 | 配置示例 |
|--------|------|----------|----------|
| **OpenAI** | 官方服务，质量最高 | gpt-4, gpt-3.5-turbo | `https://api.openai.com` |
| **Azure OpenAI** | 企业级稳定性 | gpt-4, gpt-35-turbo | `https://{resource}.openai.azure.com` |
| **Anthropic Claude** | 安全性强，中文友好 | claude-3-opus, claude-3-sonnet | `https://api.anthropic.com` |
| **国产大模型** | 本土化，延迟低 | qwen-max, baichuan2 | 各厂商API地址 |
| **本地部署** | 数据安全，成本可控 | llama2-chinese, chatglm3 | `http://localhost:8000` |

### 10.2 API配置规范

#### 10.2.1 标准OpenAI格式
```
{
  "provider": "openai",
  "baseUrl": "https://api.openai.com",
  "apiKey": "sk-...",
  "model": "gpt-4",
  "maxTokens": 2000,
  "temperature": 0.7,
  "timeout": 30
}
```

#### 10.2.2 Azure OpenAI格式
```
{
  "provider": "azure",
  "baseUrl": "https://{resource}.openai.azure.com",
  "apiKey": "your-api-key",
  "model": "gpt-4", 
  "apiVersion": "2023-12-01-preview",
  "deployment": "gpt-4-deployment-name"
}
```

#### 10.2.3 自定义API格式
```
{
  "provider": "custom",
  "baseUrl": "https://your-api-endpoint.com",
  "apiKey": "your-api-key",
  "model": "your-model-name",
  "headers": {
    "Authorization": "Bearer {apiKey}",
    "Custom-Header": "value"
  }
}
```

### 10.3 API调用流程

```
sequenceDiagram
    participant UI as 用户界面
    participant Store as 设置存储
    participant Manager as LLM管理器
    participant Primary as 主要API
    participant Fallback as 备用API
    participant Local as 本地模型
    
    UI->>Store: 保存API配置
    Store->>Manager: 更新配置
    
    Note over Manager: AI需要生成回复
    Manager->>Primary: 调用主要API
    
    alt API调用成功
        Primary-->>Manager: 返回结果
        Manager-->>UI: 显示AI回复
    else 主要API失败
        Manager->>Fallback: 尝试备用API
        alt 备用API成功
            Fallback-->>Manager: 返回结果
            Manager-->>UI: 显示AI回复
        else 所有云端API失败
            Manager->>Local: 降级到本地模型
            Local-->>Manager: 本地推理结果
            Manager-->>UI: 显示AI回复(降级模式)
        end
    end
```

### 10.4 提示词工程

#### 10.4.1 狼人杀专用提示词模板
```
const PROMPT_TEMPLATES = {
  // 基础角色提示词
  VILLAGER_BASE: `你是狼人杀游戏中的村民，目标是找出所有狼人。你需要：
1. 仔细分析每个人的发言和行为
2. 寻找逻辑漏洞和可疑点
3. 与其他好人协作推理
4. 在投票时做出理性选择`,
  
  WEREWOLF_BASE: `你是狼人杀游戏中的狼人，目标是消灭所有好人而不被发现。你需要：
1. 伪装成好人身份
2. 误导其他玩家的推理方向
3. 在适当时机带节奏投票
4. 与狼人队友配合但不能太明显`,
  
  SEER_BASE: `你是狼人杀游戏中的预言家，拥有查验身份的能力。你需要：
1. 合理使用验人技能
2. 在适当时机跳出身份
3. 传递准确的验人信息
4. 引导好人阵营的推理方向`,
  
  // 发言场景提示词
  DEFENSE_SPEECH: `现在轮到你为自己辩护，你被其他玩家怀疑。请：
1. 冷静分析对你的指控
2. 提供有力的反驳证据
3. 指出真正的可疑对象
4. 展现你的逻辑推理能力`,
  
  ACCUSATION_SPEECH: `你需要指出你认为的狼人并说明理由。请：
1. 明确指出怀疑对象
2. 列举具体的可疑证据
3. 分析对方的行为动机
4. 说服其他玩家跟你站边`,
  
  // 性格化提示词
  AGGRESSIVE_PERSONALITY: `你的性格特点是强势且富有攻击性：
- 发言时语气坚定，态度强硬
- 主动质疑他人，不轻易妥协
- 在推理时表现出强烈的自信
- 倾向于主导讨论节奏`,
  
  CAUTIOUS_PERSONALITY: `你的性格特点是谨慎且理性：
- 发言前会仔细思考，措辞严谨
- 不轻易下结论，喜欢收集更多信息
- 在推理时会考虑多种可能性
- 倾向于跟随大众意见而非独断专行`
};
```

#### 10.4.2 动态提示词构建
```
class PromptBuilder {
  buildSpeechPrompt(context: GameContext, intent: SpeechIntent, personality: AIPersonality): string {
    const basePrompt = this.getBaseRolePrompt(context.myRole);
    const scenarioPrompt = this.getScenarioPrompt(intent.type);
    const personalityPrompt = this.getPersonalityPrompt(personality);
    const contextPrompt = this.buildContextPrompt(context);
    
    return `${basePrompt}

${personalityPrompt}

${scenarioPrompt}

${contextPrompt}

约束条件：
1. 发言长度控制在50-200字之间
2. 使用自然的口语化表达
3. 体现你的性格特点
4. 符合当前游戏情境
5. 不要暴露你的真实身份（如果是狼人）

请生成你的发言：`;
  }
  
  private buildContextPrompt(context: GameContext): string {
    return `当前游戏状况：
- 游戏进度：第${context.day}天${context.phase}
- 存活玩家：${context.alivePlayerCount}人
- 死亡玩家：${context.deadPlayers.map(p => `${p.name}(${p.deathReason})`).join(', ')}
- 已知信息：${context.publicInfo}
- 投票情况：${context.voteHistory}
- 最近发言：${context.recentSpeech.slice(-3).map(s => `${s.speaker}: ${s.content}`).join('\n')}`;
  }
}
```

### 10.5 API安全与隐私

#### 10.5.1 API密钥管理
```
// 密钥加密存储
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct SecureStorage {
    cipher: Aes256Gcm,
}

impl SecureStorage {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let key = Self::derive_key()?;
        let cipher = Aes256Gcm::new(&key);
        Ok(Self { cipher })
    }
    
    pub fn encrypt_api_key(&self, api_key: &str) -> Result<String, Box<dyn std::error::Error>> {
        let nonce = Nonce::from_slice(b"unique nonce"); // 实际应用中应使用随机nonce
        let ciphertext = self.cipher.encrypt(nonce, api_key.as_bytes())?;
        Ok(base64::encode(ciphertext))
    }
    
    pub fn decrypt_api_key(&self, encrypted: &str) -> Result<String, Box<dyn std::error::Error>> {
        let ciphertext = base64::decode(encrypted)?;
        let nonce = Nonce::from_slice(b"unique nonce");
        let plaintext = self.cipher.decrypt(nonce, ciphertext.as_slice())?;
        Ok(String::from_utf8(plaintext)?)
    }
    
    fn derive_key() -> Result<Key, Box<dyn std::error::Error>> {
        // 从系统信息派生密钥，实际应用中应使用更安全的方法
        let machine_id = machine_uid::get()?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(machine_id.as_bytes());
        hasher.update(b"mindwolf_secret_salt");
        let result = hasher.finalize();
        Ok(*Key::from_slice(&result))
    }
}
```

#### 10.5.2 请求内容过滤
```
// 敏感信息过滤
class ContentFilter {
  private sensitivePatterns = [
    /sk-[a-zA-Z0-9]{48}/, // OpenAI API密钥
    /\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b/, // 信用卡号
    /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/, // 邮箱地址
    /\b\d{11}\b/, // 手机号
  ];
  
  filterGameContent(content: string): string {
    let filtered = content;
    
    // 移除敏感信息
    this.sensitivePatterns.forEach(pattern => {
      filtered = filtered.replace(pattern, '[已过滤]');
    });
    
    // 限制内容长度
    if (filtered.length > 1000) {
      filtered = filtered.substring(0, 1000) + '...';
    }
    
    return filtered;
  }
  
  validateApiKey(apiKey: string): boolean {
    // 验证API密钥格式
    const patterns = {
      openai: /^sk-[a-zA-Z0-9]{48}$/,
      anthropic: /^sk-ant-[a-zA-Z0-9\-_]{95}$/,
      custom: /^[a-zA-Z0-9\-_]{10,}$/
    };
    
    return Object.values(patterns).some(pattern => pattern.test(apiKey));
  }
}
```

### 10.6 API成本控制

#### 10.6.1 使用量监控
```
interface APIUsageStats {
  dailyTokens: number;
  dailyRequests: number;
  monthlyTokens: number;
  monthlyCost: number;
  lastResetDate: Date;
}

class APIUsageTracker {
  private stats: APIUsageStats;
  
  constructor() {
    this.loadStats();
  }
  
  async trackRequest(tokens: number, cost: number) {
    this.stats.dailyTokens += tokens;
    this.stats.dailyRequests += 1;
    this.stats.monthlyTokens += tokens;
    this.stats.monthlyCost += cost;
    
    await this.saveStats();
    
    // 检查是否超过限制
    this.checkLimits();
  }
  
  private checkLimits() {
    const limits = {
      dailyTokens: 100000,
      dailyRequests: 1000,
      monthlyCost: 50 // 美元
    };
    
    if (this.stats.dailyTokens > limits.dailyTokens) {
      throw new Error('已达到每日Token使用限制');
    }
    
    if (this.stats.monthlyCost > limits.monthlyCost) {
      throw new Error('已达到每月费用限制');
    }
  }
  
  getDashboardData() {
    return {
      today: {
        tokens: this.stats.dailyTokens,
        requests: this.stats.dailyRequests,
        estimatedCost: this.estimateCost(this.stats.dailyTokens)
      },
      month: {
        tokens: this.stats.monthlyTokens,
        cost: this.stats.monthlyCost
      }
    };
  }
}
```

