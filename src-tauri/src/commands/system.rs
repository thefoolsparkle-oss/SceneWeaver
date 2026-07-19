use tauri::State;

use crate::core::app_state::AppState;
use crate::core::error::AppResult;
use crate::models::AppStats;

#[tauri::command]
pub async fn get_app_stats(state: State<'_, AppState>) -> AppResult<AppStats> {
    state.db.get_app_stats()
}

#[tauri::command]
pub async fn acg_creator_pack_enabled(state: State<'_, AppState>) -> AppResult<bool> {
    state.db.acg_creator_pack_enabled()
}

#[tauri::command]
pub async fn set_acg_creator_pack_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> AppResult<()> {
    state.db.set_acg_creator_pack_enabled(enabled)
}

#[tauri::command]
pub async fn semantic_model_status(
    state: State<'_, AppState>,
) -> AppResult<crate::providers::semantic_clip::SemanticModelStatus> {
    Ok(crate::providers::semantic_clip::status(
        &state.cache.models_path(),
        &crate::providers::semantic_clip::default_runtime_path(),
    ))
}

#[tauri::command]
pub async fn install_semantic_model(
    state: State<'_, AppState>,
) -> AppResult<crate::providers::semantic_clip::SemanticModelStatus> {
    let models = state.cache.models_path();
    let runtime = crate::providers::semantic_clip::default_runtime_path();
    tokio::task::spawn_blocking(move || crate::providers::semantic_clip::install(&models, &runtime))
        .await
        .map_err(|error| {
            crate::core::error::AppError::Other(format!("语义模型安装任务异常终止: {error}"))
        })?
}

#[tauri::command]
pub async fn reindex_semantic_assets(
    state: State<'_, AppState>,
) -> AppResult<crate::providers::semantic_clip::SemanticIndexResult> {
    let assets = state.db.list_indexed_visual_assets()?;
    let mut entity_reference_sources = Vec::new();
    for entity in state.db.list_entities()? {
        for reference in state.db.list_entity_references(&entity.id)? {
            let source = if let Some(path) = reference.image_path.as_deref() {
                std::path::PathBuf::from(path)
            } else if let Some(asset_id) = reference.asset_id.as_deref() {
                let asset = state.db.get_asset(asset_id)?;
                match asset {
                    Some(asset) if asset.media_type == crate::models::MediaType::Image => {
                        std::path::PathBuf::from(asset.file_path)
                    }
                    Some(asset) if asset.media_type == crate::models::MediaType::Video => {
                        state.cache.thumbnail_path(&asset.id, "cover")
                    }
                    _ => continue,
                }
            } else {
                continue;
            };
            entity_reference_sources.push((reference.id, source));
        }
    }
    let db = std::sync::Arc::clone(&state.db);
    let cache = std::sync::Arc::clone(&state.cache);
    tokio::task::spawn_blocking(move || {
        let models = cache.models_path();
        let runtime = crate::providers::semantic_clip::default_runtime_path();
        let mut result = crate::providers::semantic_clip::SemanticIndexResult {
            indexed: 0,
            skipped: 0,
            failed: 0,
            entity_references_indexed: 0,
            entity_references_skipped: 0,
            entity_references_failed: 0,
        };
        for asset in assets {
            let source = match asset.media_type {
                crate::models::MediaType::Image => std::path::PathBuf::from(&asset.file_path),
                crate::models::MediaType::Video => cache.thumbnail_path(&asset.id, "cover"),
                crate::models::MediaType::Audio => {
                    result.skipped += 1;
                    continue;
                }
            };
            if !source.is_file() {
                result.skipped += 1;
                continue;
            }
            match crate::providers::semantic_clip::embed_image(&models, &runtime, &source) {
                Ok(vector) => {
                    db.upsert_asset_embedding(
                        &asset.id,
                        crate::providers::semantic_clip::PROVIDER_ID,
                        crate::providers::semantic_clip::MODEL_VERSION,
                        &vector,
                    )?;
                    result.indexed += 1;
                }
                Err(error) => {
                    log::warn!("语义索引跳过 {}: {error}", asset.file_path);
                    result.failed += 1;
                }
            }
        }
        for (reference_id, source) in entity_reference_sources {
            if !source.is_file() {
                result.entity_references_skipped += 1;
                continue;
            }
            match crate::providers::semantic_clip::embed_image(&models, &runtime, &source) {
                Ok(vector) => {
                    db.upsert_entity_reference_embedding(
                        &reference_id,
                        crate::providers::semantic_clip::PROVIDER_ID,
                        crate::providers::semantic_clip::MODEL_VERSION,
                        &vector,
                    )?;
                    result.entity_references_indexed += 1;
                }
                Err(error) => {
                    log::warn!("实体参考图语义索引跳过 {}: {error}", source.display());
                    result.entity_references_failed += 1;
                }
            }
        }
        Ok(result)
    })
    .await
    .map_err(|error| {
        crate::core::error::AppError::Other(format!("语义索引任务异常终止: {error}"))
    })?
}

#[tauri::command]
pub async fn open_asset(state: State<'_, AppState>, asset_id: String) -> AppResult<()> {
    let asset = state
        .db
        .get_asset(&asset_id)?
        .ok_or_else(|| crate::core::error::AppError::AssetNotFound(asset_id))?;
    let target = std::path::Path::new(&asset.file_path);
    if !target.is_file() {
        return Err(crate::core::error::AppError::InvalidPath(
            target.to_path_buf(),
        ));
    }
    std::process::Command::new("explorer")
        .arg(target)
        .spawn()
        .map_err(|error| crate::core::error::AppError::Other(format!("打开原文件失败: {error}")))?;
    Ok(())
}

#[tauri::command]
pub async fn reveal_asset_in_folder(state: State<'_, AppState>, asset_id: String) -> AppResult<()> {
    let asset = state
        .db
        .get_asset(&asset_id)?
        .ok_or_else(|| crate::core::error::AppError::AssetNotFound(asset_id))?;
    let target = std::path::Path::new(&asset.file_path);
    if target.is_file() {
        std::process::Command::new("explorer")
            .args(["/select,", target.to_string_lossy().as_ref()])
            .spawn()
            .map_err(|error| {
                crate::core::error::AppError::Other(format!("打开所在文件夹失败: {error}"))
            })?;
    } else {
        return Err(crate::core::error::AppError::InvalidPath(
            target.to_path_buf(),
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn copy_to_clipboard(app: tauri::AppHandle, text: String) -> AppResult<()> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text(text)
        .map_err(|e| crate::core::error::AppError::Other(format!("写入剪贴板失败: {}", e)))?;
    Ok(())
}
