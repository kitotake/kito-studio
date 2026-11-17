//! Kito Studio — cœur natif.
//!
//! Organisation :
//! - `commands` : frontière Tauri (fine, sans logique métier).
//! - `error`    : `AppError` / `AppResult`, contrat d'erreur partagé avec l'interface.
//! - `project`, `media`, `system`, `preferences` : modules métier indépendants de Tauri,
//!   testables seuls.
//! - `state`    : état géré par Tauri (projet ouvert, préférences, dossier de données).
//!
//! Les futurs modules (timeline, playback, render, export…) suivent le même schéma :
//! un module métier + une famille de commandes dans `commands/`.

pub mod commands;
pub mod error;
pub mod media;
pub mod preferences;
pub mod project;
pub mod state;
pub mod system;
pub mod timeutil;

#[cfg(test)]
pub(crate) mod testutil;

pub use error::{AppError, AppResult};

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let prefs = preferences::PreferencesStore::new(&dir).load();
            app.manage(state::AppStorage { dir });
            app.manage(state::ProjectState::default());
            app.manage(state::PreferencesState(Mutex::new(prefs)));
            commands::autosave::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::get_runtime_info,
            commands::system::check_media_tools,
            commands::project::create_project,
            commands::project::open_project,
            commands::project::list_autosaves,
            commands::project::restore_autosave,
            commands::project::get_current_project,
            commands::project::save_project,
            commands::project::save_project_as,
            commands::project::update_project_settings,
            commands::project::close_project,
            commands::project::relink_media,
            commands::project::relink_media_in_folder,
            commands::project::get_recent_projects,
            commands::project::remove_recent_project,
            commands::project::clear_recent_projects,
            commands::preferences::get_preferences,
            commands::preferences::set_preferences
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors du démarrage de Kito Studio");
}
