use std::path::Path;
use std::sync::Arc;

use tauri::Manager;

use crate::core::cache::CacheManager;
use crate::core::db::Database;
use crate::core::error::AppResult;
use crate::core::job_queue::JobQueue;

pub struct AppState {
    pub db: Arc<Database>,
    pub cache: Arc<CacheManager>,
    pub job_queue: Arc<JobQueue>,
}

impl AppState {
    pub fn new(app_data_dir: &Path, app_handle: tauri::AppHandle) -> AppResult<Self> {
        std::fs::create_dir_all(app_data_dir)?;
        let db_path = app_data_dir.join("sceneweaver.db");
        let db = Arc::new(Database::new(db_path));
        db.init()?;

        let cache_root = app_data_dir.join("cache");
        let cache = Arc::new(CacheManager::new(cache_root));
        cache.ensure_dirs()?;

        let job_queue = Arc::new(JobQueue::new(
            Arc::clone(&db),
            Arc::clone(&cache),
            app_handle,
        ));
        job_queue.recover()?;

        Ok(Self {
            db,
            cache,
            job_queue,
        })
    }
}

pub fn setup_app_state(app: &mut tauri::App) -> AppResult<()> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| crate::core::error::AppError::Other(format!("获取数据目录失败: {}", e)))?;
    let app_handle = app.app_handle().clone();
    let state = AppState::new(&app_data_dir, app_handle)?;
    app.manage(state);
    Ok(())
}
