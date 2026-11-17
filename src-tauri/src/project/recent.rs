//! Liste des projets récents (`recent-projects.json` dans le dossier de données).

use super::io::write_atomic;
use super::paths::{clean, path_to_string};
use crate::error::{AppError, AppResult};
use crate::timeutil;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const MAX_RECENTS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentEntry {
    pub path: String,
    pub name: String,
    pub last_opened_at: String,
}

#[derive(Serialize, Deserialize)]
struct RecentFile {
    version: u32,
    items: Vec<RecentEntry>,
}

fn key(path: &str) -> String {
    let cleaned = path_to_string(&clean(Path::new(path)));
    if cfg!(windows) {
        cleaned.to_lowercase()
    } else {
        cleaned
    }
}

pub struct RecentStore {
    path: PathBuf,
}

impl RecentStore {
    pub fn new(dir: &Path) -> Self {
        Self {
            path: dir.join("recent-projects.json"),
        }
    }

    /// Entrées enregistrées (récentes d'abord). Un fichier absent ou illisible donne une liste vide.
    pub fn list(&self) -> Vec<RecentEntry> {
        fs::read(&self.path)
            .ok()
            .and_then(|b| serde_json::from_slice::<RecentFile>(&b).ok())
            .map(|f| f.items)
            .unwrap_or_default()
    }

    fn save(&self, items: Vec<RecentEntry>) -> AppResult<()> {
        let bytes = serde_json::to_vec_pretty(&RecentFile { version: 1, items })
            .map_err(|e| AppError::Internal(format!("Sérialisation des projets récents : {e}")))?;
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)
                .map_err(|e| AppError::from_io("Création du dossier de données", e))?;
        }
        write_atomic(&self.path, &bytes, false)
    }

    pub fn record(&self, project_path: &Path, name: &str) -> AppResult<()> {
        let path = path_to_string(project_path);
        let k = key(&path);
        let mut items: Vec<RecentEntry> = self
            .list()
            .into_iter()
            .filter(|e| key(&e.path) != k)
            .collect();
        items.insert(
            0,
            RecentEntry {
                path,
                name: name.to_string(),
                last_opened_at: timeutil::now_iso(),
            },
        );
        items.truncate(MAX_RECENTS);
        self.save(items)
    }

    pub fn remove(&self, project_path: &str) -> AppResult<()> {
        let k = key(project_path);
        self.save(
            self.list()
                .into_iter()
                .filter(|e| key(&e.path) != k)
                .collect(),
        )
    }

    pub fn clear(&self) -> AppResult<()> {
        self.save(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    #[test]
    fn records_most_recent_first_without_duplicates() {
        let dir = TempDir::new("recent");
        let store = RecentStore::new(dir.path());
        assert!(store.list().is_empty());
        store.record(Path::new("/p/a.kito"), "A").unwrap();
        store.record(Path::new("/p/b.kito"), "B").unwrap();
        store
            .record(Path::new("/p/./x/../a.kito"), "A renommé")
            .unwrap();
        let names: Vec<_> = store.list().into_iter().map(|e| e.name).collect();
        assert_eq!(names, ["A renommé", "B"]);
    }

    #[test]
    fn is_capped_and_supports_remove_and_clear() {
        let dir = TempDir::new("recent-cap");
        let store = RecentStore::new(dir.path());
        for i in 0..(MAX_RECENTS + 5) {
            store
                .record(Path::new(&format!("/p/{i}.kito")), &format!("P{i}"))
                .unwrap();
        }
        let list = store.list();
        assert_eq!(list.len(), MAX_RECENTS);
        assert_eq!(list[0].name, format!("P{}", MAX_RECENTS + 4));
        store.remove("/p/24.kito").unwrap();
        assert_eq!(store.list().len(), MAX_RECENTS - 1);
        store.clear().unwrap();
        assert!(store.list().is_empty());
    }

    #[test]
    fn corrupt_file_is_treated_as_empty_and_can_be_overwritten() {
        let dir = TempDir::new("recent-corrupt");
        fs::write(dir.path().join("recent-projects.json"), b"\xff\xfe garbage").unwrap();
        let store = RecentStore::new(dir.path());
        assert!(store.list().is_empty());
        store.record(Path::new("/p/a.kito"), "A").unwrap();
        assert_eq!(store.list().len(), 1);
    }
}
