use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// 游戏记录模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GameRecord {
    pub id: String,
    pub config: String, // JSON格式的游戏配置
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub winner: Option<String>,
    pub player_count: i32,
    pub duration_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// 玩家记录模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlayerRecord {
    pub id: String,
    pub game_id: String,
    pub player_name: String,
    pub role_type: String,
    pub faction: String,
    pub is_ai: bool,
    pub is_winner: bool,
    pub elimination_day: Option<i32>,
    pub final_votes: i32,
}

/// 发言记录模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SpeechRecord {
    pub id: String,
    pub game_id: String,
    pub player_id: String,
    pub content: String,
    pub day: i32,
    pub phase: String,
    pub timestamp: DateTime<Utc>,
    pub analysis_result: Option<String>, // JSON格式的分析结果
}

/// 投票记录模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VoteRecord {
    pub id: String,
    pub game_id: String,
    pub voter_id: String,
    pub target_id: String,
    pub day: i32,
    pub vote_round: i32,
    pub timestamp: DateTime<Utc>,
}

/// 夜晚行动记录模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NightActionRecord {
    pub id: String,
    pub game_id: String,
    pub player_id: String,
    pub action_type: String,
    pub target_id: Option<String>,
    pub night: i32,
    pub result: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// AI分析记录模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AIAnalysisRecord {
    pub id: String,
    pub game_id: String,
    pub player_id: String,
    pub analysis_type: String,
    pub analysis_data: String, // JSON格式的分析数据
    pub day: i32,
    pub timestamp: DateTime<Utc>,
}

/// 游戏详情（包含所有相关记录）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDetails {
    pub game: GameRecord,
    pub players: Vec<PlayerRecord>,
    pub speeches: Vec<SpeechRecord>,
    pub votes: Vec<VoteRecord>,
    pub night_actions: Vec<NightActionRecord>,
    pub ai_analyses: Vec<AIAnalysisRecord>,
}

/// 游戏统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStatistics {
    pub total_games: u32,
    pub total_speeches: u32,
    pub total_votes: u32,
    pub average_game_duration: f32, // 分钟
    pub win_rate_by_faction: std::collections::HashMap<String, f32>,
    pub most_played_roles: Vec<(String, u32)>,
}

/// 玩家统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatistics {
    pub player_name: String,
    pub total_games: u32,
    pub wins: u32,
    pub win_rate: f32,
    pub favorite_roles: Vec<(String, u32)>,
    pub average_speeches_per_game: f32,
    pub survival_rate: f32,
}
