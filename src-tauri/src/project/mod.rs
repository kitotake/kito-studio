//! Module Project : format `.kito`, session de travail, sauvegardes et récents.
//!
//! Aucune dépendance à Tauri ici : tout est testable sans fenêtre.

pub mod autosave;
pub mod io;
pub mod media;
pub mod migrate;
pub mod model;
pub mod names;
pub mod paths;
pub mod recent;
pub mod session;
pub mod snapshot;
pub mod validate;
