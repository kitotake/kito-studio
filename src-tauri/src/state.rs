//! État géré par Tauri.

use crate::preferences::Preferences;
use crate::project::session::Session;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// Projet actuellement ouvert (au plus un).
#[derive(Default)]
pub struct ProjectState {
    pub session: Mutex<Option<Session>>,
}

/// Dossier de données de l'application (préférences, projets récents).
pub struct AppStorage {
    pub dir: PathBuf,
}

pub struct PreferencesState(pub Mutex<Preferences>);

/// Verrou tolérant à l'empoisonnement : un thread en panne ne doit pas bloquer l'application.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
