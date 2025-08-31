use crate::database::models::*;
use crate::error::{AppError, AppResult};
use crate::types::*;
use sqlx::SqlitePool;
use chrono::Utc;
use log::{info, debug};
use uuid::Uuid;

/// 游戏记录仓库
pub struct GameRepository {
    pool: SqlitePool,
}

impl GameRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    /// 创建新游戏记录
    pub async fn create_game(&self, game_state: &GameState) -> AppResult<String> {
        let game_id = Uuid::new_v4().to_string();
        let config_json = serde_json::to_string(&game_state.game_config)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        
        sqlx::query(
            r#\"
            INSERT INTO game_records (id, config, start_time, player_count)
            VALUES (?, ?, ?, ?)
            \"#
        )
        .bind(&game_id)
        .bind(&config_json)
        .bind(Utc::now())
        .bind(game_state.players.len() as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"创建游戏记录失败: {}\", e)))?;
        
        // 创建玩家记录
        for player in &game_state.players {
            self.create_player_record(&game_id, player).await?;
        }
        
        info!(\"创建游戏记录: {}\", game_id);
        Ok(game_id)
    }
    
    /// 更新游戏结束信息
    pub async fn finish_game(&self, game_id: &str, winner: Option<&Faction>, duration_seconds: i32) -> AppResult<()> {
        let winner_str = winner.map(|f| match f {
            Faction::Werewolf => \"werewolf\",
            Faction::Villager => \"villager\",
        });
        
        sqlx::query(
            \"UPDATE game_records SET end_time = ?, winner = ?, duration_seconds = ? WHERE id = ?\"
        )
        .bind(Utc::now())
        .bind(winner_str)
        .bind(duration_seconds)
        .bind(game_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"更新游戏结束信息失败: {}\", e)))?;
        
        info!(\"游戏 {} 已结束，获胜方: {:?}\", game_id, winner);
        Ok(())
    }
    
    /// 记录发言
    pub async fn record_speech(&self, game_id: &str, speech: &ChatMessage, day: u32, phase: &GamePhase) -> AppResult<()> {
        let speech_id = Uuid::new_v4().to_string();
        let phase_str = self.phase_to_string(phase);
        
        sqlx::query(
            r#\"
            INSERT INTO speech_records (id, game_id, player_id, content, day, phase, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            \"#
        )
        .bind(&speech_id)
        .bind(game_id)
        .bind(&speech.sender)
        .bind(&speech.content)
        .bind(day as i32)
        .bind(&phase_str)
        .bind(speech.timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"记录发言失败: {}\", e)))?;
        
        debug!(\"记录发言: {} - {}\", speech.sender, speech.content);
        Ok(())
    }
    
    /// 记录投票
    pub async fn record_vote(&self, game_id: &str, vote: &VoteRecord, day: u32, round: u32) -> AppResult<()> {
        let vote_id = Uuid::new_v4().to_string();
        
        sqlx::query(
            r#\"
            INSERT INTO vote_records (id, game_id, voter_id, target_id, day, vote_round, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            \"#
        )
        .bind(&vote_id)
        .bind(game_id)
        .bind(&vote.voter)
        .bind(&vote.target)
        .bind(day as i32)
        .bind(round as i32)
        .bind(vote.timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"记录投票失败: {}\", e)))?;
        
        debug!(\"记录投票: {} -> {}\", vote.voter, vote.target);
        Ok(())
    }
    
    /// 记录夜晚行动
    pub async fn record_night_action(&self, game_id: &str, action: &NightAction, night: u32, result: Option<&str>) -> AppResult<()> {
        let action_id = Uuid::new_v4().to_string();
        let action_type = self.night_action_to_string(&action.action);
        
        sqlx::query(
            r#\"
            INSERT INTO night_action_records (id, game_id, player_id, action_type, target_id, night, result, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            \"#
        )
        .bind(&action_id)
        .bind(game_id)
        .bind(&action.player)
        .bind(&action_type)
        .bind(&action.target)
        .bind(night as i32)
        .bind(result)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"记录夜晚行动失败: {}\", e)))?;
        
        debug!(\"记录夜晚行动: {} - {:?}\", action.player, action.action);
        Ok(())
    }
    
    /// 记录AI分析
    pub async fn record_ai_analysis(&self, game_id: &str, player_id: &str, analysis_type: &str, analysis_data: &str, day: u32) -> AppResult<()> {
        let analysis_id = Uuid::new_v4().to_string();
        
        sqlx::query(
            r#\"
            INSERT INTO ai_analysis_records (id, game_id, player_id, analysis_type, analysis_data, day, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            \"#
        )
        .bind(&analysis_id)
        .bind(game_id)
        .bind(player_id)
        .bind(analysis_type)
        .bind(analysis_data)
        .bind(day as i32)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"记录AI分析失败: {}\", e)))?;
        
        debug!(\"记录AI分析: {} - {}\", player_id, analysis_type);
        Ok(())
    }
    
    /// 获取游戏详情
    pub async fn get_game_details(&self, game_id: &str) -> AppResult<GameDetails> {
        // 获取游戏基本信息
        let game = sqlx::query_as::<_, GameRecord>(
            \"SELECT * FROM game_records WHERE id = ?\"
        )
        .bind(game_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"获取游戏记录失败: {}\", e)))?;
        
        // 获取玩家记录
        let players = sqlx::query_as::<_, PlayerRecord>(
            \"SELECT * FROM player_records WHERE game_id = ? ORDER BY id\"
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"获取玩家记录失败: {}\", e)))?;
        
        // 获取发言记录
        let speeches = sqlx::query_as::<_, SpeechRecord>(
            \"SELECT * FROM speech_records WHERE game_id = ? ORDER BY timestamp\"
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"获取发言记录失败: {}\", e)))?;
        
        // 获取投票记录
        let votes = sqlx::query_as::<_, VoteRecord>(
            \"SELECT * FROM vote_records WHERE game_id = ? ORDER BY timestamp\"
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"获取投票记录失败: {}\", e)))?;
        
        // 获取夜晚行动记录
        let night_actions = sqlx::query_as::<_, NightActionRecord>(
            \"SELECT * FROM night_action_records WHERE game_id = ? ORDER BY timestamp\"
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"获取夜晚行动记录失败: {}\", e)))?;
        
        // 获取AI分析记录
        let ai_analyses = sqlx::query_as::<_, AIAnalysisRecord>(
            \"SELECT * FROM ai_analysis_records WHERE game_id = ? ORDER BY timestamp\"
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"获取AI分析记录失败: {}\", e)))?;
        
        Ok(GameDetails {
            game,
            players,
            speeches,
            votes,
            night_actions,
            ai_analyses,
        })
    }
    
    /// 获取最近的游戏列表
    pub async fn get_recent_games(&self, limit: u32) -> AppResult<Vec<GameRecord>> {
        let games = sqlx::query_as::<_, GameRecord>(
            \"SELECT * FROM game_records ORDER BY start_time DESC LIMIT ?\"
        )
        .bind(limit as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"获取最近游戏列表失败: {}\", e)))?;
        
        Ok(games)
    }
    
    /// 删除游戏记录
    pub async fn delete_game(&self, game_id: &str) -> AppResult<()> {
        // 由于外键约束，删除游戏记录会自动删除相关的其他记录
        let result = sqlx::query(\"DELETE FROM game_records WHERE id = ?\")
            .bind(game_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!(\"删除游戏记录失败: {}\", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::Database(\"游戏记录不存在\".to_string()));
        }
        
        info!(\"删除游戏记录: {}\", game_id);
        Ok(())
    }
    
    // 私有辅助方法
    
    async fn create_player_record(&self, game_id: &str, player: &Player) -> AppResult<()> {
        let role_type = self.role_type_to_string(&player.role.role_type);
        let faction = self.faction_to_string(&player.role.faction);
        
        sqlx::query(
            r#\"
            INSERT INTO player_records (id, game_id, player_name, role_type, faction, is_ai, is_winner)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            \"#
        )
        .bind(&player.id)
        .bind(game_id)
        .bind(&player.name)
        .bind(&role_type)
        .bind(&faction)
        .bind(player.is_ai)
        .bind(false) // 初始时都不是获胜者
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!(\"创建玩家记录失败: {}\", e)))?;
        
        Ok(())
    }
    
    fn role_type_to_string(&self, role_type: &RoleType) -> String {
        match role_type {
            RoleType::Werewolf => \"werewolf\",
            RoleType::Villager => \"villager\",
            RoleType::Seer => \"seer\",
            RoleType::Witch => \"witch\",
            RoleType::Hunter => \"hunter\",
            RoleType::Guard => \"guard\",
        }.to_string()
    }
    
    fn faction_to_string(&self, faction: &Faction) -> String {
        match faction {
            Faction::Werewolf => \"werewolf\",
            Faction::Villager => \"villager\",
        }.to_string()
    }
    
    fn phase_to_string(&self, phase: &GamePhase) -> String {
        match phase {
            GamePhase::Preparation => \"preparation\",
            GamePhase::Night => \"night\",
            GamePhase::DayDiscussion => \"day_discussion\",
            GamePhase::Voting => \"voting\",
            GamePhase::LastWords => \"last_words\",
            GamePhase::GameOver => \"game_over\",
        }.to_string()
    }
    
    fn night_action_to_string(&self, action: &NightActionType) -> String {
        match action {
            NightActionType::Kill => \"kill\",
            NightActionType::Check => \"check\",
            NightActionType::Heal => \"heal\",
            NightActionType::Protect => \"protect\",
            NightActionType::Poison => \"poison\",
        }.to_string()
    }
}