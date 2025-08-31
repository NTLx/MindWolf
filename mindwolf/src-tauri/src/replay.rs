use crate::error::Result;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 游戏复盘数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameReplay {
    pub game_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub players: Vec<Player>,
    pub game_events: Vec<GameEvent>,
    pub ai_decisions: Vec<AIDecision>,
    pub game_result: Option<GameResult>,
    pub game_config: GameConfig,
    pub analysis: Option<GameAnalysis>,
}

/// 游戏事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub id: String,
    pub event_type: GameEventType,
    pub timestamp: DateTime<Utc>,
    pub round: u32,
    pub phase: GamePhase,
    pub player_id: Option<String>,
    pub target_id: Option<String>,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 游戏事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEventType {
    /// 游戏开始
    GameStart,
    /// 游戏结束
    GameEnd,
    /// 角色分配
    RoleAssignment,
    /// 发言
    Speech,
    /// 投票
    Vote,
    /// 技能使用
    SkillUse,
    /// 阶段切换
    PhaseChange,
    /// 玩家死亡
    PlayerDeath,
    /// 警长竞选
    SheriffElection,
    /// 遗言
    LastWords,
    /// 系统公告
    SystemAnnouncement,
}

/// AI决策记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIDecision {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub player_id: String,
    pub decision_type: DecisionType,
    pub context: DecisionContext,
    pub reasoning: String,
    pub confidence: f32,
    pub execution_time_ms: u64,
    pub alternatives: Vec<AlternativeDecision>,
}

/// 决策类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {
    Speech,
    Vote,
    SkillTarget,
    SheriffVote,
    Strategy,
}

/// 决策上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub round: u32,
    pub phase: GamePhase,
    pub alive_players: Vec<String>,
    pub known_roles: HashMap<String, Role>,
    pub voting_history: Vec<VoteRecord>,
    pub speech_history: Vec<SpeechRecord>,
    pub game_state: GameStateSnapshot,
}

/// 备选决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeDecision {
    pub option: String,
    pub score: f32,
    pub reasoning: String,
}

/// 游戏分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnalysis {
    pub winner_analysis: WinnerAnalysis,
    pub player_performance: HashMap<String, PlayerPerformance>,
    pub turning_points: Vec<TurningPoint>,
    pub strategic_insights: Vec<StrategicInsight>,
    pub ai_performance_metrics: AIPerformanceMetrics,
    pub game_statistics: GameStatistics,
}

/// 获胜分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinnerAnalysis {
    pub winning_faction: Faction,
    pub winning_reason: String,
    pub key_factors: Vec<String>,
    pub critical_decisions: Vec<String>,
}

/// 玩家表现分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPerformance {
    pub player_id: String,
    pub survival_rounds: u32,
    pub speech_quality: f32,
    pub logical_consistency: f32,
    pub deception_ability: f32,
    pub voting_accuracy: f32,
    pub influence_score: f32,
    pub overall_rating: f32,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

/// 转折点分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurningPoint {
    pub timestamp: DateTime<Utc>,
    pub round: u32,
    pub phase: GamePhase,
    pub event_id: String,
    pub description: String,
    pub impact_score: f32,
    pub affected_players: Vec<String>,
    pub faction_advantage_shift: HashMap<Faction, f32>,
}

/// 策略洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicInsight {
    pub category: InsightCategory,
    pub title: String,
    pub description: String,
    pub supporting_evidence: Vec<String>,
    pub learning_value: f32,
}

/// 洞察类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightCategory {
    Deception,
    Reasoning,
    Teamwork,
    Voting,
    Communication,
    RolePlay,
}

/// AI性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPerformanceMetrics {
    pub average_response_time: f32,
    pub decision_confidence: f32,
    pub strategy_consistency: f32,
    pub role_playing_accuracy: f32,
    pub language_fluency: f32,
    pub logical_reasoning: f32,
    pub adaptability: f32,
}

/// 游戏统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStatistics {
    pub total_rounds: u32,
    pub total_speeches: u32,
    pub total_votes: u32,
    pub average_speech_length: f32,
    pub voting_patterns: HashMap<String, u32>,
    pub role_distribution: HashMap<Role, u32>,
    pub faction_balance: HashMap<Faction, f32>,
}

/// 复盘系统
pub struct ReplaySystem {
    replays: HashMap<String, GameReplay>,
    analyzer: GameAnalyzer,
}

impl ReplaySystem {
    pub fn new() -> Self {
        Self {
            replays: HashMap::new(),
            analyzer: GameAnalyzer::new(),
        }
    }

