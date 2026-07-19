use std::sync::Arc;

use sceneweaver_lib::core::cache::CacheManager;
use sceneweaver_lib::core::db::Database;
use sceneweaver_lib::core::export::{
    write_csv, write_edl, write_fcpxml, write_json, write_select_contact_sheet_png,
    write_select_items_csv, write_select_items_edl, write_select_items_fcpxml,
    write_select_items_json,
};
use sceneweaver_lib::core::job_queue::{JobControl, ProgressUpdate};
use sceneweaver_lib::core::scanner::Scanner;
use sceneweaver_lib::models::{
    Entity, EntityReference, IndexProfile, Job, JobStatus, JobType, Library, LibraryStatus,
    SearchRequest,
};

struct SilentProgress;

impl ProgressUpdate for SilentProgress {
    fn report_progress(&self, _progress: f64, _processed: i64, _total: i64, _errors: i64) {}
    fn report_step(&self, _step: String) {}
    fn report_total(&self, _total: i64) {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!("sceneweaver-smoke-{}", uuid::Uuid::new_v4()));
    let media_root = root.join("中文 素材库");
    std::fs::create_dir_all(&media_root)?;
    let image_path = media_root.join("雨夜 角色.png");
    image::RgbImage::from_pixel(32, 18, image::Rgb([12, 34, 56])).save(&image_path)?;

