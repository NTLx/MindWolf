pub mod models;
pub mod migrations;
pub mod repository;

pub use models::*;
pub use repository::*;

use crate::error::{AppError, AppResult};
use sqlx::{SqlitePool, Row};
use std::path::PathBuf;
use log::{info, error};

/// 数据库管理器
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    /// 创建数据库管理器
    pub async fn new() -> AppResult<Self> {
        let db_path = Self::get_database_path()?;
        
        // 确保数据库目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Database(format!("创建数据库目录失败: {}", e)))?;
        }
        
        let database_url = format!("sqlite:{}", db_path.to_string_lossy());
        info!("连接数据库: {}", database_url);
        
        let pool = SqlitePool::connect(&database_url).await
            .map_err(|e| AppError::Database(format!("连接数据库失败: {}", e)))?;
        
        let manager = Self { pool };
        
        // 运行迁移
        manager.run_migrations().await?;
        
        Ok(manager)
    }
    
    /// 获取数据库路径
    fn get_database_path() -> AppResult<PathBuf> {
        let mut path = dirs::data_dir()
            .ok_or_else(|| AppError::Database("无法获取数据目录".to_string()))?;
        
        path.push("MindWolf");
        path.push("mindwolf.db");
        
        Ok(path)
    }
    
    /// 运行数据库迁移
    async fn run_migrations(&self) -> AppResult<()> {
        info!("运行数据库迁移...");
        
        // 创建游戏记录表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS game_records (
                id TEXT PRIMARY KEY,
                config TEXT NOT NULL,
                start_time DATETIME NOT NULL,
                end_time DATETIME,
                winner TEXT,
                player_count INTEGER NOT NULL,
                duration_seconds INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("创建game_records表失败: {}", e)))?;
        
        // 创建玩家记录表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS player_records (
                id TEXT PRIMARY KEY,
                game_id TEXT NOT NULL,
                player_name TEXT NOT NULL,
                role_type TEXT NOT NULL,
                faction TEXT NOT NULL,
                is_ai BOOLEAN NOT NULL,
                is_winner BOOLEAN NOT NULL,
                elimination_day INTEGER,
                final_votes INTEGER DEFAULT 0,
                FOREIGN KEY (game_id) REFERENCES game_records (id)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("创建player_records表失败: {}", e)))?;
        
        // 创建发言记录表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS speech_records (
                id TEXT PRIMARY KEY,
                game_id TEXT NOT NULL,
                player_id TEXT NOT NULL,
                content TEXT NOT NULL,
                day INTEGER NOT NULL,
                phase TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                analysis_result TEXT,
                FOREIGN KEY (game_id) REFERENCES game_records (id)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("创建speech_records表失败: {}", e)))?;
        
        // 创建投票记录表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS vote_records (
                id TEXT PRIMARY KEY,
                game_id TEXT NOT NULL,
                voter_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                day INTEGER NOT NULL,
                vote_round INTEGER NOT NULL,
                timestamp DATETIME NOT NULL,
                FOREIGN KEY (game_id) REFERENCES game_records (id)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("创建vote_records表失败: {}", e)))?;
        
        // 创建夜晚行动记录表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS night_action_records (
                id TEXT PRIMARY KEY,
                game_id TEXT NOT NULL,
                player_id TEXT NOT NULL,
                action_type TEXT NOT NULL,
                target_id TEXT,
                night INTEGER NOT NULL,
                result TEXT,
                timestamp DATETIME NOT NULL,
                FOREIGN KEY (game_id) REFERENCES game_records (id)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("创建night_action_records表失败: {}", e)))?;
        
        // 创建AI分析记录表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ai_analysis_records (
                id TEXT PRIMARY KEY,
                game_id TEXT NOT NULL,
                player_id TEXT NOT NULL,
                analysis_type TEXT NOT NULL,
                analysis_data TEXT NOT NULL,
                day INTEGER NOT NULL,
                timestamp DATETIME NOT NULL,
                FOREIGN KEY (game_id) REFERENCES game_records (id)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("创建ai_analysis_records表失败: {}", e)))?;
        
        info!("数据库迁移完成");
        Ok(())
    }
    
    /// 获取数据库连接池
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }
    
    /// 关闭数据库连接
    pub async fn close(self) {
        self.pool.close().await;
        info!("数据库连接已关闭");
    }
    
    /// 获取数据库统计信息
    pub async fn get_statistics(&self) -> AppResult<DatabaseStatistics> {
        let game_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM game_records")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("查询游戏数量失败: {}", e)))? as u32;
        
        let total_speeches = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM speech_records")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("查询发言数量失败: {}", e)))? as u32;
        
        let total_votes = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM vote_records")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("查询投票数量失败: {}", e)))? as u32;
        
        // 获取最近游戏时间
        let last_game_time = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
            "SELECT MAX(start_time) FROM game_records"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("查询最近游戏时间失败: {}", e)))?;
        
        Ok(DatabaseStatistics {
            total_games: game_count,
            total_speeches,
            total_votes,
            last_game_time,
        })
    }
    
    /// 清理旧数据
    pub async fn cleanup_old_data(&self, days_to_keep: u32) -> AppResult<u32> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_to_keep as i64);
        
        let result = sqlx::query(
            "DELETE FROM game_records WHERE start_time < ?"
        )
        .bind(cutoff_date)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("清理旧数据失败: {}", e)))?;
        
        let deleted_count = result.rows_affected() as u32;
        info!("清理了 {} 条旧游戏记录", deleted_count);
        
        Ok(deleted_count)
    }
}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    pub total_games: u32,
    pub total_speeches: u32,
    pub total_votes: u32,
    pub last_game_time: Option<chrono::DateTime<chrono::Utc>>,
}
