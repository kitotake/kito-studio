//! Sauvegarde automatique et récupération après crash.
//!
//! Les copies automatiques sont écrites dans `<dossier projet>/Autosave/` sous des noms
//! horodatés. Elles n'écrasent JAMAIS le fichier `.kito` enregistré manuellement : il ne
//! change que par « Enregistrer ». Après un crash, une copie plus récente que le fichier
//! est proposée à la restauration.

use super::io::write_atomic;
use crate::error::{AppError, AppResult};
use crate::timeutil;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

pub const AUTOSAVE_DIR: &str = "Autosave";

#[derive(Debug, Clone)]
pub struct AutosaveJob {
    pub project_id: String,
    pub revision: u64,
    pub dir: PathBuf,
    pub stem: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutosaveInfo {
    pub path: String,
    pub modified_at: String,
    #[serde(skip)]
    pub modified: SystemTime,
}

pub fn autosave_dir(project_file: &Path) -> PathBuf {
    project_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(AUTOSAVE_DIR)
}

pub fn stem_of(project_file: &Path) -> String {
    project_file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "projet".into())
}

/// Vrai quand une copie automatique doit être faite maintenant.
pub fn is_due(
    enabled: bool,
    dirty: bool,
    revision: u64,
    last_autosaved_revision: Option<u64>,
    last_attempt: Instant,
    interval: Duration,
    now: Instant,
) -> bool {
    enabled
        && dirty
        && last_autosaved_revision != Some(revision)
        && now.saturating_duration_since(last_attempt) >= interval
}

fn list_in(dir: &Path, stem: &str) -> Vec<AutosaveInfo> {
    let prefix = format!("{stem}-");
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<AutosaveInfo> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !(name.starts_with(&prefix) && name.ends_with(".kito")) {
                return None;
            }
            let meta = entry.metadata().ok().filter(|m| m.is_file())?;
            let modified = meta.modified().ok()?;
            Some(AutosaveInfo {
                path: entry.path().to_string_lossy().into_owned(),
                modified_at: timeutil::format_iso(timeutil::unix_secs(modified)),
                modified,
            })
        })
        .collect();
    found.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| b.path.cmp(&a.path))
    });
    found
}

/// Copies automatiques d'un projet, la plus récente d'abord.
pub fn list_autosaves(project_file: &Path) -> Vec<AutosaveInfo> {
    list_in(&autosave_dir(project_file), &stem_of(project_file))
}

/// Écrit une copie (écriture atomique) puis supprime les plus anciennes au-delà de `keep`.
pub fn write_autosave(job: &AutosaveJob, keep: usize) -> AppResult<PathBuf> {
    fs::create_dir_all(&job.dir)
        .map_err(|e| AppError::from_io("Création du dossier de sauvegarde automatique", e))?;
    let stamp = timeutil::compact_stamp(timeutil::now_unix_secs());
    let mut target = job.dir.join(format!("{}-{stamp}.kito", job.stem));
    let mut n = 1;
    while target.exists() {
        target = job.dir.join(format!("{}-{stamp}-{n}.kito", job.stem));
        n += 1;
    }
    write_atomic(&target, &job.bytes, false)?;

    for old in list_in(&job.dir, &job.stem).into_iter().skip(keep.max(1)) {
        let _ = fs::remove_file(&old.path);
    }
    Ok(target)
}

fn project_mtime(project_file: &Path) -> Option<SystemTime> {
    fs::metadata(project_file)
        .ok()
        .and_then(|m| m.modified().ok())
}

/// Copie à proposer après un arrêt brutal : la plus récente, si elle est plus récente
/// que le fichier enregistré.
pub fn find_recovery(project_file: &Path) -> Option<AutosaveInfo> {
    let newest = list_autosaves(project_file).into_iter().next()?;
    match project_mtime(project_file) {
        Some(saved) if newest.modified <= saved => None,
        _ => Some(newest),
    }
}

