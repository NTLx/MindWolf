use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 应用程序错误类型
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AppError {
    #[error("IO错误: {0}")]
    Io(String),
    
    #[error("序列化错误: {0}")]
    Serialization(String),
    
    #[error("网络错误: {0}")]
    Network(String),
    
    #[error("LLM API错误: {0}")]
    LlmApi(String),
    
    #[error("数据库错误: {0}")]
    Database(String),
    
    #[error("游戏逻辑错误: {0}")]
    GameLogic(String),
    
    #[error("配置错误: {0}")]
    Config(String),
    
    #[error("未知错误: {0}")]
    Unknown(String),
    
    #[error("未找到资源: {0}")]
    NotFound(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;