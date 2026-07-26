//! Fixtures exposed only by the explicit desktop WebDriver build feature.
//! They are never compiled into release or installer binaries.

use chrono::Utc;
use serde::Serialize;
use tauri::State;

use crate::core::app_state::AppState;
use crate::core::error::AppResult;
use crate::models::{
    Asset, AssetStatus, IndexProfile, Library, LibraryStatus, MediaType, Segment, SelectCollection,
};

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SegmentSelectFixture {
    pub library_id: String,
    pub collection_id: String,
    pub asset_id: String,
    pub segment_id: String,
}

#[tauri::command]
pub async fn seed_e2e_segment_select_fixture(
    state: State<'_, AppState>,
) -> AppResult<SegmentSelectFixture> {
    let now = Utc::now().timestamp_millis();
    let library_id = uuid::Uuid::new_v4().to_string();
    let asset_id = uuid::Uuid::new_v4().to_string();
    let segment_id = uuid::Uuid::new_v4().to_string();
    let collection_id = uuid::Uuid::new_v4().to_string();

    state.db.create_library(&Library {
        id: library_id.clone(),
        name: "E2E 片段素材库".to_string(),
        root_path: "C:\\SceneWeaver E2E Fixture".to_string(),
        status: LibraryStatus::Idle,
        index_profile: IndexProfile::Quick,
        include_patterns: Vec::new(),
        exclude_patterns: Vec::new(),
        watch_enabled: false,
        last_scan_at: Some(now),
        created_at: now,
        updated_at: now,
    })?;
    state.db.create_or_update_asset(&Asset {
        id: asset_id.clone(),
        library_id: library_id.clone(),
        media_type: MediaType::Video,
        file_path: "C:\\SceneWeaver E2E Fixture\\fixture.mp4".to_string(),
        normalized_path: "C:\\SceneWeaver E2E Fixture\\fixture.mp4".to_string(),
        file_name: "fixture.mp4".to_string(),
        extension: "mp4".to_string(),
        size_bytes: 1,
        modified_at: now,
        quick_fingerprint: "e2e-fixture".to_string(),
        full_hash: None,
        duration_ms: Some(5_000),
        width: Some(1920),
        height: Some(1080),
        fps: Some(24.0),
        codec: Some("h264".to_string()),
        capture_time: None,
        status: AssetStatus::Indexed,
        index_level: 1,
        analysis_version: 1,
        created_at: now,
        updated_at: now,
        thumbnail_data_url: None,
    })?;
    state.db.replace_segments(
        &asset_id,
        &[Segment {
            id: segment_id.clone(),
            asset_id: asset_id.clone(),
            segment_type: "scene".to_string(),
            segment_index: 0,
            start_ms: 1_000,
            end_ms: 4_000,
            duration_ms: 3_000,
            representative_frame_path: None,
            thumbnail_path: None,
            thumbnail_data_url: None,
            preview_path: None,
            quality_score: Some(0.9),
            subtitle_present: Some(false),
            game_ui: Some(false),
            black_frame_score: Some(0.0),
            blur_score: Some(0.0),
            embedding_ref: None,
            created_at: now,
            updated_at: now,
        }],
    )?;
    state.db.create_select_collection(&SelectCollection {
        id: collection_id.clone(),
        name: "E2E 片段集合".to_string(),
        description: Some("仅用于隔离桌面自动化".to_string()),
        created_at: now,
        updated_at: now,
    })?;

    Ok(SegmentSelectFixture {
        library_id,
        collection_id,
        asset_id,
        segment_id,
    })
}
