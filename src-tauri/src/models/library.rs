use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryStatus {
    Idle,
    Scanning,
    Paused,
    Failed,
}

impl LibraryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LibraryStatus::Idle => "idle",
            LibraryStatus::Scanning => "scanning",
            LibraryStatus::Paused => "paused",
            LibraryStatus::Failed => "error",
        }
    }
}

impl TryFrom<&str> for LibraryStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "idle" => Ok(LibraryStatus::Idle),
            "scanning" => Ok(LibraryStatus::Scanning),
            "paused" => Ok(LibraryStatus::Paused),
            "error" => Ok(LibraryStatus::Failed),
            _ => Err(format!("未知素材库状态: {}", value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexProfile {
    Quick,
    Balanced,
    Precise,
}

impl IndexProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            IndexProfile::Quick => "quick",
            IndexProfile::Balanced => "balanced",
            IndexProfile::Precise => "precise",
        }
    }
}

impl TryFrom<&str> for IndexProfile {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "quick" => Ok(IndexProfile::Quick),
            "balanced" => Ok(IndexProfile::Balanced),
            "precise" => Ok(IndexProfile::Precise),
            _ => Err(format!("未知索引模式: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Library {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub status: LibraryStatus,
    pub index_profile: IndexProfile,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub watch_enabled: bool,
    pub last_scan_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateLibraryRequest {
    pub name: String,
    pub root_path: String,
    pub index_profile: Option<IndexProfile>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReconnectLibraryRequest {
    pub library_id: String,
    pub root_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReconnectLibraryResult {
    pub library: Library,
    pub job: crate::models::Job,
    pub rebased_assets: usize,
    pub offline_assets: usize,
}
