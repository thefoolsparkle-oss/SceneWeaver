use super::Asset;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchRequest {
    pub raw_query: String,
    #[serde(default)]
    pub must: Vec<String>,
    #[serde(default)]
    pub should: Vec<String>,
    #[serde(default)]
    pub must_not: Vec<String>,
    #[serde(default)]
    pub media_types: Vec<String>,
    #[serde(default)]
    pub min_quality_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchResult {
    pub asset: Asset,
    pub score: f64,
    pub match_reasons: Vec<String>,
    pub unmet_should: Vec<String>,
}
