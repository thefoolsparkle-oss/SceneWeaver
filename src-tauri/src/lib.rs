pub mod commands;
pub mod core;
pub mod models;
pub mod providers;

pub use commands::register_commands;
pub use core::app_state::{setup_app_state, AppState};

pub fn run() {
    let builder = tauri::Builder::default();

    // Enabled only by the explicit `webdriver-e2e` build feature. Release
    // bundles never expose this test server.
    #[cfg(feature = "webdriver-e2e")]
    let builder = builder.plugin(tauri_plugin_wdio_webdriver::init());

    builder
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