    let db = Arc::new(Database::new(root.join("sceneweaver.db")));
    db.init()?;
    let legacy_database_path = root.join("legacy-v6.db");
    let legacy_connection = rusqlite::Connection::open(&legacy_database_path)?;
    legacy_connection.execute_batch(
        "CREATE TABLE assets (id TEXT PRIMARY KEY);
         CREATE TABLE asset_embeddings (asset_id TEXT PRIMARY KEY, provider_id TEXT NOT NULL, model_version TEXT NOT NULL, vector_json TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE);
         CREATE TABLE entities (id TEXT PRIMARY KEY);
         CREATE TABLE entity_references (id TEXT PRIMARY KEY, entity_id TEXT NOT NULL, asset_id TEXT, image_path TEXT, embedding_ref TEXT, is_positive INTEGER NOT NULL, created_at INTEGER NOT NULL);
         INSERT INTO assets (id) VALUES ('legacy-asset');
         INSERT INTO entities (id) VALUES ('legacy-entity');
         INSERT INTO entity_references (id, entity_id, embedding_ref, is_positive, created_at) VALUES ('legacy-reference', 'legacy-entity', '[1.0, 0.0]', 1, 1);
         INSERT INTO asset_embeddings (asset_id, provider_id, model_version, vector_json, created_at, updated_at) VALUES ('legacy-asset', 'legacy-provider', 'v1', '[1.0, 0.0]', 1, 1);
         PRAGMA user_version = 6;",
    )?;
    drop(legacy_connection);
    let legacy_database = Database::new(&legacy_database_path);
    legacy_database.init()?;
    legacy_database.upsert_asset_embedding("legacy-asset", "new-provider", "v1", &[0.0, 1.0])?;
    let legacy_embedding_count: i64 = rusqlite::Connection::open(&legacy_database_path)?
        .query_row(
            "SELECT COUNT(*) FROM asset_embeddings WHERE asset_id = 'legacy-asset'",
            [],
            |row| row.get(0),
        )?;
    assert_eq!(legacy_embedding_count, 2);
    let legacy_reference_embedding_count: i64 = rusqlite::Connection::open(&legacy_database_path)?
        .query_row(
            "SELECT COUNT(*) FROM entity_reference_embeddings WHERE reference_id = 'legacy-reference' AND provider_id = 'local-color-histogram'",
            [],
            |row| row.get(0),
        )?;
    assert_eq!(legacy_reference_embedding_count, 1);
    let cache = Arc::new(CacheManager::new(root.join("cache")));
    cache.ensure_dirs()?;
    let semantic_status = sceneweaver_lib::providers::semantic_clip::status(
        &cache.models_path(),
        &sceneweaver_lib::providers::semantic_clip::default_runtime_path(),
    );
    assert!(
        semantic_status.runtime_available,
        "bundled ONNX Runtime must be present"
    );
    assert!(
        !semantic_status.model_installed,
        "the smoke test must not implicitly download a semantic model"
    );
    for (repository, files) in [
        (
            "models--qdrant--clip-vit-b-32-vision",
            vec!["model.onnx", "preprocessor_config.json"],
        ),
        (
            "models--qdrant--clip-vit-b-32-text",
            vec![
                "model.onnx",
                "tokenizer.json",
                "config.json",
                "special_tokens_map.json",
                "tokenizer_config.json",
            ],
        ),
    ] {
        let snapshot = cache
            .models_path()
            .join(repository)
            .join("snapshots")
            .join("smoke");
        std::fs::create_dir_all(&snapshot)?;
        for file in files {
            std::fs::write(snapshot.join(file), "smoke")?;
        }
    }
    std::fs::write(cache.models_path().join("semantic-clip-v1.ready"), "smoke")?;
    assert!(
        sceneweaver_lib::providers::semantic_clip::status(
            &cache.models_path(),
            &sceneweaver_lib::providers::semantic_clip::default_runtime_path(),
        )
        .model_installed
    );
    std::fs::remove_file(
        cache
            .models_path()
            .join("models--qdrant--clip-vit-b-32-text")
            .join("snapshots")
            .join("smoke")
            .join("tokenizer.json"),
    )?;
    assert!(
        !sceneweaver_lib::providers::semantic_clip::status(
            &cache.models_path(),
            &sceneweaver_lib::providers::semantic_clip::default_runtime_path(),
        )
        .model_installed
    );
    let exclusion_safe_prompt =
        sceneweaver_lib::providers::semantic_clip::semantic_query_prompt_for_conditions(
            &["战斗".to_string()],
            &["近景".to_string(), "粉色头发".to_string()],
            "战斗 优先 近景、粉色头发 不要 游戏 UI、字幕",
        )
        .expect("known creator terms should produce a semantic prompt");
    assert!(exclusion_safe_prompt.contains("battle"));
    assert!(exclusion_safe_prompt.contains("pink hair"));
    assert!(!exclusion_safe_prompt.contains("subtitles"));
    std::fs::remove_dir_all(cache.models_path())?;
    std::fs::create_dir_all(cache.models_path())?;
    let now = chrono::Utc::now().timestamp_millis();
    let library = Library {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Smoke test 素材库".to_string(),
        root_path: media_root.to_string_lossy().to_string(),
        status: LibraryStatus::Idle,
        index_profile: IndexProfile::Quick,
        include_patterns: vec!["**/*".to_string()],
        exclude_patterns: vec![],
        watch_enabled: false,
        last_scan_at: None,
        created_at: now,
        updated_at: now,
    };
    db.create_library(&library)?;
    let job = Job {
        id: uuid::Uuid::new_v4().to_string(),
        job_type: JobType::Scan,
        library_id: Some(library.id.clone()),
        asset_id: None,
        status: JobStatus::Paused,
        priority: 0,
        progress: 0.5,
        current_step: "等待恢复".to_string(),
        checkpoint_json: Some("{\"processed\":1}".to_string()),
        error_code: None,
        error_message: None,
        started_at: Some(now),
        finished_at: None,
        created_at: now,
        updated_at: now,
    };
    db.create_job(&job)?;
    let reopened = Database::new(root.join("sceneweaver.db"));
    assert_eq!(
        reopened.get_job(&job.id)?.expect("job must persist").status,
        JobStatus::Paused
    );
    assert!(reopened
        .list_active_jobs()?
        .iter()
        .all(|active_job| active_job.id != job.id));
    let scanner = Scanner::new(Arc::clone(&db), Arc::clone(&cache));
    let control = JobControl::new();
    let progress = SilentProgress;

    let mut long_path_dir = media_root.clone();
    for index in 0..8 {
        long_path_dir.push(format!("超长路径-{index:02}-abcdefghijklmnopqrstuvwxyz"));
    }
    std::fs::create_dir_all(&long_path_dir)?;
    let long_path_image = long_path_dir.join("长路径素材.png");
    image::RgbImage::from_pixel(12, 12, image::Rgb([90, 60, 30])).save(&long_path_image)?;
    assert!(long_path_image.to_string_lossy().chars().count() > 260);

