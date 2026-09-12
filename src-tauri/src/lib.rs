pub mod coach;
pub mod commands;
pub mod error;
pub mod models;
pub mod replay;
pub mod store;

use commands::AppState;
use store::Store;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      #[cfg(debug_assertions)]
      app.handle().plugin(
        tauri_plugin_log::Builder::default()
          .level(log::LevelFilter::Info)
          .build(),
      )?;
      app.handle().plugin(tauri_plugin_dialog::init())?;
      app.handle().plugin(tauri_plugin_fs::init())?;
      app.handle().plugin(tauri_plugin_shell::init())?;

      let data_dir = app.path().app_data_dir()?;
      log::info!("Datos de Maestro RL en: {}", data_dir.display());
      let store = Store::load(&data_dir)?;
      app.manage(AppState { store });
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::get_settings,
      commands::save_settings,
      commands::parse_replay,
      commands::analyze_replay,
      commands::reanalyze_entry,
      commands::list_entries,
      commands::get_entry,
      commands::delete_entry,
      commands::clear_history,
      commands::get_progress,
      commands::test_connection,
      commands::recent_replays,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}