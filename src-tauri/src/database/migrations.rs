// 数据库迁移脚本
// 这个模块包含数据库版本升级的迁移逻辑

use crate::error::{AppError, AppResult};
use sqlx::SqlitePool;
use log::{info, warn};

/// 数据库版本
const CURRENT_VERSION: i32 = 1;

/// 运行数据库迁移
pub async fn run_migrations(pool: &SqlitePool) -> AppResult<()> {
    // 创建版本表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("创建迁移表失败: {}", e)))?;
    
    // 获取当前版本
    let current_version = get_current_version(pool).await?;
    
    if current_version < CURRENT_VERSION {
        info!("开始数据库迁移，从版本 {} 到 {}", current_version, CURRENT_VERSION);
        
        for version in (current_version + 1)..=CURRENT_VERSION {
            apply_migration(pool, version).await?;
        }
        
        info!("数据库迁移完成");
    } else {
        info!("数据库已是最新版本: {}", current_version);
    }
    
    Ok(())
}

/// 获取当前数据库版本
async fn get_current_version(pool: &SqlitePool) -> AppResult<i32> {
    let version = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT MAX(version) FROM schema_migrations"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(format!("获取数据库版本失败: {}", e)))?;
    
    Ok(version.unwrap_or(0))
}

/// 应用特定版本的迁移
async fn apply_migration(pool: &SqlitePool, version: i32) -> AppResult<()> {
    info!("应用迁移版本: {}", version);
    
    match version {
        1 => apply_migration_v1(pool).await?,
        _ => {
            warn!("未知的迁移版本: {}", version);
            return Err(AppError::Database(format!("未知的迁移版本: {}", version)));
        }
    }
    
    // 记录迁移已应用
    sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
        .bind(version)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("记录迁移版本失败: {}", e)))?;
    
    Ok(())
}

/// 迁移版本1：初始数据库结构
async fn apply_migration_v1(pool: &SqlitePool) -> AppResult<()> {
    info!("应用迁移v1：创建初始表结构");
    
    // 这些迁移已经在DatabaseManager::run_migrations中实现
    // 这里只是为了版本控制
    
    Ok(())
}

/// 回滚迁移（紧急情况使用）
pub async fn rollback_migration(pool: &SqlitePool, target_version: i32) -> AppResult<()> {
    let current_version = get_current_version(pool).await?;
    
    if target_version >= current_version {
        warn!("目标版本 {} 不低于当前版本 {}", target_version, current_version);
        return Ok(());
    }
    
    warn!("回滚数据库从版本 {} 到 {}", current_version, target_version);
    
    for version in (target_version + 1)..=current_version {
        rollback_migration_version(pool, version).await?;
    }
    
    Ok(())
}

/// 回滚特定版本
async fn rollback_migration_version(pool: &SqlitePool, version: i32) -> AppResult<()> {
    warn!("回滚迁移版本: {}", version);
    
    match version {
        1 => rollback_migration_v1(pool).await?,
        _ => {
            warn!("未知的回滚版本: {}", version);
        }
    }
    
    // 删除迁移记录
    sqlx::query("DELETE FROM schema_migrations WHERE version = ?")
        .bind(version)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("删除迁移记录失败: {}", e)))?;
    
    Ok(())
}

/// 回滚版本1
async fn rollback_migration_v1(pool: &SqlitePool) -> AppResult<()> {
    warn!("回滚v1：删除所有表");
    
    let tables = [
        "ai_analysis_records",
        "night_action_records", 
        "vote_records",
        "speech_records",
        "player_records",
        "game_records"
    ];
    
    for table in &tables {
        sqlx::query(&format!("DROP TABLE IF EXISTS {}", table))
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(format!("删除表{}失败: {}", table, e)))?;
    }
    
    Ok(())
}
