use serde::{Deserialize, Serialize};

use super::{Asset, Segment};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SelectCollection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateSelectCollectionRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SelectItem {
    pub id: String,
    pub collection_id: String,
    pub asset_id: String,
    pub segment_id: Option<String>,
    pub position: i64,
    pub rating: Option<i32>,
    pub note: Option<String>,
    pub recommended_in_ms: Option<i64>,
    pub recommended_out_ms: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub asset: Asset,
    pub segment: Option<Segment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateSelectItemRequest {
    pub rating: Option<i32>,
    pub note: Option<String>,
    pub recommended_in_ms: Option<i64>,
    pub recommended_out_ms: Option<i64>,
}
