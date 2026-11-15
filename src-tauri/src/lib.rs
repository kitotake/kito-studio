//! Kito Studio — cœur natif.
//!
//! Organisation :
//! - `commands` : frontière Tauri (fine, sans logique métier).
//! - `error`    : `AppError` / `AppResult`, contrat d'erreur partagé avec l'interface.
//! - `media`, `system` : modules métier indépendants de Tauri, testables seuls.
//!
//! Les futurs modules (project, timeline, playback, render, export…) suivent le même
//! schéma : un module métier + une famille de commandes dans `commands/`.

pub mod commands;
pub mod error;
pub mod media;
pub mod system;

pub use error::{AppError, AppResult};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::system::get_runtime_info,
            commands::system::check_media_tools
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors du démarrage de Kito Studio");
}