    /// 开始记录游戏
    pub fn start_recording(&mut self, game_id: String, config: GameConfig, players: Vec<Player>) -> Result<()> {
        let replay = GameReplay {
            game_id: game_id.clone(),
            start_time: Utc::now(),
            end_time: None,
            players,
            game_events: Vec::new(),
            ai_decisions: Vec::new(),
            game_result: None,
            game_config: config,
            analysis: None,
        };

        self.replays.insert(game_id, replay);
        log::info("开始记录游戏复盘数据");
        Ok(())
    }

    /// 记录游戏事件
    pub fn record_event(&mut self, game_id: &str, event: GameEvent) -> Result<()> {
        if let Some(replay) = self.replays.get_mut(game_id) {
            replay.game_events.push(event);
        }
        Ok(())
    }

    /// 记录AI决策
    pub fn record_ai_decision(&mut self, game_id: &str, decision: AIDecision) -> Result<()> {
        if let Some(replay) = self.replays.get_mut(game_id) {
            replay.ai_decisions.push(decision);
        }
        Ok(())
    }

    /// 结束游戏记录并分析
    pub async fn finish_recording(&mut self, game_id: &str, result: GameResult) -> Result<()> {
        if let Some(replay) = self.replays.get_mut(game_id) {
            replay.end_time = Some(Utc::now());
            replay.game_result = Some(result);
            
            // 执行游戏分析
            replay.analysis = Some(self.analyzer.analyze_game(replay).await?);
            
            log::info(&format!("游戏 {} 复盘记录完成", game_id));
        }
        Ok(())
    }

    /// 获取游戏复盘
    pub fn get_replay(&self, game_id: &str) -> Option<&GameReplay> {
        self.replays.get(game_id)
    }

    /// 获取所有复盘列表
    pub fn get_replay_list(&self) -> Vec<&GameReplay> {
        self.replays.values().collect()
    }

    /// 搜索复盘
    pub fn search_replays(&self, query: &ReplayQuery) -> Vec<&GameReplay> {
        self.replays
            .values()
            .filter(|replay| self.matches_query(replay, query))
            .collect()
    }

    /// 导出复盘数据
    pub fn export_replay(&self, game_id: &str, format: ExportFormat) -> Result<Vec<u8>> {
        if let Some(replay) = self.replays.get(game_id) {
            match format {
                ExportFormat::Json => {
                    let json = serde_json::to_string_pretty(replay)?;
                    Ok(json.into_bytes())
                }
                ExportFormat::Csv => {
                    // 实现CSV导出
                    self.export_to_csv(replay)
                }
                ExportFormat::Html => {
                    // 实现HTML报告导出
                    self.export_to_html(replay)
                }
            }
        } else {
            Err(crate::error::AppError::NotFound(format!("游戏复盘不存在: {}", game_id)).into())
        }
    }

    /// 删除复盘
    pub fn delete_replay(&mut self, game_id: &str) -> Result<()> {
        self.replays.remove(game_id);
        log::info(&format!("已删除游戏复盘: {}", game_id));
        Ok(())
    }

    /// 生成复盘统计报告
    pub fn generate_statistics(&self, filter: Option<&ReplayQuery>) -> ReplayStatistics {
        let replays: Vec<_> = if let Some(query) = filter {
            self.search_replays(query)
        } else {
            self.get_replay_list()
        };

        self.analyzer.generate_statistics(&replays)
    }