    let first = scanner.scan_library(&library, &control, &progress)?;
    let assets = db.list_assets(&library.id)?;
    assert_eq!(first.changed, 2);
    assert_eq!(assets.len(), 2);
    let primary_asset = assets
        .iter()
        .find(|asset| asset.file_name == "雨夜 角色.png")
        .cloned()
        .expect("primary smoke image must be indexed");
    assert!(cache.thumbnail_path(&primary_asset.id, "cover").is_file());
    let mut similar_asset = primary_asset.clone();
    similar_asset.id = uuid::Uuid::new_v4().to_string();
    similar_asset.normalized_path = format!("{}-similar", similar_asset.normalized_path);
    similar_asset.file_path = format!("{}-similar", similar_asset.file_path);
    similar_asset.file_name = "相似参考.png".to_string();
    db.create_or_update_asset(&similar_asset)?;
    let red = sceneweaver_lib::providers::visual_embedding::embed_color_query("红色")
        .expect("red colour query must be supported");
    let blue = sceneweaver_lib::providers::visual_embedding::embed_color_query("蓝色")
        .expect("blue colour query must be supported");
    db.upsert_asset_embedding(&primary_asset.id, "test", "v1", &red)?;
    db.upsert_asset_embedding(&similar_asset.id, "test", "v1", &blue)?;
    let embedding_count: i64 = rusqlite::Connection::open(root.join("sceneweaver.db"))?.query_row(
        "SELECT COUNT(*) FROM asset_embeddings WHERE asset_id = ?1",
        [&primary_asset.id],
        |row| row.get(0),
    )?;
    assert_eq!(embedding_count, 2);
    // A database created by the current schema must already advertise v8.
    // Reopening it must not replay the legacy single-provider migration.
    let reopened = Database::new(root.join("sceneweaver.db"));
    reopened.init()?;
    let schema_version: i64 = rusqlite::Connection::open(root.join("sceneweaver.db"))?.query_row(
        "PRAGMA user_version",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(schema_version, 8);
    let reopened_embedding_count: i64 = rusqlite::Connection::open(root.join("sceneweaver.db"))?
        .query_row(
            "SELECT COUNT(*) FROM asset_embeddings WHERE asset_id = ?1",
            [&primary_asset.id],
            |row| row.get(0),
        )?;
    assert_eq!(reopened_embedding_count, 2);
    assert!(db
        .similar_assets_for_provider(&primary_asset.id, "test", 5)?
        .iter()
        .any(|asset| asset.id == similar_asset.id));
    assert_eq!(
        db.assets_for_embedding_provider("test", &red, 5)?[0].id,
        primary_asset.id
    );
    assert_eq!(
        db.add_asset_acg_tag(&primary_asset.id, "游戏 UI")?,
        vec!["游戏 UI".to_string()]
    );
    assert_eq!(
        db.search_assets_with_conditions(
            &SearchRequest {
                raw_query: String::new(),
                must: vec!["游戏 UI".to_string()],
                should: vec![],
                must_not: vec![],
                media_types: vec![],
                min_quality_score: None,
            },
            10,
        )?[0]
            .id,
        primary_asset.id
    );
    db.remove_asset_acg_tag(&primary_asset.id, "游戏 UI")?;
    assert!(db.asset_acg_tags(&primary_asset.id)?.is_empty());
    let entity = Entity {
        id: uuid::Uuid::new_v4().to_string(),
        entity_type: "character".to_string(),
        name: "红色参考".to_string(),
        description: None,
        aliases: vec!["角色".to_string()],
        pack_id: None,
        created_at: now,
        updated_at: now,
    };
    db.create_entity(&entity)?;
    let positive_reference = EntityReference {
        id: uuid::Uuid::new_v4().to_string(),
        entity_id: entity.id.clone(),
        asset_id: None,
        image_path: Some(image_path.to_string_lossy().to_string()),
        is_positive: true,
        created_at: now,
    };
    let primary_colour_embedding =
        sceneweaver_lib::providers::visual_embedding::embed_image(&image_path)?;
    db.add_entity_reference(&positive_reference, &primary_colour_embedding)?;
    db.upsert_entity_reference_embedding(&positive_reference.id, "test", "v1", &red)?;
    let entity_request = SearchRequest {
        raw_query: "角色 雨夜".to_string(),
        must: vec!["角色".to_string()],
        should: vec!["雨夜".to_string()],
        must_not: vec![],
        media_types: vec![],
        min_quality_score: None,
    };
    assert!(db
        .entities_matching_search_request(&entity_request)?
        .iter()
        .any(|matched| matched.id == entity.id));
    assert!(db
        .entity_candidate_assets(&entity.id, false, 5)?
        .iter()
        .any(|asset| asset.id == primary_asset.id));
    let exclusion_only_entity_request = SearchRequest {
        raw_query: "不要角色".to_string(),
        must: vec![],
        should: vec![],
        must_not: vec!["角色".to_string()],
        media_types: vec![],
        min_quality_score: None,
    };
    assert!(db
        .entities_matching_search_request(&exclusion_only_entity_request)?
        .is_empty());
    assert_eq!(
        db.similar_assets_for_entity_provider(&entity.id, "test", 5)?[0].id,
        primary_asset.id
    );
    assert_eq!(
        db.similar_assets_for_entity(&entity.id, 5)?[0].id,
        primary_asset.id
    );
    assert!(db
        .assets_matching_entity_terms(&entity.id, 5)?
        .iter()
        .any(|asset| asset.id == primary_asset.id));
    db.remove_entity_reference(&entity.id, &positive_reference.id)?;
    assert!(db.list_entity_references(&entity.id)?.is_empty());
    db.set_entity_asset_feedback(&entity.id, &primary_asset.id, true)?;
    assert_eq!(
        db.similar_assets_for_entity(&entity.id, 5)?[0].id,
        primary_asset.id
    );
    db.set_entity_asset_feedback(&entity.id, &primary_asset.id, false)?;
    let feedback = db.list_entity_references(&entity.id)?;
    assert_eq!(feedback.len(), 1);
    assert_eq!(
        feedback[0].asset_id.as_deref(),
        Some(primary_asset.id.as_str())
    );
    assert!(!feedback[0].is_positive);
    db.remove_entity_reference(&entity.id, &feedback[0].id)?;
    assert!(db
        .search_assets_with_conditions(
            &SearchRequest {
                raw_query: String::new(),
                must: vec![],
                should: vec![],
                must_not: vec![],
                media_types: vec!["audio".to_string()],
                min_quality_score: None,
            },
            10,
        )?
        .is_empty());
    db.add_to_default_selects(&primary_asset.id)?;
    let selects = db.default_select_assets()?;
    assert_eq!(selects.len(), 1);
    let csv_path = root.join("selects.csv");
    write_csv(&csv_path, &selects)?;
    assert!(std::fs::read_to_string(&csv_path)?.contains("雨夜 角色.png"));
    let json_path = root.join("selects.json");
    write_json(&json_path, &selects)?;
    assert!(std::fs::read_to_string(&json_path)?.contains("file_path"));
    let edl_path = root.join("selects.edl");
    write_edl(&edl_path, &selects)?;
    assert!(std::fs::read_to_string(&edl_path)?.contains("TITLE: SceneWeaver Selects"));
    let fcpxml_path = root.join("selects.fcpxml");
    write_fcpxml(&fcpxml_path, &selects)?;
    let fcpxml = std::fs::read_to_string(&fcpxml_path)?;
    assert!(fcpxml.contains("<fcpxml"));
    assert!(fcpxml.contains("%E4%B8%AD%E6%96%87"));
    assert!(fcpxml.contains("hasVideo=\"1\"/>"));
    assert!(fcpxml.contains("id=\"format-timeline\""));
    assert!(fcpxml.contains("id=\"asset-1\""));
    assert!(fcpxml.contains("ref=\"asset-1\""));

    let segments = sceneweaver_lib::core::scene_detect::build_segments(
        &primary_asset.id,
        5_000,
        &[1_000, 3_000],
    );
    db.replace_segments(&primary_asset.id, &segments)?;
    db.add_segment_to_default_selects(&primary_asset.id, &segments[1].id)?;
    db.add_segment_to_default_selects(&primary_asset.id, &segments[1].id)?;
    let default_collection = db
        .list_select_collections()?
        .into_iter()
        .find(|collection| collection.name == "我的选片")
        .expect("default selects collection must exist");
    let select_items = db.list_select_items(&default_collection.id)?;
    assert_eq!(select_items.len(), 2);
    let segment_item = select_items
        .iter()
        .find(|item| item.segment_id.as_deref() == Some(segments[1].id.as_str()))
        .expect("segment select item must be present");
    assert_eq!(
        segment_item
            .segment
            .as_ref()
            .map(|segment| segment.start_ms),
        Some(1_000)
    );
    db.reorder_select_item(&segment_item.id, 0)?;
    assert_eq!(
        db.list_select_items(&default_collection.id)?[0].id,
        segment_item.id
    );
    let mut range_items = db.list_select_items(&default_collection.id)?;
    assert_eq!(db.default_select_items()?.len(), range_items.len());
    let range_csv_path = root.join("range-selects.csv");
    write_select_items_csv(&range_csv_path, &range_items)?;
    assert!(std::fs::read_to_string(&range_csv_path)?.contains("00:00:01.000"));
    let range_json_path = root.join("range-selects.json");
    write_select_items_json(&range_json_path, &range_items)?;
    assert!(std::fs::read_to_string(&range_json_path)?.contains("\"start_ms\": 1000"));
    let range_edl_path = root.join("range-selects.edl");
    write_select_items_edl(&range_edl_path, &range_items)?;
    assert!(std::fs::read_to_string(&range_edl_path)?.contains("00:00:01:00"));
    range_items[0].asset.fps = Some(23.976);
    range_items[0].asset.width = Some(1920);
    range_items[0].asset.height = Some(1080);
    let range_fcpxml_path = root.join("range-selects.fcpxml");
    write_select_items_fcpxml(&range_fcpxml_path, &range_items)?;
    let range_fcpxml = std::fs::read_to_string(&range_fcpxml_path)?;
    assert!(range_fcpxml.contains("start=\"1000/1000s\""));
    assert!(range_fcpxml.contains("frameDuration=\"1001/24000s\""));
    assert!(range_fcpxml.contains("format=\"format-asset-1\""));
    let contact_sheet_path = root.join("range-selects.png");
    write_select_contact_sheet_png(&contact_sheet_path, &range_items, &cache)?;
    assert_eq!(image::open(&contact_sheet_path)?.width(), 1_000);

    db.replace_segments(&primary_asset.id, &[])?;
    assert_eq!(db.list_select_items(&default_collection.id)?.len(), 1);

    // A source replacement must invalidate content-derived vectors and
    // segment ranges while keeping the stable asset-level select intact.
    let stale_segments =
        sceneweaver_lib::core::scene_detect::build_segments(&primary_asset.id, 2_000, &[1_000]);
    db.replace_segments(&primary_asset.id, &stale_segments)?;
    db.add_segment_to_default_selects(&primary_asset.id, &stale_segments[0].id)?;
    db.set_entity_asset_feedback(&entity.id, &primary_asset.id, true)?;
    assert!(db
        .list_select_items(&default_collection.id)?
        .iter()
        .any(|item| item.segment_id.is_some()));
    let feedback_provider_count: i64 = rusqlite::Connection::open(root.join("sceneweaver.db"))?
        .query_row(
            "SELECT COUNT(*) FROM entity_reference_embeddings e JOIN entity_references r ON r.id = e.reference_id WHERE r.entity_id = ?1 AND e.provider_id = 'test'",
            [&entity.id],
            |row| row.get(0),
        )?;
    assert_eq!(feedback_provider_count, 1);

    let corrupt_image = media_root.join("损坏素材.png");
    std::fs::write(&corrupt_image, b"not an image")?;
    let scan_with_corrupt_file = scanner.scan_library(&library, &control, &progress)?;
    assert_eq!(scan_with_corrupt_file.changed, 1);
    assert!(db
        .list_assets(&library.id)?
        .iter()
        .any(|asset| asset.normalized_path
            == sceneweaver_lib::core::scanner::normalize_path(&long_path_image)));

    let second = scanner.scan_library(&library, &control, &progress)?;
    assert_eq!(second.unchanged, 3);

    std::thread::sleep(std::time::Duration::from_millis(20));
    image::RgbImage::from_pixel(48, 27, image::Rgb([56, 34, 12])).save(&image_path)?;
    let changed = scanner.scan_library(&library, &control, &progress)?;
    let changed_asset = db
        .list_assets(&library.id)?
        .into_iter()
        .find(|asset| asset.id == primary_asset.id)
        .expect("original asset must remain present");
    assert_eq!(changed.changed, 1);
    assert_eq!(
        (changed_asset.width, changed_asset.height),
        (Some(48), Some(27))
    );
    let remaining_embeddings: i64 = rusqlite::Connection::open(root.join("sceneweaver.db"))?
        .query_row(
            "SELECT COUNT(*) FROM asset_embeddings WHERE asset_id = ?1",
            [&primary_asset.id],
            |row| row.get(0),
        )?;
    assert_eq!(remaining_embeddings, 1);
    assert!(db.list_segments(&primary_asset.id)?.is_empty());
    assert!(db
        .list_select_items(&default_collection.id)?
        .iter()
        .all(|item| item.segment_id.is_none()));
    let stale_feedback_provider_count: i64 = rusqlite::Connection::open(root.join("sceneweaver.db"))?
        .query_row(
            "SELECT COUNT(*) FROM entity_reference_embeddings e JOIN entity_references r ON r.id = e.reference_id WHERE r.entity_id = ?1 AND e.provider_id = 'test'",
            [&entity.id],
            |row| row.get(0),
        )?;
    assert_eq!(stale_feedback_provider_count, 0);
    assert_eq!(
        db.similar_assets_for_entity(&entity.id, 5)?[0].id,
        primary_asset.id
    );

    std::fs::remove_file(&image_path)?;
    let third = scanner.scan_library(&library, &control, &progress)?;
    assert_eq!(third.removed, 1);
    assert_eq!(
        db.list_assets(&library.id)?
            .into_iter()
            .find(|asset| asset.id == primary_asset.id)
            .expect("original asset must remain present")
            .status,
        sceneweaver_lib::models::AssetStatus::Offline
    );

    if which::which("ffmpeg").is_ok() && which::which("ffprobe").is_ok() {
        let video_root = root.join("视频 验证");
        std::fs::create_dir_all(&video_root)?;
        let video_path = video_root.join("红蓝切换.mp4");
        let status = std::process::Command::new("ffmpeg")
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=red:s=64x48:d=1",
                "-f",
                "lavfi",
                "-i",
                "color=c=blue:s=64x48:d=1",
                "-filter_complex",
                "[0:v]drawbox=x=2:y=32:w=8:h=6:color=white:t=fill,drawbox=x=54:y=32:w=8:h=6:color=white:t=fill[v0];[1:v]drawbox=x=2:y=32:w=8:h=6:color=white:t=fill,drawbox=x=54:y=32:w=8:h=6:color=white:t=fill[v1];[v0][v1]concat=n=2:v=1:a=0",
                "-c:v",
                "libx264",
            ])
            .arg(&video_path)
            .status()?;
        assert!(status.success(), "synthetic video generation must succeed");
        let video_library = Library {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Video smoke library".to_string(),
            root_path: video_root.to_string_lossy().to_string(),
            status: LibraryStatus::Idle,
            index_profile: IndexProfile::Quick,
            include_patterns: vec!["**/*".to_string()],
            exclude_patterns: vec![],
            watch_enabled: false,
            last_scan_at: None,
            created_at: now,
            updated_at: now,
        };
        db.create_library(&video_library)?;
        assert_eq!(
            scanner
                .scan_library(&video_library, &control, &progress)?
                .changed,
            1
        );
        let video_asset = db
            .list_assets(&video_library.id)?
            .pop()
            .expect("video must index");
        assert_eq!(
            video_asset.media_type,
            sceneweaver_lib::models::MediaType::Video
        );
        assert!(video_asset.duration_ms.unwrap_or(0) >= 1_900);
        let mut video_segments = db.list_segments(&video_asset.id)?;
        assert!(!video_segments.is_empty(), "video must create segments");
        assert!(video_segments
            .iter()
            .any(|segment| segment.thumbnail_path.is_some()));
        assert!(video_segments
            .iter()
            .any(|segment| segment.preview_path.is_some()));
        assert!(video_segments
            .iter()
            .any(|segment| segment.quality_score.is_some()));
        assert!(video_segments
            .iter()
            .any(|segment| segment.black_frame_score.is_some()));
        assert!(video_segments
            .iter()
            .any(|segment| segment.blur_score.is_some()));
        assert!(video_segments
            .iter()
            .any(|segment| segment.subtitle_present.is_some()));
        assert!(
            video_segments
                .iter()
                .any(|segment| segment.game_ui == Some(true)),
            "synthetic dual-corner HUD must be detected automatically"
        );

