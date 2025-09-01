use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};


/// 角色信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Role {
    pub role_type: RoleType,
    pub faction: Faction,
    pub description: String,
    pub can_vote: bool,
    pub has_night_action: bool,
}

/// 角色类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RoleType {
    Werewolf,
    Villager,
    Seer,
    Witch,
    Hunter,
    Guard,
}

/// 阵营枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Faction {
    Werewolf,
    Villager,
}

/// 游戏阶段枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GamePhase {
    Preparation,
    Night,
    DayDiscussion,
    Voting,
    LastWords,
    GameOver,
}

/// 游戏状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub phase: GamePhase,
    pub day: u32,
    pub players: Vec<Player>,
    pub dead_players: Vec<Player>,
    pub votes: Vec<VoteRecord>,
    pub game_config: GameConfig,
    pub winner: Option<Faction>,
    pub current_speaker: Option<String>,
    pub time_remaining: Option<u32>,
}

/// 投票记录
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VoteRecord {
    pub voter: String,
    pub target: String,
    pub timestamp: DateTime<Utc>,
}

/// 游戏配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub total_players: u8,
    pub role_distribution: HashMap<RoleType, u8>,
    pub discussion_time: u32,
    pub voting_time: u32,
    pub enable_voice: bool,
}

/// AI性格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPersonality {
    pub id: String,
    pub name: String,
    pub description: String,
    pub traits: PersonalityTraits,
}

/// 性格特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    pub aggressiveness: f32, // 0.0-1.0
    pub logic: f32,         // 0.0-1.0
    pub deception: f32,     // 0.0-1.0
    pub trustfulness: f32,  // 0.0-1.0
}

/// 发言意图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechIntent {
    pub intent_type: SpeechType,
    pub target: Option<String>,
    pub content: String,
    pub confidence: f32,
}

/// 发言类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeechType {
    Accusation,
    Defense,
    Information,
    Strategy,
    Vote,
}

/// LLM配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout: u64,
}

/// LLM提供商
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI,
    Anthropic,
    Azure,
    Custom,
}

/// 游戏动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAction {
    pub action_type: String,
    pub player: String,
    pub target: Option<String>,
    pub data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

/// 夜晚动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightAction {
    pub player: String,
    pub action: NightActionType,
    pub target: Option<String>,
}

/// 夜晚动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NightActionType {
    Kill,
    Check,
    Heal,
    Protect,
    Poison,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Human,
    AI,
    System,
}

/// 玩家信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub faction: Faction,
    pub is_alive: bool,
    pub is_ai: bool,
    pub personality: Option<AIPersonality>,
}

/// 夜晚结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NightResult {
    PlayerKilled(String),
    PlayerSaved,
    NoKill,
}

/// 记忆信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechMemory {
    pub speaker: String,
    pub content: String,
    pub day: u32,
    pub phase: GamePhase,
    pub my_reaction: String,
}

/// AI分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisReport {
    pub agent_id: String,
    pub current_strategy: String,
    pub trust_rankings: Vec<String>,
    pub suspicion_rankings: Vec<String>,
    pub reasoning_summary: String,
    pub memory_highlights: Vec<String>,
}

/// 游戏结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    pub winner: Faction,
    pub game_duration: u32,
    pub total_votes: u32,
    pub players_killed: Vec<String>,
}

/// 语音记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechRecord {
    pub speaker: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub phase: GamePhase,
    pub day: u32,
}

/// 转折点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurningPoint {
    pub day: u32,
    pub phase: GamePhase,
    pub description: String,
    pub impact_score: f32,
}

/// 策略洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicInsight {
    pub insight_type: String,
    pub description: String,
    pub confidence: f32,
}

/// 游戏状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub day: u32,
    pub phase: GamePhase,
    pub alive_players: Vec<String>,
    pub votes: Vec<VoteRecord>,
    pub timestamp: DateTime<Utc>,
}

impl Role {
    /// 获取角色所属阵营
    pub fn get_faction(&self) -> &Faction {
        &self.faction
    }
}

/// 发言分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechAnalysis {
    pub intent: SpeechIntent,
    pub emotion: String,
    pub credibility: f32,
    pub key_information: Vec<String>,
    pub targets_mentioned: Vec<String>,
}

/// 推理报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningReport {
    pub summary: String,
    pub confidence: f32,
    pub key_findings: Vec<String>,
}

/// 发言策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechStrategy {
    pub strategy_type: StrategyType,
    pub target_players: Vec<String>,
    pub key_points: Vec<String>,
}

/// 策略类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    Aggressive,
    Defensive,
    Neutral,
    Deceptive,
    Logical,
}

/// 夜晚行动记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightActionRecord {
    pub night: u32,
    pub action: NightActionType,
    pub player: String,
    pub target: Option<String>,
    pub result: Option<String>,
}

/// 夜晚行动记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightActionMemory {
    pub night: u32,
    pub my_action: Option<NightAction>,
    pub observed_results: Vec<String>,
}