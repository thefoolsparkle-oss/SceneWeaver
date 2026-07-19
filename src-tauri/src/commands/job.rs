use tauri::State;

use crate::core::app_state::AppState;
use crate::core::error::AppResult;
use crate::models::Job;

#[tauri::command]
pub async fn list_jobs(state: State<'_, AppState>) -> AppResult<Vec<Job>> {
    state.db.list_jobs()
}

#[tauri::command]
pub async fn pause_job(state: State<'_, AppState>, job_id: String) -> AppResult<Job> {
    state.job_queue.pause_job(&job_id)
}

#[tauri::command]
pub async fn resume_job(state: State<'_, AppState>, job_id: String) -> AppResult<Job> {
    state.job_queue.resume_job(&job_id)
}

#[tauri::command]
pub async fn cancel_job(state: State<'_, AppState>, job_id: String) -> AppResult<Job> {
    state.job_queue.cancel_job(&job_id)
}

#[tauri::command]
pub async fn retry_job(state: State<'_, AppState>, job_id: String) -> AppResult<Job> {
    state.job_queue.retry_job(&job_id)
}