        // Subtitle is a structured segment signal, not merely a filename
        // keyword. Subtitle, black-frame and blur labels must be usable as
        // hard SQLite constraints even by semantic/entity retrieval paths.
        video_segments[0].subtitle_present = Some(true);
        video_segments[0].game_ui = Some(true);
        video_segments[0].black_frame_score = Some(0.9);
        video_segments[0].blur_score = Some(0.9);
        assert!(video_segments.len() >= 2, "test video must have two shots");
        video_segments[1].subtitle_present = Some(false);
        video_segments[1].game_ui = Some(false);
        video_segments[1].black_frame_score = Some(0.1);
        video_segments[1].blur_score = Some(0.1);
        db.replace_segments(&video_asset.id, &video_segments)?;
        let subtitle_request = SearchRequest {
            raw_query: String::new(),
            must: vec!["字幕".to_string()],
            should: vec![],
            must_not: vec![],
            media_types: vec!["video".to_string()],
            min_quality_score: None,
        };
        assert!(db
            .search_assets_with_conditions(&subtitle_request, 10)?
            .iter()
            .any(|asset| asset.id == video_asset.id));
        assert!(db
            .assets_matching_nonsemantic_filters(&subtitle_request, 10)?
            .iter()
            .any(|asset| asset.id == video_asset.id));
        let ui_request = SearchRequest {
            raw_query: String::new(),
            must: vec!["HUD".to_string()],
            should: vec![],
            must_not: vec![],
            media_types: vec!["video".to_string()],
            min_quality_score: None,
        };
        assert!(db
            .search_assets_with_conditions(&ui_request, 10)?
            .iter()
            .any(|asset| asset.id == video_asset.id));
        let clean_ui_segment_request = SearchRequest {
            must: vec![],
            must_not: vec!["game ui".to_string()],
            ..ui_request
        };
        assert!(db
            .assets_matching_nonsemantic_filters(&clean_ui_segment_request, 10)?
            .iter()
            .any(|asset| asset.id == video_asset.id));
        let clean_subtitle_segment_request = SearchRequest {
            must: vec![],
            must_not: vec!["subtitle".to_string()],
            ..subtitle_request
        };
        assert!(db
            .assets_matching_nonsemantic_filters(&clean_subtitle_segment_request, 10)?
            .iter()
            .any(|asset| asset.id == video_asset.id));

