pub mod commands;
pub mod core;
pub mod models;
pub mod providers;

pub use commands::register_commands;
pub use core::app_state::{setup_app_state, AppState};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            setup_app_state(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(register_commands())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
