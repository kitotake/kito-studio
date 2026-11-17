//! Préférences de l'application (fichier `preferences.json`, versionné).

use crate::error::{AppError, AppResult};
use crate::project::io::write_atomic;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const ALLOWED_INTERVALS_MINUTES: [u32; 6] = [1, 2, 5, 10, 15, 30];
pub const MAX_KEEP_COPIES: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutosavePrefs {
    pub enabled: bool,
    pub interval_minutes: u32,
    pub keep_copies: u32,
}

impl Default for AutosavePrefs {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 5,
            keep_copies: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub version: u32,
    pub autosave: AutosavePrefs,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            version: 1,
            autosave: AutosavePrefs::default(),
        }
    }
}

impl Preferences {
    pub fn validated(self) -> AppResult<Self> {
        let a = &self.autosave;
        if !ALLOWED_INTERVALS_MINUTES.contains(&a.interval_minutes) {
            return Err(AppError::InvalidInput(format!(
                "Intervalle de sauvegarde automatique invalide ({} min). Valeurs possibles : {:?}.",
                a.interval_minutes, ALLOWED_INTERVALS_MINUTES
            )));
        }
        if !(1..=MAX_KEEP_COPIES).contains(&a.keep_copies) {
            return Err(AppError::InvalidInput(format!(
                "Le nombre de copies conservées doit être compris entre 1 et {MAX_KEEP_COPIES}."
            )));
        }
        Ok(self)
    }
}

pub struct PreferencesStore {
    path: PathBuf,
}

impl PreferencesStore {
    pub fn new(dir: &Path) -> Self {
        Self {
            path: dir.join("preferences.json"),
        }
    }

    /// Préférences enregistrées, ou valeurs par défaut si le fichier est absent ou invalide.
    pub fn load(&self) -> Preferences {
        fs::read(&self.path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Preferences>(&bytes).ok())
            .and_then(|p| p.validated().ok())
            .unwrap_or_default()
    }

    pub fn save(&self, prefs: &Preferences) -> AppResult<()> {
        let bytes = serde_json::to_vec_pretty(prefs)
            .map_err(|e| AppError::Internal(format!("Sérialisation des préférences : {e}")))?;
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)
                .map_err(|e| AppError::from_io("Création du dossier de données", e))?;
        }
        write_atomic(&self.path, &bytes, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    #[test]
    fn defaults_are_valid_and_sensible() {
        let p = Preferences::default().validated().unwrap();
        assert!(p.autosave.enabled);
        assert_eq!(
            (p.autosave.interval_minutes, p.autosave.keep_copies),
            (5, 10)
        );
    }

    #[test]
    fn rejects_invalid_values() {
        let mut p = Preferences::default();
        p.autosave.interval_minutes = 3;
        assert_eq!(p.validated().unwrap_err().code(), "INVALID_INPUT");
        let mut p = Preferences::default();
        p.autosave.keep_copies = 0;
        assert!(p.validated().is_err());
        let mut p = Preferences::default();
        p.autosave.keep_copies = 101;
        assert!(p.validated().is_err());
    }

    #[test]
    fn save_then_load_round_trips_and_missing_or_corrupt_files_give_defaults() {
        let dir = TempDir::new("prefs");
        let store = PreferencesStore::new(dir.path());
        assert_eq!(store.load(), Preferences::default());
        let mut p = Preferences::default();
        p.autosave = AutosavePrefs {
            enabled: false,
            interval_minutes: 15,
            keep_copies: 3,
        };
        store.save(&p).unwrap();
        assert_eq!(store.load(), p);
        fs::write(dir.path().join("preferences.json"), b"{ pas du json").unwrap();
        assert_eq!(store.load(), Preferences::default());
        fs::write(
            dir.path().join("preferences.json"),
            br#"{"autosave":{"intervalMinutes":7}}"#,
        )
        .unwrap();
        assert_eq!(
            store.load(),
            Preferences::default(),
            "valeurs invalides ignorées"
        );
    }

    #[test]
    fn partial_files_use_defaults_for_missing_fields() {
        let dir = TempDir::new("prefs-partial");
        fs::write(
            dir.path().join("preferences.json"),
            br#"{"autosave":{"enabled":false}}"#,
        )
        .unwrap();
        let p = PreferencesStore::new(dir.path()).load();
        assert!(!p.autosave.enabled);
        assert_eq!(p.autosave.interval_minutes, 5);
    }
}
