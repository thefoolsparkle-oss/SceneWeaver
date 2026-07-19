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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentLabel {
    Subtitle,
}

pub fn segment_label_for_term(term: &str) -> Option<SegmentLabel> {
    match term.trim().to_lowercase().as_str() {
        "字幕" | "subtitle" | "subtitles" | "caption" | "captions" => {
            Some(SegmentLabel::Subtitle)
        }
        _ => None,
    }
}

impl SegmentLabel {
    pub fn sql_predicate(self, positive: bool) -> &'static str {
        match (self, positive) {
            (SegmentLabel::Subtitle, true) => "EXISTS (SELECT 1 FROM segments WHERE segments.asset_id = assets.id AND segments.subtitle_present = 1)",
            (SegmentLabel::Subtitle, false) => "NOT EXISTS (SELECT 1 FROM segments WHERE segments.asset_id = assets.id AND segments.subtitle_present = 1)",
        }
    }
}
