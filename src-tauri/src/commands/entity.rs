use chrono::Utc;
use tauri::State;

use crate::core::app_state::AppState;
use crate::core::error::{AppError, AppResult};
use crate::models::{CreateEntityRequest, Entity, EntityReference};

#[tauri::command]
pub async fn create_entity(
    state: State<'_, AppState>,
    request: CreateEntityRequest,
) -> AppResult<Entity> {
    if request.name.trim().is_empty() || request.entity_type.trim().is_empty() {
        return Err(AppError::Other("实体名称和类型不能为空".to_string()));
    }
    let now = Utc::now().timestamp_millis();
    let entity = Entity {
        id: uuid::Uuid::new_v4().to_string(),
        entity_type: request.entity_type.trim().to_string(),
        name: request.name.trim().to_string(),
        description: request.description.filter(|value| !value.trim().is_empty()),
        aliases: request
            .aliases
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect(),
        pack_id: None,
        created_at: now,
        updated_at: now,
    };
    state.db.create_entity(&entity)?;
    Ok(entity)
}

#[tauri::command]
pub async fn list_entities(state: State<'_, AppState>) -> AppResult<Vec<Entity>> {
    state.db.list_entities()
}

#[tauri::command]
pub async fn add_entity_reference_image(
    state: State<'_, AppState>,
    entity_id: String,
    image_path: String,
    is_positive: bool,
) -> AppResult<EntityReference> {
    let path = std::path::PathBuf::from(image_path);
    if !path.is_absolute() || !path.is_file() {
        return Err(AppError::InvalidPath(path));
    }
    let vector = crate::providers::visual_embedding::embed_image(&path)?;
    let reference = EntityReference {
        id: uuid::Uuid::new_v4().to_string(),
        entity_id,
        asset_id: None,
        image_path: Some(path.to_string_lossy().to_string()),
        is_positive,
        created_at: Utc::now().timestamp_millis(),
    };
    state.db.add_entity_reference(&reference, &vector)?;
    let semantic_status = crate::providers::semantic_clip::status(
        &state.cache.models_path(),
        &crate::providers::semantic_clip::default_runtime_path(),
    );
    if semantic_status.ready {
        match crate::providers::semantic_clip::embed_image(
            &state.cache.models_path(),
            &crate::providers::semantic_clip::default_runtime_path(),
            &path,
        ) {
            Ok(embedding) => state.db.upsert_entity_reference_embedding(
                &reference.id,
                crate::providers::semantic_clip::PROVIDER_ID,
                crate::providers::semantic_clip::MODEL_VERSION,
                &embedding,
            )?,
            Err(error) => log::warn!("实体参考图语义索引降级为颜色特征: {error}"),
        }
    }
    Ok(reference)
}

#[tauri::command]
pub async fn list_entity_references(
    state: State<'_, AppState>,
    entity_id: String,
) -> AppResult<Vec<EntityReference>> {
    state.db.list_entity_references(&entity_id)
}

#[tauri::command]
pub async fn remove_entity_reference(
    state: State<'_, AppState>,
    entity_id: String,
    reference_id: String,
) -> AppResult<()> {
    state.db.remove_entity_reference(&entity_id, &reference_id)
}

#[tauri::command]
pub async fn set_entity_asset_feedback(
    state: State<'_, AppState>,
    entity_id: String,
    asset_id: String,
    is_positive: bool,
) -> AppResult<()> {
    state
        .db
        .set_entity_asset_feedback(&entity_id, &asset_id, is_positive)
}

#[tauri::command]
pub async fn find_assets_for_entity(
    state: State<'_, AppState>,
    entity_id: String,
) -> AppResult<Vec<crate::models::Asset>> {
    let mut assets = state.db.assets_matching_entity_terms(&entity_id, 100)?;
    if let Ok(visual_matches) = state.db.similar_assets_for_entity(&entity_id, 100) {
        for asset in visual_matches {
            if !assets.iter().any(|existing| existing.id == asset.id) {
                assets.push(asset);
            }
        }
    }
    let semantic_status = crate::providers::semantic_clip::status(
        &state.cache.models_path(),
        &crate::providers::semantic_clip::default_runtime_path(),
    );
    if semantic_status.ready {
        if let Ok(semantic_matches) = state.db.similar_assets_for_entity_provider(
            &entity_id,
            crate::providers::semantic_clip::PROVIDER_ID,
            100,
        ) {
            for asset in semantic_matches {
                if !assets.iter().any(|existing| existing.id == asset.id) {
                    assets.push(asset);
                }
            }
        }
    }
    for asset in &mut assets {
        let path = state.cache.thumbnail_path(&asset.id, "cover");
        if let Ok(bytes) = std::fs::read(path) {
            asset.thumbnail_data_url =
                Some(format!("data:image/jpeg;base64,{}", base64_encode(&bytes)));
        }
    }
    Ok(assets)
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        encoded.push(TABLE[(first >> 2) as usize] as char);
        encoded.push(TABLE[(((first & 3) << 4) | (second >> 4)) as usize] as char);
        encoded.push(if chunk.len() > 1 {
            TABLE[(((second & 15) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            TABLE[(third & 63) as usize] as char
        } else {
            '='
        });
    }
    encoded
}