        let quality_label_request = SearchRequest {
            raw_query: String::new(),
            must: vec!["黑帧".to_string()],
            should: vec!["模糊".to_string()],
            must_not: vec![],
            media_types: vec!["video".to_string()],
            min_quality_score: None,
        };
        assert!(db
            .search_assets_with_conditions(&quality_label_request, 10)?
            .iter()
            .any(|asset| asset.id == video_asset.id));
        let safe_segment_request = SearchRequest {
            must: vec![],
            should: vec![],
            must_not: vec!["blur".to_string()],
            ..quality_label_request.clone()
        };
        assert!(db
            .assets_matching_nonsemantic_filters(&safe_segment_request, 10)?
            .iter()
            .any(|asset| asset.id == video_asset.id));
        let conflicting_segment_request = SearchRequest {
            must: vec!["字幕".to_string()],
            should: vec![],
            must_not: vec!["模糊".to_string()],
            ..quality_label_request
        };
        assert!(db
            .search_assets_with_conditions(&conflicting_segment_request, 10)?
            .iter()
            .all(|asset| asset.id != video_asset.id));
    }

    std::fs::remove_dir_all(&root)?;
    println!(
        "core smoke passed: Chinese + spaced path, long/corrupt media resilience, thumbnail, multi-provider visual index, bundled semantic runtime, incremental scan, offline detection, selects exports, segment selects and optional FFmpeg video derivatives"
    );
    Ok(())
}
