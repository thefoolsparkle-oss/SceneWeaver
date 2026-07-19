use chrono::Utc;
use tauri::State;

use crate::core::app_state::AppState;
use crate::core::error::{AppError, AppResult};
use crate::models::{
    CreateSelectCollectionRequest, SelectCollection, SelectItem, UpdateSelectItemRequest,
};

#[tauri::command]
pub async fn list_select_collections(
    state: State<'_, AppState>,
) -> AppResult<Vec<SelectCollection>> {
    state.db.list_select_collections()
}

#[tauri::command]
pub async fn create_select_collection(
    state: State<'_, AppState>,
    request: CreateSelectCollectionRequest,
) -> AppResult<SelectCollection> {
    let name = request.name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(AppError::Other(
            "选片集合名称不能为空且不能超过 80 个字符".to_string(),
        ));
    }
    let now = Utc::now().timestamp_millis();
    let collection = SelectCollection {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        description: request.description.filter(|value| !value.trim().is_empty()),
        created_at: now,
        updated_at: now,
    };
    state.db.create_select_collection(&collection)?;
    Ok(collection)
}

#[tauri::command]
pub async fn list_select_items(
    state: State<'_, AppState>,
    collection_id: String,
) -> AppResult<Vec<SelectItem>> {
    let mut items = state.db.list_select_items(&collection_id)?;
    for item in &mut items {
        attach_thumbnail_data_url(&state, &mut item.asset);
        if let Some(segment) = item.segment.as_mut() {
            attach_segment_thumbnail_data_url(&state, segment);
        }
    }
    Ok(items)
}

#[tauri::command]
pub async fn add_asset_to_select_collection(
    state: State<'_, AppState>,
    collection_id: String,
    asset_id: String,
) -> AppResult<()> {
    state
        .db
        .add_asset_to_select_collection(&collection_id, &asset_id)
}

#[tauri::command]
pub async fn update_select_item(
    state: State<'_, AppState>,
    item_id: String,
    request: UpdateSelectItemRequest,
) -> AppResult<SelectItem> {
    validate_update_request(&request)?;
    let mut item = state.db.update_select_item(&item_id, &request)?;
    attach_thumbnail_data_url(&state, &mut item.asset);
    if let Some(segment) = item.segment.as_mut() {
        attach_segment_thumbnail_data_url(&state, segment);
    }
    Ok(item)
}