/// Supprime les copies plus récentes que le fichier (changements abandonnés volontairement),
/// pour qu'elles ne soient pas re-proposées au prochain démarrage.
pub fn discard_newer_than_project(project_file: &Path) -> AppResult<usize> {
    let Some(saved) = project_mtime(project_file) else {
        return Ok(0);
    };
    let mut removed = 0;
    for info in list_autosaves(project_file) {
        if info.modified > saved {
            fs::remove_file(&info.path)
                .map_err(|e| AppError::from_io("Suppression d'une copie automatique", e))?;
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;
    use std::fs::File;

    fn job(dir: &Path, bytes: &[u8]) -> AutosaveJob {
        AutosaveJob {
            project_id: "p".into(),
            revision: 1,
            dir: dir.join(AUTOSAVE_DIR),
            stem: "Film".into(),
            bytes: bytes.to_vec(),
        }
    }

    fn set_mtime(path: &Path, secs_after_epoch: u64) {
        let f = File::options().write(true).open(path).unwrap();
        f.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(secs_after_epoch))
            .unwrap();
    }

    #[test]
    fn due_only_when_enabled_dirty_changed_and_interval_elapsed() {
        let t0 = Instant::now();
        let later = t0 + Duration::from_secs(301);
        let iv = Duration::from_secs(300);
        assert!(is_due(true, true, 3, None, t0, iv, later));
        assert!(!is_due(false, true, 3, None, t0, iv, later), "désactivée");
        assert!(
            !is_due(true, false, 3, None, t0, iv, later),
            "rien à sauvegarder"
        );
        assert!(
            !is_due(true, true, 3, Some(3), t0, iv, later),
            "révision déjà sauvegardée"
        );
        assert!(
            is_due(true, true, 4, Some(3), t0, iv, later),
            "nouvelle révision"
        );
        assert!(
            !is_due(true, true, 3, None, t0, iv, t0 + Duration::from_secs(299)),
            "trop tôt"
        );
    }

    #[test]
    fn writes_unique_timestamped_copies_and_rotates() {
        let dir = TempDir::new("autosave-rotate");
        for i in 0..5u8 {
            let p = write_autosave(&job(dir.path(), &[i]), 3).unwrap();
            set_mtime(&p, 1_000 + u64::from(i));
        }
        let kept = list_in(&dir.path().join(AUTOSAVE_DIR), "Film");
        assert_eq!(kept.len(), 3);
        // Les 3 plus récentes sont conservées, de la plus récente à la plus ancienne.
        let contents: Vec<Vec<u8>> = kept.iter().map(|k| fs::read(&k.path).unwrap()).collect();
        assert_eq!(contents, vec![vec![4u8], vec![3], vec![2]]);
    }

    #[test]
    fn rotation_never_touches_other_files_or_other_projects() {
        let dir = TempDir::new("autosave-other");
        let ad = dir.path().join(AUTOSAVE_DIR);
        fs::create_dir_all(&ad).unwrap();
        fs::write(ad.join("Autre-20260101-000000.kito"), b"autre projet").unwrap();
        fs::write(ad.join("notes.txt"), b"x").unwrap();
        for i in 0..3u8 {
            write_autosave(&job(dir.path(), &[i]), 1).unwrap();
        }
        assert!(ad.join("Autre-20260101-000000.kito").exists());
        assert!(ad.join("notes.txt").exists());
        assert_eq!(list_in(&ad, "Film").len(), 1);
    }

    #[test]
    fn recovery_is_offered_only_for_copies_newer_than_the_saved_file() {
        let dir = TempDir::new("autosave-recovery");
        let project = dir.path().join("Film.kito");
        fs::write(&project, b"manuel").unwrap();
        set_mtime(&project, 5_000);
        assert!(find_recovery(&project).is_none(), "aucune copie");

        let old = write_autosave(&job(dir.path(), b"vieux"), 5).unwrap();
        set_mtime(&old, 4_000);
        assert!(
            find_recovery(&project).is_none(),
            "copie plus ancienne que la sauvegarde manuelle"
        );

        let newer = write_autosave(&job(dir.path(), b"recent"), 5).unwrap();
        set_mtime(&newer, 6_000);
        let found = find_recovery(&project).expect("copie récente proposée");
        assert_eq!(PathBuf::from(&found.path), newer);
        assert_eq!(
            fs::read(&project).unwrap(),
            b"manuel",
            "le fichier manuel n'est jamais modifié"
        );
    }

    #[test]
    fn recovery_is_offered_when_the_project_file_is_gone() {
        let dir = TempDir::new("autosave-gone");
        write_autosave(&job(dir.path(), b"x"), 5).unwrap();
        assert!(find_recovery(&dir.path().join("Film.kito")).is_some());
    }

    #[test]
    fn discarding_removes_only_newer_copies() {
        let dir = TempDir::new("autosave-discard");
        let project = dir.path().join("Film.kito");
        fs::write(&project, b"m").unwrap();
        set_mtime(&project, 5_000);
        let old = write_autosave(&job(dir.path(), b"o"), 9).unwrap();
        set_mtime(&old, 1_000);
        let new = write_autosave(&job(dir.path(), b"n"), 9).unwrap();
        set_mtime(&new, 9_000);
        assert_eq!(discard_newer_than_project(&project).unwrap(), 1);
        assert!(old.exists() && !new.exists());
        assert!(find_recovery(&project).is_none());
    }
}
