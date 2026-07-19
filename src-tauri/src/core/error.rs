use serde::Serialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(String),

    #[error("IO 错误: {0}")]
    Io(String),

    #[error("序列化错误: {0}")]
    Json(String),

    #[error("路径不合法或存在目录穿越风险: {0}")]
    InvalidPath(PathBuf),

    #[error("素材库不存在: {0}")]
    LibraryNotFound(String),

    #[error("任务不存在: {0}")]
    JobNotFound(String),

    #[error("素材不存在: {0}")]
    AssetNotFound(String),

    #[error("ffprobe 不可用: {0}")]
    FfprobeUnavailable(String),

    #[error("任务已取消")]
    Cancelled,

    #[error("任务已暂停")]
    Paused,

    #[error("{0}")]
    Other(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e.to_string())
    }
}

impl From<walkdir::Error> for AppError {
    fn from(e: walkdir::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<image::ImageError> for AppError {
    fn from(e: image::ImageError) -> Self {
        AppError::Io(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
