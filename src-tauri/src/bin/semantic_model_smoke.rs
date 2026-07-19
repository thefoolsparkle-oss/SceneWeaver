//! Explicit, networked smoke for the optional local semantic model.
//!
//! It is deliberately opt-in: normal tests and application startup never
//! download model weights. Run with `SCENEWEAVER_DOWNLOAD_SEMANTIC_MODEL=1`.
use sceneweaver_lib::core::db::Database;
use sceneweaver_lib::models::{
    Asset, AssetStatus, Entity, EntityReference, IndexProfile, Library, LibraryStatus, MediaType,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("SCENEWEAVER_DOWNLOAD_SEMANTIC_MODEL").as_deref() != Ok("1") {
        return Err(
            "set SCENEWEAVER_DOWNLOAD_SEMANTIC_MODEL=1 to explicitly run the networked model smoke"
                .into(),
        );
    }

    let root = std::env::temp_dir().join(format!(
        "sceneweaver-semantic-live-{}",
        uuid::Uuid::new_v4()
    ));
    if root.exists() {
        return Err("unexpected pre-existing semantic smoke directory".into());
    }
    std::fs::create_dir_all(&root)?;
    let image = root.join("红色样本.png");
    image::RgbImage::from_pixel(32, 32, image::Rgb([220, 30, 30])).save(&image)?;

    let runtime = sceneweaver_lib::providers::semantic_clip::default_runtime_path();
    let installed = sceneweaver_lib::providers::semantic_clip::install(&root, &runtime)?;
    assert!(
        installed.ready,
        "semantic model must be ready after installation"
    );
    let image_embedding =
        sceneweaver_lib::providers::semantic_clip::embed_image(&root, &runtime, &image)?;
    let text_embedding =
        sceneweaver_lib::providers::semantic_clip::embed_text(&root, &runtime, "a red object")?;
    assert_eq!(image_embedding.len(), 512);
    assert_eq!(text_embedding.len(), 512);
    let score = sceneweaver_lib::providers::visual_embedding::cosine_similarity(
        &image_embedding,
        &text_embedding,
    );
    assert!(score.is_finite());

    let database = Database::new(root.join("sceneweaver.db"));
    database.init()?;
    let now = chrono::Utc::now().timestamp_millis();
    let library = Library {
        id: uuid::Uuid::new_v4().to_string(),
        name: "semantic smoke".to_string(),
        root_path: root.to_string_lossy().to_string(),
        status: LibraryStatus::Idle,
        index_profile: IndexProfile::Quick,
        include_patterns: vec!["**/*".to_string()],
        exclude_patterns: vec![],
        watch_enabled: false,
        last_scan_at: None,
        created_at: now,
        updated_at: now,
    };
    database.create_library(&library)?;
    let asset = Asset {
        id: uuid::Uuid::new_v4().to_string(),
        library_id: library.id,
        media_type: MediaType::Image,
        file_path: image.to_string_lossy().to_string(),
        normalized_path: image.to_string_lossy().to_string(),
        file_name: "红色样本.png".to_string(),
        extension: "png".to_string(),
        size_bytes: std::fs::metadata(&image)?.len() as i64,
        modified_at: now,
        quick_fingerprint: "semantic-smoke".to_string(),
        full_hash: None,
        duration_ms: None,
        width: Some(32),
        height: Some(32),
        fps: None,
        codec: None,
        capture_time: None,
        status: AssetStatus::Indexed,
        index_level: 1,
        analysis_version: 1,
        created_at: now,
        updated_at: now,
        thumbnail_data_url: None,
    };
    database.create_or_update_asset(&asset)?;
    database.upsert_asset_embedding(
        &asset.id,
        sceneweaver_lib::providers::semantic_clip::PROVIDER_ID,
        sceneweaver_lib::providers::semantic_clip::MODEL_VERSION,
        &image_embedding,
    )?;
    assert_eq!(
        database.assets_for_embedding_provider(
            sceneweaver_lib::providers::semantic_clip::PROVIDER_ID,
            &text_embedding,
            1,
        )?[0]
            .id,
        asset.id
    );
    let entity = Entity {
        id: uuid::Uuid::new_v4().to_string(),
        entity_type: "character".to_string(),
        name: "红色样本".to_string(),
        description: None,
        aliases: vec![],
        pack_id: None,
        created_at: now,
        updated_at: now,
    };
    database.create_entity(&entity)?;
    let reference = EntityReference {
        id: uuid::Uuid::new_v4().to_string(),
        entity_id: entity.id.clone(),
        asset_id: None,
        image_path: Some(image.to_string_lossy().to_string()),
        is_positive: true,
        created_at: now,
    };
    // Entity rows keep a colour fallback for users who later remove the model;
    // the semantic row is persisted independently.
    database.add_entity_reference(&reference, &vec![1.0, 0.0, 0.0])?;
    database.upsert_entity_reference_embedding(
        &reference.id,
        sceneweaver_lib::providers::semantic_clip::PROVIDER_ID,
        sceneweaver_lib::providers::semantic_clip::MODEL_VERSION,
        &image_embedding,
    )?;
    assert_eq!(
        database.similar_assets_for_entity_provider(
            &entity.id,
            sceneweaver_lib::providers::semantic_clip::PROVIDER_ID,
            1,
        )?[0]
            .id,
        asset.id
    );
    std::fs::remove_dir_all(&root)?;
    println!(
        "semantic model smoke passed: downloaded paired CLIP encoders, produced 512-D image/text vectors, persisted text retrieval and entity semantic retrieval (cosine={score:.3})"
    );
    Ok(())
}
