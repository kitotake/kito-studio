//! Lecture validée et écriture ATOMIQUE des fichiers `.kito`.

use super::migrate::{self, MigrationReport};
use super::model::KitoDocument;
use super::validate;
use crate::error::{AppError, AppResult};
use serde_json::Value;
#[cfg(unix)]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU32, Ordering};

pub const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;
static TMP_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub struct LoadedDocument {
    pub doc: KitoDocument,
    pub report: MigrationReport,
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

pub fn read_document(path: &Path) -> AppResult<LoadedDocument> {
    read_document_limited(path, MAX_FILE_BYTES)
}

pub(crate) fn read_document_limited(path: &Path, max_bytes: u64) -> AppResult<LoadedDocument> {
    let ctx = format!("Lecture de « {} »", display_name(path));
    let meta = fs::metadata(path).map_err(|e| AppError::from_io(&ctx, e))?;
    if !meta.is_file() {
        return Err(AppError::InvalidInput(format!(
            "« {} » n'est pas un fichier.",
            display_name(path)
        )));
    }
    if meta.len() > max_bytes {
        return Err(AppError::InvalidFormat(format!(
            "Le fichier est trop volumineux pour un projet ({} Mo maximum).",
            max_bytes / (1024 * 1024)
        )));
    }
    let bytes = fs::read(path).map_err(|e| AppError::from_io(&ctx, e))?;
    parse_document(&bytes)
}

/// Analyse, migre et valide le contenu d'un fichier `.kito`.
pub fn parse_document(bytes: &[u8]) -> AppResult<LoadedDocument> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        AppError::InvalidFormat("Le fichier n'est pas un texte UTF-8 valide.".into())
    })?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut value: Value = serde_json::from_str(text).map_err(|e| {
        AppError::InvalidFormat(format!(
            "Fichier illisible ou incomplet (JSON invalide, ligne {}, colonne {}).",
            e.line(),
            e.column()
        ))
    })?;
    let report = migrate::migrate(&mut value)?;
    let doc: KitoDocument = serde_json::from_value(value)
        .map_err(|e| AppError::InvalidFormat(format!("Structure du projet invalide : {e}.")))?;
    validate::validate(&doc)?;
    Ok(LoadedDocument { doc, report })
}

