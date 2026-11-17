//! Frontière Tauri : une commande = une fonction fine qui valide l'entrée,
//! délègue à un module métier et retourne un `AppResult`. Aucune logique métier ici.

pub mod autosave;
pub mod preferences;
pub mod project;
pub mod system;
