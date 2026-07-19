use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Segment {
    pub id: String,
    pub asset_id: String,
    pub segment_type: String,
    pub segment_index: i32,
    pub start_ms: i64,
    pub end_ms: i64,
    pub duration_ms: i64,
    pub representative_frame_path: Option<String>,
    pub thumbnail_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_data_url: Option<String>,
    pub preview_path: Option<String>,
    pub quality_score: Option<f64>,
    pub subtitle_present: Option<bool>,
    pub game_ui: Option<bool>,
    pub black_frame_score: Option<f64>,
    pub blur_score: Option<f64>,
    pub embedding_ref: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