pub fn serialize_document(doc: &KitoDocument) -> AppResult<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(doc)
        .map_err(|e| AppError::Internal(format!("Sérialisation du projet impossible : {e}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// `Projet.kito` → `Projet.kito.bak`
pub fn backup_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(".bak");
    path.with_file_name(name)
}

/// Écrit `bytes` dans `path` sans jamais laisser un fichier à moitié écrit :
/// écriture dans un fichier temporaire voisin, `fsync`, copie de secours de l'ancienne
/// version (`.bak`), puis renommage atomique par-dessus la cible.
/// En cas d'échec, la cible d'origine est intacte et le fichier temporaire supprimé.
pub fn write_atomic(path: &Path, bytes: &[u8], keep_backup: bool) -> AppResult<()> {
    write_atomic_with(path, bytes, keep_backup, || Ok(()))
}

pub(crate) fn write_atomic_with(
    path: &Path,
    bytes: &[u8],
    keep_backup: bool,
    before_rename: impl FnOnce() -> io::Result<()>,
) -> AppResult<()> {
    let name = path
        .file_name()
        .ok_or_else(|| AppError::InvalidInput("Chemin de fichier invalide.".into()))?
        .to_string_lossy()
        .into_owned();
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let ctx = format!("Écriture de « {name} »");
    let tmp = dir.join(format!(
        ".{name}.{}.{}.tmp",
        process::id(),
        TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));

    let result = (|| -> AppResult<()> {
        let io_err = |e: io::Error| AppError::from_io(&ctx, e);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(io_err)?;
        file.write_all(bytes).map_err(io_err)?;
        file.sync_all().map_err(io_err)?;
        drop(file);
        before_rename().map_err(io_err)?;
        if keep_backup && path.is_file() {
            fs::copy(path, backup_path(path))
                .map_err(|e| AppError::from_io(format!("Copie de secours de « {name} »"), e))?;
        }
        fs::rename(&tmp, path).map_err(io_err)?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    } else {
        #[cfg(unix)]
        if let Ok(dir_handle) = File::open(dir) {
            let _ = dir_handle.sync_all();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::model::{NewDocument, CURRENT_FORMAT_VERSION};
    use crate::testutil::TempDir;
    use std::fs;

    fn doc() -> KitoDocument {
        KitoDocument::new(&NewDocument {
            name: "Film".into(),
            width: 1920,
            height: 1080,
            frame_rate_num: 25,
            frame_rate_den: 1,
            sample_rate: 48_000,
        })
    }

    fn temp_files(dir: &Path) -> Vec<String> {
        fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect()
    }

    #[test]
    fn write_then_read_gives_the_same_document() {
        let dir = TempDir::new("io-roundtrip");
        let path = dir.path().join("Film.kito");
        let original = doc();
        write_atomic(&path, &serialize_document(&original).unwrap(), true).unwrap();
        let loaded = read_document(&path).unwrap();
        assert_eq!(loaded.doc, original);
        assert!(!loaded.report.migrated);
        assert_eq!(loaded.report.from_version, CURRENT_FORMAT_VERSION);
        assert!(temp_files(dir.path()).is_empty());
    }

    #[test]
    fn overwriting_keeps_the_previous_version_as_bak() {
        let dir = TempDir::new("io-bak");
        let path = dir.path().join("a.kito");
        write_atomic(&path, b"version 1", true).unwrap();
        assert!(
            !backup_path(&path).exists(),
            "pas de .bak à la première écriture"
        );
        write_atomic(&path, b"version 2", true).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"version 2");
        assert_eq!(fs::read(backup_path(&path)).unwrap(), b"version 1");
        write_atomic(&path, b"version 3", false).unwrap();
        assert_eq!(
            fs::read(backup_path(&path)).unwrap(),
            b"version 1",
            "sans keep_backup le .bak n'est pas touché"
        );
    }

    #[test]
    fn a_failure_before_the_rename_leaves_the_original_intact() {
        let dir = TempDir::new("io-fail");
        let path = dir.path().join("a.kito");
        write_atomic(&path, b"original", true).unwrap();
        let err = write_atomic_with(&path, b"nouveau contenu", true, || {
            Err(io::Error::from(io::ErrorKind::StorageFull))
        })
        .unwrap_err();
        assert_eq!(err.code(), "DISK_FULL");
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert!(
            temp_files(dir.path()).is_empty(),
            "le fichier temporaire est nettoyé"
        );
        assert!(
            !backup_path(&path).exists(),
            "le .bak n'est pas créé avant le point de non-retour"
        );
    }

    #[test]
    fn a_failing_rename_is_reported_and_cleaned_up() {
        let dir = TempDir::new("io-dirtarget");
        let target = dir.path().join("cible.kito");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("x"), b"x").unwrap();
        assert!(write_atomic(&target, b"data", false).is_err());
        assert!(target.is_dir() && target.join("x").exists());
        assert!(temp_files(dir.path()).is_empty());
    }

    #[test]
    fn writing_into_a_missing_folder_is_not_found() {
        let dir = TempDir::new("io-missing");
        let err = write_atomic(&dir.path().join("nope").join("a.kito"), b"x", true).unwrap_err();
        assert_eq!(err.code(), "NOT_FOUND");
    }

    #[test]
    fn truncated_or_corrupted_files_are_rejected_with_invalid_format() {
        let dir = TempDir::new("io-corrupt");
        let full = serialize_document(&doc()).unwrap();
        for (label, bytes) in [
            ("tronqué", full[..full.len() / 2].to_vec()),
            ("vide", Vec::new()),
            ("binaire", vec![0xff, 0xfe, 0x00, 0x01]),
            ("json non projet", br#"{"hello":"world"}"#.to_vec()),
            ("tableau", b"[1,2,3]".to_vec()),
        ] {
            let path = dir.path().join(format!("{label}.kito"));
            fs::write(&path, bytes).unwrap();
            let err = read_document(&path).expect_err(label);
            assert_eq!(err.code(), "INVALID_FORMAT", "{label} : {err}");
        }
    }

    #[test]
    fn missing_file_is_not_found_and_directory_is_invalid_input() {
        let dir = TempDir::new("io-notfound");
        assert_eq!(
            read_document(&dir.path().join("x.kito"))
                .unwrap_err()
                .code(),
            "NOT_FOUND"
        );
        assert_eq!(
            read_document(dir.path()).unwrap_err().code(),
            "INVALID_INPUT"
        );
    }

    #[test]
    fn utf8_bom_is_tolerated() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend(serialize_document(&doc()).unwrap());
        parse_document(&bytes).unwrap();
    }

    #[test]
    fn oversized_files_are_refused_before_being_read() {
        let dir = TempDir::new("io-big");
        let path = dir.path().join("big.kito");
        fs::write(&path, vec![b' '; 2048]).unwrap();
        let err = read_document_limited(&path, 1024).unwrap_err();
        assert_eq!(err.code(), "INVALID_FORMAT");
        assert!(err.to_string().contains("volumineux"));
    }

    #[test]
    fn incompatible_future_file_is_refused_and_left_untouched() {
        let dir = TempDir::new("io-future");
        let path = dir.path().join("future.kito");
        let mut v = serde_json::to_value(doc()).unwrap();
        v["formatVersion"] = 9.into();
        v["minReaderVersion"] = 9.into();
        let content = serde_json::to_vec_pretty(&v).unwrap();
        fs::write(&path, &content).unwrap();
        assert_eq!(
            read_document(&path).unwrap_err().code(),
            "INCOMPATIBLE_VERSION"
        );
        assert_eq!(fs::read(&path).unwrap(), content);
    }
}