    // 私有方法
    fn matches_query(&self, replay: &GameReplay, query: &ReplayQuery) -> bool {
        // 时间范围过滤
        if let Some(start) = query.start_time {
            if replay.start_time < start {
                return false;
            }
        }
        
        if let Some(end) = query.end_time {
            if replay.start_time > end {
                return false;
            }
        }

        // 玩家过滤
        if let Some(player_id) = &query.player_id {
            if !replay.players.iter().any(|p| &p.id == player_id) {
                return false;
            }
        }

        // 获胜方过滤
        if let Some(winner) = &query.winner_faction {
            if let Some(result) = &replay.game_result {
                if &result.winner != winner {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn export_to_csv(&self, replay: &GameReplay) -> Result<Vec<u8>> {
        let mut csv_content = String::new();
        
        // CSV头部
        csv_content.push_str("Event Type,Timestamp,Round,Phase,Player,Target,Content\n");
        
        // 导出事件数据
        for event in &replay.game_events {
            csv_content.push_str(&format!(
                "{:?},{},{},{:?},{},{},{}\n",
                event.event_type,
                event.timestamp.format("%Y-%m-%d %H:%M:%S"),
                event.round,
                event.phase,
                event.player_id.as_deref().unwrap_or(""),
                event.target_id.as_deref().unwrap_or(""),
                event.content.replace(',', ";").replace('\n', " ")
            ));
        }
        
        Ok(csv_content.into_bytes())
    }

    fn export_to_html(&self, replay: &GameReplay) -> Result<Vec<u8>> {
        let mut html = String::new();
        
        html.push_str("<!DOCTYPE html><html><head><title>游戏复盘报告</title>");
        html.push_str("<style>body{font-family:Arial,sans-serif;margin:20px;}table{border-collapse:collapse;width:100%;}th,td{border:1px solid #ddd;padding:8px;text-align:left;}th{background-color:#f2f2f2;}</style>");
        html.push_str("</head><body>");
        
        html.push_str(&format!("<h1>游戏复盘报告 - {}</h1>", replay.game_id));
        html.push_str(&format!("<p>开始时间: {}</p>", replay.start_time.format("%Y-%m-%d %H:%M:%S")));
        
        if let Some(end_time) = replay.end_time {
            html.push_str(&format!("<p>结束时间: {}</p>", end_time.format("%Y-%m-%d %H:%M:%S")));
        }

        // 玩家信息
        html.push_str("<h2>玩家信息</h2><table><tr><th>玩家</th><th>角色</th><th>阵营</th></tr>");
        for player in &replay.players {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{:?}</td><td>{:?}</td></tr>",
                player.name, player.role, player.role.get_faction()
            ));
        }
        html.push_str("</table>");

        // 游戏结果
        if let Some(result) = &replay.game_result {
            html.push_str(&format!("<h2>游戏结果</h2><p>获胜方: {:?}</p>", result.winner));
        }

        html.push_str("</body></html>");
        
        Ok(html.into_bytes())
    }
}

/// 复盘查询条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub player_id: Option<String>,
    pub winner_faction: Option<Faction>,
    pub min_rounds: Option<u32>,
    pub max_rounds: Option<u32>,
}

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Html,
}

/// 复盘统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayStatistics {
    pub total_games: u32,
    pub faction_win_rates: HashMap<Faction, f32>,
    pub average_game_duration: f32,
    pub average_rounds: f32,
    pub most_active_players: Vec<(String, u32)>,
    pub role_performance: HashMap<Role, RolePerformance>,
}

/// 角色表现统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePerformance {
    pub win_rate: f32,
    pub average_survival: f32,
    pub impact_score: f32,
}

/// 游戏分析器
pub struct GameAnalyzer;