fn validate_update_request(request: &UpdateSelectItemRequest) -> AppResult<()> {
    if request
        .rating
        .is_some_and(|rating| !(0..=5).contains(&rating))
    {
        return Err(AppError::Other("评分必须在 0 到 5 之间".to_string()));
    }
    if request.recommended_in_ms.is_some_and(|value| value < 0)
        || request.recommended_out_ms.is_some_and(|value| value < 0)
        || matches!((request.recommended_in_ms, request.recommended_out_ms), (Some(start), Some(end)) if start >= end)
    {
        return Err(AppError::Other("推荐入点必须早于推荐出点".to_string()));
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_select_item(state: State<'_, AppState>, item_id: String) -> AppResult<()> {
    state.db.remove_select_item(&item_id)
}

#[tauri::command]
pub async fn move_select_item(
    state: State<'_, AppState>,
    item_id: String,
    collection_id: String,
) -> AppResult<()> {
    state.db.move_select_item(&item_id, &collection_id)
}

#[tauri::command]
pub async fn reorder_select_item(
    state: State<'_, AppState>,
    item_id: String,
    target_position: i64,
) -> AppResult<()> {
    state.db.reorder_select_item(&item_id, target_position)
}

#[tauri::command]
pub async fn export_select_collection_csv(
    state: State<'_, AppState>,
    collection_id: String,
    path: String,
) -> AppResult<()> {
    let output_path = validate_output_path(&path)?;
    crate::core::export::write_select_items_csv(
        output_path,
        &state.db.list_select_items(&collection_id)?,
    )
}

#[tauri::command]
pub async fn export_select_collection_json(
    state: State<'_, AppState>,
    collection_id: String,
    path: String,
) -> AppResult<()> {
    let output_path = validate_output_path(&path)?;
    crate::core::export::write_select_items_json(
        output_path,
        &state.db.list_select_items(&collection_id)?,
    )
}

#[tauri::command]
pub async fn export_select_collection_edl(
    state: State<'_, AppState>,
    collection_id: String,
    path: String,
) -> AppResult<()> {
    let output_path = validate_output_path(&path)?;
    crate::core::export::write_select_items_edl(
        output_path,
        &state.db.list_select_items(&collection_id)?,
    )
}

#[tauri::command]
pub async fn export_select_collection_fcpxml(
    state: State<'_, AppState>,
    collection_id: String,
    path: String,
) -> AppResult<()> {
    let output_path = validate_output_path(&path)?;
    crate::core::export::write_select_items_fcpxml(
        output_path,
        &state.db.list_select_items(&collection_id)?,
    )
}

#[tauri::command]
pub async fn export_select_collection_contact_sheet(
    state: State<'_, AppState>,
    collection_id: String,
    path: String,
) -> AppResult<()> {
    let output_path = validate_output_path(&path)?;
    crate::core::export::write_select_contact_sheet_png(
        output_path,
        &state.db.list_select_items(&collection_id)?,
        &state.cache,
    )
}

#[tauri::command]
pub async fn export_select_collection_contact_sheet_html(
    state: State<'_, AppState>,
    collection_id: String,
    path: String,
) -> AppResult<()> {
    let output_path = validate_output_path(&path)?;
    crate::core::export::write_select_contact_sheet_html(
        output_path,
        &state.db.list_select_items(&collection_id)?,
        &state.cache,
    )
}

fn validate_output_path(path: &str) -> AppResult<&std::path::Path> {
    let output_path = std::path::Path::new(path);
    if !output_path.is_absolute()
        || output_path
            .parent()
            .filter(|parent| parent.is_dir())
            .is_none()
    {
        return Err(AppError::InvalidPath(output_path.to_path_buf()));
    }
    Ok(output_path)
}

fn attach_thumbnail_data_url(state: &AppState, asset: &mut crate::models::Asset) {
    let path = state.cache.thumbnail_path(&asset.id, "cover");
    if let Ok(bytes) = std::fs::read(path) {
        asset.thumbnail_data_url =
            Some(format!("data:image/jpeg;base64,{}", base64_encode(&bytes)));
    }
}

fn attach_segment_thumbnail_data_url(state: &AppState, segment: &mut crate::models::Segment) {
    let Some(path) = segment.thumbnail_path.as_ref() else {
        return;
    };
    let Ok(cache_root) = std::fs::canonicalize(state.cache.root()) else {
        return;
    };
    let Ok(path) = std::fs::canonicalize(path) else {
        return;
    };
    if !path.starts_with(&cache_root)
        || !matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("jpg") | Some("jpeg")
        )
    {
        return;
    }
    const MAX_THUMBNAIL_RESPONSE_BYTES: usize = 3 * 1024 * 1024;
    if let Ok(bytes) = std::fs::read(path) {
        if bytes.len() <= MAX_THUMBNAIL_RESPONSE_BYTES {
            segment.thumbnail_data_url =
                Some(format!("data:image/jpeg;base64,{}", base64_encode(&bytes)));
        }
    }
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

#[cfg(test)]
mod tests {
    use super::validate_update_request;
    use crate::models::UpdateSelectItemRequest;

    #[test]
    fn accepts_valid_select_annotation() {
        let request = UpdateSelectItemRequest {
            rating: Some(5),
            note: Some("保留".to_string()),
            recommended_in_ms: Some(100),
            recommended_out_ms: Some(1_000),
        };
        assert!(validate_update_request(&request).is_ok());
    }

    #[test]
    fn rejects_invalid_select_annotation() {
        let rating = UpdateSelectItemRequest {
            rating: Some(6),
            note: None,
            recommended_in_ms: None,
            recommended_out_ms: None,
        };
        let range = UpdateSelectItemRequest {
            rating: None,
            note: None,
            recommended_in_ms: Some(1_000),
            recommended_out_ms: Some(100),
        };
        assert!(validate_update_request(&rating).is_err());
        assert!(validate_update_request(&range).is_err());
    }
}
