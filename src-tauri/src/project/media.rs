//! Résolution portable des médias externes.
//!
//! Un média est retrouvé en essayant, dans l'ordre :
//! 1. le chemin RELATIF au dossier du projet (survit au déplacement du projet
//!    avec ses médias, y compris vers un autre ordinateur) ;
//! 2. le chemin ABSOLU d'origine (survit au déplacement du projet seul).
//!
//! Un candidat dont la taille correspond à celle enregistrée est préféré ; un fichier
//! de taille différente est signalé `Changed`. Ces chemins sont seulement lus
//! (`metadata`) : ils ne sont jamais ouverts ni exécutés par ce module.

use super::model::MediaRef;
use super::paths::{clean, from_relative, path_to_string, to_relative};
use crate::error::{AppError, AppResult};
use crate::timeutil;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaStatus {
    Ok,
    /// Fichier présent mais de taille différente de celle enregistrée.
    Changed,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolvedFrom {
    Relative,
    Absolute,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub media_id: String,
    pub status: MediaStatus,
    pub resolved_path: Option<PathBuf>,
    pub resolved_from: Option<ResolvedFrom>,
}

const MAX_SCAN_DEPTH: usize = 10;
const MAX_SCAN_ENTRIES: usize = 200_000;

pub fn resolve(project_dir: &Path, media: &MediaRef) -> Resolution {
    let mut candidates: Vec<(ResolvedFrom, PathBuf)> = Vec::new();
    if let Some(p) = media
        .relative_path
        .as_deref()
        .and_then(|rel| from_relative(project_dir, rel))
    {
        candidates.push((ResolvedFrom::Relative, p));
    }
    let absolute = Path::new(&media.path);
    if absolute.is_absolute() {
        candidates.push((ResolvedFrom::Absolute, clean(absolute)));
    }

    let mut changed: Option<(ResolvedFrom, PathBuf)> = None;
    for (from, path) in candidates {
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        if media.size.is_none_or(|expected| expected == meta.len()) {
            return Resolution {
                media_id: media.id.clone(),
                status: MediaStatus::Ok,
                resolved_path: Some(path),
                resolved_from: Some(from),
            };
        }
        changed.get_or_insert((from, path));
    }

    match changed {
        Some((from, path)) => Resolution {
            media_id: media.id.clone(),
            status: MediaStatus::Changed,
            resolved_path: Some(path),
            resolved_from: Some(from),
        },
        None => Resolution {
            media_id: media.id.clone(),
            status: MediaStatus::Missing,
            resolved_path: None,
            resolved_from: None,
        },
    }
}

pub fn resolve_all(project_dir: &Path, media: &[MediaRef]) -> Vec<Resolution> {
    media.iter().map(|m| resolve(project_dir, m)).collect()
}

fn modified_iso(meta: &fs::Metadata) -> Option<String> {
    meta.modified()
        .ok()
        .map(|t| timeutil::format_iso(timeutil::unix_secs(t)))
}

/// Crée la référence d'un média existant (utilisé par l'import et le reliage).
pub fn media_ref_from_file(
    project_dir: &Path,
    id: String,
    kind: &str,
    file: &Path,
) -> AppResult<MediaRef> {
    let absolute = clean(
        &std::path::absolute(file)
            .map_err(|e| AppError::from_io("Résolution du chemin du média", e))?,
    );
    let meta = fs::metadata(&absolute).map_err(|e| {
        AppError::from_io(format!("Lecture du média « {} »", absolute.display()), e)
    })?;
    if !meta.is_file() {
        return Err(AppError::InvalidInput(format!(
            "« {} » n'est pas un fichier.",
            absolute.display()
        )));
    }
    Ok(MediaRef {
        id,
        kind: kind.to_string(),
        path: path_to_string(&absolute),
        relative_path: to_relative(project_dir, &absolute),
        size: Some(meta.len()),
        modified_at: modified_iso(&meta),
        extra: Default::default(),
    })
}

/// Pointe un média existant vers un nouveau fichier choisi par l'utilisateur.
pub fn relink_file(project_dir: &Path, media: &mut MediaRef, file: &Path) -> AppResult<()> {
    let fresh = media_ref_from_file(project_dir, media.id.clone(), &media.kind, file)?;
    media.path = fresh.path;
    media.relative_path = fresh.relative_path;
    media.size = fresh.size;
    media.modified_at = fresh.modified_at;
    Ok(())
}

/// Recalcule `path` / `relativePath` avant une sauvegarde, pour que le projet reste
/// portable : le chemin relatif est toujours exprimé par rapport au dossier de destination.
pub fn refresh_paths(
    old_dir: &Path,
    new_dir: &Path,
    media: &mut [MediaRef],
    resolutions: &[Resolution],
) {
    for (m, resolution) in media.iter_mut().zip(resolutions) {
        let guess = resolution
            .resolved_path
            .clone()
            .or_else(|| {
                m.relative_path
                    .as_deref()
                    .and_then(|rel| from_relative(old_dir, rel))
            })
            .or_else(|| Some(PathBuf::from(&m.path)).filter(|p| p.is_absolute()));
        if let Some(abs) = guess {
            m.path = path_to_string(&clean(&abs));
            m.relative_path = to_relative(new_dir, &abs);
        }
    }
}

/// Cherche dans `folder` (récursif, borné) les fichiers manquants par nom + taille.
/// Ne relie que les correspondances non ambiguës. Retourne le nombre de médias reliés.
pub fn relink_in_folder(
    project_dir: &Path,
    media: &mut [MediaRef],
    resolutions: &[Resolution],
    folder: &Path,
) -> AppResult<usize> {
    let meta = fs::metadata(folder)
        .map_err(|e| AppError::from_io("Lecture du dossier de recherche", e))?;
    if !meta.is_dir() {
        return Err(AppError::InvalidInput(
            "Le chemin choisi n'est pas un dossier.".into(),
        ));
    }

    let mut index: HashMap<String, Vec<(PathBuf, u64)>> = HashMap::new();
    let mut stack = vec![(folder.to_path_buf(), 0usize)];
    let mut seen = 0usize;
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            seen += 1;
            if seen > MAX_SCAN_ENTRIES {
                break;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                if depth < MAX_SCAN_DEPTH {
                    stack.push((entry.path(), depth + 1));
                }
            } else if file_type.is_file() {
                if let Ok(m) = entry.metadata() {
                    let key = entry.file_name().to_string_lossy().to_lowercase();
                    index.entry(key).or_default().push((entry.path(), m.len()));
                }
            }
        }
    }

    let mut relinked = 0;
    for (m, resolution) in media.iter_mut().zip(resolutions) {
        if resolution.status == MediaStatus::Ok {
            continue;
        }
        let file_name = m
            .path
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(&m.path)
            .to_lowercase();
        let Some(candidates) = index.get(&file_name) else {
            continue;
        };
        let matching: Vec<&(PathBuf, u64)> = candidates
            .iter()
            .filter(|(_, size)| m.size.is_none_or(|expected| expected == *size))
            .collect();
        if let [(path, _)] = matching.as_slice() {
            relink_file(project_dir, m, path)?;
            relinked += 1;
        }
    }
    Ok(relinked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    fn write(path: &Path, content: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn media(project_dir: &Path, file: &Path) -> MediaRef {
        media_ref_from_file(project_dir, "m1".into(), "video", file).unwrap()
    }

    #[test]
    fn existing_media_resolves_ok() {
        let t = TempDir::new("media-ok");
        let proj = t.path().join("Projet");
        let clip = t.path().join("Rushes").join("a.mp4");
        write(&clip, b"12345");
        fs::create_dir_all(&proj).unwrap();
        let m = media(&proj, &clip);
        assert_eq!(m.relative_path.as_deref(), Some("../Rushes/a.mp4"));
        assert_eq!(m.size, Some(5));
        let r = resolve(&proj, &m);
        assert_eq!(r.status, MediaStatus::Ok);
        assert_eq!(r.resolved_from, Some(ResolvedFrom::Relative));
    }

    #[test]
    fn moving_project_and_media_together_still_resolves_via_relative_path() {
        let t = TempDir::new("media-move");
        let old_root = t.path().join("PC1");
        let proj = old_root.join("Projet");
        let clip = old_root.join("Rushes").join("a.mp4");
        write(&clip, b"abc");
        fs::create_dir_all(&proj).unwrap();
        let m = media(&proj, &clip);

        // « Autre ordinateur » : tout l'arbre est déplacé ailleurs.
        let new_root = t.path().join("PC2").join("copie");
        fs::create_dir_all(new_root.parent().unwrap()).unwrap();
        fs::rename(&old_root, &new_root).unwrap();

        assert!(
            !Path::new(&m.path).exists(),
            "l'ancien chemin absolu n'existe plus"
        );
        let r = resolve(&new_root.join("Projet"), &m);
        assert_eq!(r.status, MediaStatus::Ok);
        assert_eq!(r.resolved_from, Some(ResolvedFrom::Relative));
        assert_eq!(r.resolved_path, Some(new_root.join("Rushes").join("a.mp4")));
    }

    #[test]
    fn moving_only_the_project_falls_back_to_the_absolute_path() {
        let t = TempDir::new("media-abs");
        let proj = t.path().join("Projet");
        let clip = t.path().join("Rushes").join("a.mp4");
        write(&clip, b"abc");
        fs::create_dir_all(&proj).unwrap();
        let m = media(&proj, &clip);
        let elsewhere = t.path().join("autre").join("niveau").join("Projet");
        fs::create_dir_all(&elsewhere).unwrap();
        let r = resolve(&elsewhere, &m);
        assert_eq!(r.status, MediaStatus::Ok);
        assert_eq!(r.resolved_from, Some(ResolvedFrom::Absolute));
    }

    #[test]
    fn a_same_size_file_wins_over_a_different_one_at_the_other_candidate() {
        let t = TempDir::new("media-prefer");
        let proj = t.path().join("Projet");
        let clip = t.path().join("Rushes").join("a.mp4");
        write(&clip, b"abc");
        fs::create_dir_all(&proj).unwrap();
        let mut m = media(&proj, &clip);
        // Un AUTRE fichier (taille différente) existe au chemin relatif d'un projet déplacé.
        let moved = t.path().join("moved").join("Projet");
        write(
            &t.path().join("moved").join("Rushes").join("a.mp4"),
            b"different size!",
        );
        fs::create_dir_all(&moved).unwrap();
        m.relative_path = Some("../Rushes/a.mp4".into());
        let r = resolve(&moved, &m);
        assert_eq!(r.status, MediaStatus::Ok);
        assert_eq!(
            r.resolved_from,
            Some(ResolvedFrom::Absolute),
            "le bon fichier est celui de même taille"
        );
    }

    #[test]
    fn a_different_size_is_reported_changed_and_a_missing_file_missing() {
        let t = TempDir::new("media-changed");
        let proj = t.path().join("Projet");
        let clip = t.path().join("a.mp4");
        write(&clip, b"abc");
        fs::create_dir_all(&proj).unwrap();
        let m = media(&proj, &clip);
        write(&clip, b"abcdef");
        assert_eq!(resolve(&proj, &m).status, MediaStatus::Changed);
        fs::remove_file(&clip).unwrap();
        let r = resolve(&proj, &m);
        assert_eq!(r.status, MediaStatus::Missing);
        assert!(r.resolved_path.is_none());
    }

    #[test]
    fn a_directory_is_not_a_media_file() {
        let t = TempDir::new("media-dir");
        let proj = t.path().join("Projet");
        fs::create_dir_all(&proj).unwrap();
        let mut m = media_ref_from_file(&proj, "m".into(), "video", &{
            let f = t.path().join("f.mp4");
            write(&f, b"x");
            f
        })
        .unwrap();
        m.path = path_to_string(t.path());
        m.relative_path = None;
        m.size = None;
        assert_eq!(resolve(&proj, &m).status, MediaStatus::Missing);
    }

    #[test]
    fn hostile_relative_paths_are_only_statted() {
        let t = TempDir::new("media-hostile");
        let proj = t.path().join("Projet");
        fs::create_dir_all(&proj).unwrap();
        let mut m = MediaRef {
            id: "m".into(),
            kind: "video".into(),
            path: "relative/not-absolute.exe".into(),
            relative_path: Some("/etc/passwd".into()),
            size: None,
            modified_at: None,
            extra: Default::default(),
        };
        assert_eq!(
            resolve(&proj, &m).status,
            MediaStatus::Missing,
            "un « relatif » absolu est ignoré"
        );
        m.relative_path = Some("a\0b".into());
        assert_eq!(resolve(&proj, &m).status, MediaStatus::Missing);
    }

    #[test]
    fn refresh_paths_keeps_the_project_portable_after_save_as() {
        let t = TempDir::new("media-refresh");
        let proj = t.path().join("A").join("Projet");
        let clip = t.path().join("Rushes").join("a.mp4");
        write(&clip, b"abc");
        fs::create_dir_all(&proj).unwrap();
        let mut list = vec![media(&proj, &clip)];
        let res = resolve_all(&proj, &list);
        let new_dir = t.path().join("B").join("profond").join("Projet");
        refresh_paths(&proj, &new_dir, &mut list, &res);
        assert_eq!(
            list[0].relative_path.as_deref(),
            Some("../../../Rushes/a.mp4")
        );
        assert_eq!(list[0].path, path_to_string(&clip));
    }

    #[test]
    fn refresh_paths_rebases_missing_media_from_the_current_location() {
        let t = TempDir::new("media-refresh-missing");
        let proj = t.path().join("Projet");
        fs::create_dir_all(&proj).unwrap();
        let mut list = vec![MediaRef {
            id: "m".into(),
            kind: "video".into(),
            path: "/ancien/pc/Rushes/a.mp4".into(),
            relative_path: Some("../Rushes/a.mp4".into()),
            size: Some(3),
            modified_at: None,
            extra: Default::default(),
        }];
        let res = resolve_all(&proj, &list);
        assert_eq!(res[0].status, MediaStatus::Missing);
        refresh_paths(&proj, &proj, &mut list, &res);
        assert_eq!(
            list[0].path,
            path_to_string(&t.path().join("Rushes").join("a.mp4"))
        );
        assert_eq!(list[0].relative_path.as_deref(), Some("../Rushes/a.mp4"));
    }

    #[test]
    fn relink_file_updates_paths_and_size() {
        let t = TempDir::new("media-relink");
        let proj = t.path().join("Projet");
        fs::create_dir_all(&proj).unwrap();
        let old = t.path().join("old.mp4");
        write(&old, b"abc");
        let mut m = media(&proj, &old);
        let new = t.path().join("nouveau").join("old.mp4");
        write(&new, b"abcdefgh");
        relink_file(&proj, &mut m, &new).unwrap();
        assert_eq!(m.size, Some(8));
        assert_eq!(m.relative_path.as_deref(), Some("../nouveau/old.mp4"));
        assert_eq!(resolve(&proj, &m).status, MediaStatus::Ok);
        assert_eq!(
            relink_file(&proj, &mut m, &t.path().join("absent.mp4"))
                .unwrap_err()
                .code(),
            "NOT_FOUND"
        );
    }

    #[test]
    fn relink_in_folder_links_unique_matches_only() {
        let t = TempDir::new("media-folder");
        let proj = t.path().join("Projet");
        fs::create_dir_all(&proj).unwrap();
        let mk = |id: &str, name: &str, size: u64| MediaRef {
            id: id.into(),
            kind: "video".into(),
            path: format!("/disparu/{name}"),
            relative_path: None,
            size: Some(size),
            modified_at: None,
            extra: Default::default(),
        };
        let mut list = vec![
            mk("unique", "Plan1.MP4", 3),
            mk("ambigu", "plan2.mp4", 4),
            mk("absent", "plan3.mp4", 5),
            mk("mauvaise-taille", "plan4.mp4", 99),
        ];
        let found = t.path().join("Nouveaux");
        write(&found.join("sous").join("plan1.mp4"), b"abc");
        write(&found.join("plan2.mp4"), b"abcd");
        write(&found.join("copie").join("plan2.mp4"), b"abcd");
        write(&found.join("plan4.mp4"), b"abcd");
        let res = resolve_all(&proj, &list);
        let n = relink_in_folder(&proj, &mut list, &res, &found).unwrap();
        assert_eq!(n, 1);
        assert_eq!(resolve(&proj, &list[0]).status, MediaStatus::Ok);
        for m in &list[1..] {
            assert_eq!(resolve(&proj, m).status, MediaStatus::Missing, "{}", m.id);
        }
        assert_eq!(
            relink_in_folder(&proj, &mut list, &res, &found.join("plan2.mp4"))
                .unwrap_err()
                .code(),
            "INVALID_INPUT"
        );
    }
}