impl GameAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// 分析游戏
    pub async fn analyze_game(&self, replay: &GameReplay) -> Result<GameAnalysis> {
        log::info("开始分析游戏数据...");

        let winner_analysis = self.analyze_winner(replay).await?;
        let player_performance = self.analyze_player_performance(replay).await?;
        let turning_points = self.identify_turning_points(replay).await?;
        let strategic_insights = self.extract_strategic_insights(replay).await?;
        let ai_performance_metrics = self.calculate_ai_metrics(replay).await?;
        let game_statistics = self.calculate_game_statistics(replay).await?;

        Ok(GameAnalysis {
            winner_analysis,
            player_performance,
            turning_points,
            strategic_insights,
            ai_performance_metrics,
            game_statistics,
        })
    }

    /// 分析获胜原因
    async fn analyze_winner(&self, replay: &GameReplay) -> Result<WinnerAnalysis> {
        // 实现获胜分析逻辑
        Ok(WinnerAnalysis {
            winning_faction: replay.game_result.as_ref().unwrap().winner.clone(),
            winning_reason: "详细分析待实现".to_string(),
            key_factors: vec!["因素1".to_string(), "因素2".to_string()],
            critical_decisions: vec!["关键决策1".to_string()],
        })
    }

    /// 分析玩家表现
    async fn analyze_player_performance(&self, replay: &GameReplay) -> Result<HashMap<String, PlayerPerformance>> {
        let mut performance = HashMap::new();
        
        for player in &replay.players {
            // 计算各项指标
            let perf = PlayerPerformance {
                player_id: player.id.clone(),
                survival_rounds: self.calculate_survival_rounds(&player.id, replay),
                speech_quality: self.calculate_speech_quality(&player.id, replay),
                logical_consistency: self.calculate_logical_consistency(&player.id, replay),
                deception_ability: self.calculate_deception_ability(&player.id, replay),
                voting_accuracy: self.calculate_voting_accuracy(&player.id, replay),
                influence_score: self.calculate_influence_score(&player.id, replay),
                overall_rating: 0.0, // 将在后面计算
                strengths: vec![],
                weaknesses: vec![],
            };
            
            performance.insert(player.id.clone(), perf);
        }
        
        Ok(performance)
    }

    /// 识别转折点
    async fn identify_turning_points(&self, replay: &GameReplay) -> Result<Vec<TurningPoint>> {
        // 实现转折点识别逻辑
        Ok(vec![])
    }

    /// 提取策略洞察
    async fn extract_strategic_insights(&self, replay: &GameReplay) -> Result<Vec<StrategicInsight>> {
        // 实现策略洞察提取逻辑
        Ok(vec![])
    }

    /// 计算AI性能指标
    async fn calculate_ai_metrics(&self, replay: &GameReplay) -> Result<AIPerformanceMetrics> {
        let decisions = &replay.ai_decisions;
        
        let average_response_time = if !decisions.is_empty() {
            decisions.iter().map(|d| d.execution_time_ms as f32).sum::<f32>() / decisions.len() as f32
        } else {
            0.0
        };

        let decision_confidence = if !decisions.is_empty() {
            decisions.iter().map(|d| d.confidence).sum::<f32>() / decisions.len() as f32
        } else {
            0.0
        };

        Ok(AIPerformanceMetrics {
            average_response_time,
            decision_confidence,
            strategy_consistency: 0.8, // 待实现
            role_playing_accuracy: 0.75, // 待实现
            language_fluency: 0.85, // 待实现
            logical_reasoning: 0.8, // 待实现
            adaptability: 0.7, // 待实现
        })
    }

    /// 计算游戏统计数据
    async fn calculate_game_statistics(&self, replay: &GameReplay) -> Result<GameStatistics> {
        let events = &replay.game_events;
        
        let total_rounds = events.iter()
            .map(|e| e.round)
            .max()
            .unwrap_or(0);

        let total_speeches = events.iter()
            .filter(|e| matches!(e.event_type, GameEventType::Speech))
            .count() as u32;

        let total_votes = events.iter()
            .filter(|e| matches!(e.event_type, GameEventType::Vote))
            .count() as u32;

        Ok(GameStatistics {
            total_rounds,
            total_speeches,
            total_votes,
            average_speech_length: 0.0, // 待实现
            voting_patterns: HashMap::new(), // 待实现
            role_distribution: HashMap::new(), // 待实现
            faction_balance: HashMap::new(), // 待实现
        })
    }

    /// 生成统计报告
    pub fn generate_statistics(&self, replays: &[&GameReplay]) -> ReplayStatistics {
        let total_games = replays.len() as u32;
        
        // 计算阵营胜率
        let mut faction_wins = HashMap::new();
        for replay in replays {
            if let Some(result) = &replay.game_result {
                *faction_wins.entry(result.winner.clone()).or_insert(0) += 1;
            }
        }
        
        let faction_win_rates: HashMap<Faction, f32> = faction_wins.iter()
            .map(|(faction, wins)| (faction.clone(), *wins as f32 / total_games as f32))
            .collect();

        ReplayStatistics {
            total_games,
            faction_win_rates,
            average_game_duration: 0.0, // 待实现
            average_rounds: 0.0, // 待实现
            most_active_players: vec![], // 待实现
            role_performance: HashMap::new(), // 待实现
        }
    }

    // 辅助计算方法
    fn calculate_survival_rounds(&self, _player_id: &str, _replay: &GameReplay) -> u32 {
        // 实现生存轮数计算
        0
    }

    fn calculate_speech_quality(&self, _player_id: &str, _replay: &GameReplay) -> f32 {
        // 实现发言质量计算
        0.0
    }

    fn calculate_logical_consistency(&self, _player_id: &str, _replay: &GameReplay) -> f32 {
        // 实现逻辑一致性计算
        0.0
    }

    fn calculate_deception_ability(&self, _player_id: &str, _replay: &GameReplay) -> f32 {
        // 实现欺骗能力计算
        0.0
    }

    fn calculate_voting_accuracy(&self, _player_id: &str, _replay: &GameReplay) -> f32 {
        // 实现投票准确性计算
        0.0
    }

    fn calculate_influence_score(&self, _player_id: &str, _replay: &GameReplay) -> f32 {
        // 实现影响力分数计算
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_system_creation() {
        let replay_system = ReplaySystem::new();
        assert_eq!(replay_system.replays.len(), 0);
    }

    #[tokio::test]
    async fn test_game_analyzer() {
        let analyzer = GameAnalyzer::new();
        // 创建测试数据
        let replay = GameReplay {
            game_id: "test".to_string(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            players: vec![],
            game_events: vec![],
            ai_decisions: vec![],
            game_result: Some(GameResult {
                winner: Faction::Village,
                reason: "测试".to_string(),
                survivors: vec![],
            }),
            game_config: GameConfig::default(),
            analysis: None,
        };

        let analysis = analyzer.analyze_game(&replay).await.unwrap();
        assert_eq!(analysis.winner_analysis.winning_faction, Faction::Village);
    }
}