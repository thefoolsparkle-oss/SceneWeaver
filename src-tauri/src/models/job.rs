use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Paused => "paused",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for JobStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(JobStatus::Pending),
            "running" => Ok(JobStatus::Running),
            "paused" => Ok(JobStatus::Paused),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            "cancelled" => Ok(JobStatus::Cancelled),
            _ => Err(format!("未知任务状态: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Scan,
    Thumbnail,
    ShotDetect,
    Index,
}

impl JobType {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobType::Scan => "scan",
            JobType::Thumbnail => "thumbnail",
            JobType::ShotDetect => "shot_detect",
            JobType::Index => "index",
        }
    }
}

impl TryFrom<&str> for JobType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "scan" => Ok(JobType::Scan),
            "thumbnail" => Ok(JobType::Thumbnail),
            "shot_detect" => Ok(JobType::ShotDetect),
            "index" => Ok(JobType::Index),
            _ => Err(format!("未知任务类型: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub library_id: Option<String>,
    pub asset_id: Option<String>,
    pub status: JobStatus,
    pub priority: i32,
    pub progress: f64,
    pub current_step: String,
    pub checkpoint_json: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ScanProgress {
    pub job_id: String,
    pub library_id: String,
    pub status: JobStatus,
    pub progress: f64,
    pub current_step: String,
    pub processed: i64,
    pub total: i64,
    pub errors: i64,
}
