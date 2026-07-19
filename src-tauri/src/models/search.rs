use super::{Asset, Segment};
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
    /// Exact segments satisfying structured subtitle/quality conditions. An
    /// empty list means that this result was not narrowed by a segment label.
    #[serde(default)]
    pub matching_segment_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentLabel {
    Subtitle,
    GameUi,
    BlackFrame,
    Blurry,
}

pub fn segment_label_for_term(term: &str) -> Option<SegmentLabel> {
    match term.trim().to_lowercase().as_str() {
        "字幕" | "subtitle" | "subtitles" | "caption" | "captions" => {
            Some(SegmentLabel::Subtitle)
        }
        "ui" | "game ui" | "hud" | "interface" => Some(SegmentLabel::GameUi),
        "黑帧" | "黑屏" | "black frame" | "black frames" | "black screen" => {
            Some(SegmentLabel::BlackFrame)
        }
        "模糊" | "blur" | "blurry" | "out of focus" => Some(SegmentLabel::Blurry),
        _ => None,
    }
}

impl SegmentLabel {
    pub fn sql_predicate(self, positive: bool) -> &'static str {
        match (self, positive) {
            (SegmentLabel::Subtitle, true) => "EXISTS (SELECT 1 FROM segments WHERE segments.asset_id = assets.id AND segments.subtitle_present = 1)",
            (SegmentLabel::Subtitle, false) => "NOT EXISTS (SELECT 1 FROM segments WHERE segments.asset_id = assets.id AND segments.subtitle_present = 1)",
            (SegmentLabel::GameUi, true) => "EXISTS (SELECT 1 FROM segments WHERE segments.asset_id = assets.id AND segments.game_ui = 1)",
            (SegmentLabel::GameUi, false) => "NOT EXISTS (SELECT 1 FROM segments WHERE segments.asset_id = assets.id AND segments.game_ui = 1)",
            (SegmentLabel::BlackFrame, true) => "EXISTS (SELECT 1 FROM segments WHERE segments.asset_id = assets.id AND segments.black_frame_score >= 0.85)",
            (SegmentLabel::BlackFrame, false) => "NOT EXISTS (SELECT 1 FROM segments WHERE segments.asset_id = assets.id AND segments.black_frame_score >= 0.85)",
            (SegmentLabel::Blurry, true) => "EXISTS (SELECT 1 FROM segments WHERE segments.asset_id = assets.id AND segments.blur_score >= 0.80)",
            (SegmentLabel::Blurry, false) => "NOT EXISTS (SELECT 1 FROM segments WHERE segments.asset_id = assets.id AND segments.blur_score >= 0.80)",
        }
    }

    /// Predicate scoped to one candidate segment. The negative form treats a
    /// missing analysis value as "not flagged", so an unanalysed fallback
    /// segment remains usable instead of being silently discarded.
    pub fn segment_predicate(self, positive: bool) -> &'static str {
        match (self, positive) {
            (SegmentLabel::Subtitle, true) => "segments.subtitle_present = 1",
            (SegmentLabel::Subtitle, false) => {
                "(segments.subtitle_present IS NULL OR segments.subtitle_present = 0)"
            }
            (SegmentLabel::GameUi, true) => "segments.game_ui = 1",
            (SegmentLabel::GameUi, false) => "(segments.game_ui IS NULL OR segments.game_ui = 0)",
            (SegmentLabel::BlackFrame, true) => "segments.black_frame_score >= 0.85",
            (SegmentLabel::BlackFrame, false) => {
                "(segments.black_frame_score IS NULL OR segments.black_frame_score < 0.85)"
            }
            (SegmentLabel::Blurry, true) => "segments.blur_score >= 0.80",
            (SegmentLabel::Blurry, false) => {
                "(segments.blur_score IS NULL OR segments.blur_score < 0.80)"
            }
        }
    }

    pub fn matches_segment(self, segment: &Segment) -> bool {
        match self {
            SegmentLabel::Subtitle => segment.subtitle_present == Some(true),
            SegmentLabel::GameUi => segment.game_ui == Some(true),
            SegmentLabel::BlackFrame => segment.black_frame_score.unwrap_or_default() >= 0.85,
            SegmentLabel::Blurry => segment.blur_score.unwrap_or_default() >= 0.80,
        }
    }

    pub fn explanation(self) -> &'static str {
        match self {
            SegmentLabel::Subtitle => "本地字幕提示",
            SegmentLabel::GameUi => "本地 HUD 提示（双下角高对比元素）",
            SegmentLabel::BlackFrame => "本地黑帧提示（黑帧比例 ≥ 85%）",
            SegmentLabel::Blurry => "本地低细节/模糊提示（模糊分 ≥ 80%）",
        }
    }
}
