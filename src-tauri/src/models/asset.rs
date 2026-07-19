use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Video,
    Audio,
}

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::Image => "image",
            MediaType::Video => "video",
            MediaType::Audio => "audio",
        }
    }
}

impl TryFrom<&str> for MediaType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "image" => Ok(MediaType::Image),
            "video" => Ok(MediaType::Video),
            "audio" => Ok(MediaType::Audio),
            _ => Err(format!("未知媒体类型: {}", value)),
        }
    }
}

impl MediaType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "heic" | "heif"
            | "avif" | "jxl" => MediaType::Image,
            "mp4" | "mov" | "mkv" | "avi" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg"
            | "mts" | "m2ts" | "ts" => MediaType::Video,
            _ => MediaType::Audio,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    Pending,
    Indexed,
    Failed,
    Offline,
}

impl AssetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetStatus::Pending => "pending",
            AssetStatus::Indexed => "indexed",
            AssetStatus::Failed => "error",
            AssetStatus::Offline => "offline",
        }
    }
}

impl TryFrom<&str> for AssetStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(AssetStatus::Pending),
            "indexed" => Ok(AssetStatus::Indexed),
            "error" => Ok(AssetStatus::Failed),
            "offline" => Ok(AssetStatus::Offline),
            _ => Err(format!("未知素材状态: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Asset {
    pub id: String,
    pub library_id: String,
    pub media_type: MediaType,
    pub file_path: String,
    pub normalized_path: String,
    pub file_name: String,
    pub extension: String,
    pub size_bytes: i64,
    pub modified_at: i64,
    pub quick_fingerprint: String,
    pub full_hash: Option<String>,
    pub duration_ms: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fps: Option<f64>,
    pub codec: Option<String>,
    pub capture_time: Option<i64>,
    pub status: AssetStatus,
    pub index_level: i32,
    pub analysis_version: i32,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_data_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MediaProbeInfo {
    pub duration_ms: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fps: Option<f64>,
    pub codec: Option<String>,
}
