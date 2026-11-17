//! Session de travail : le projet ouvert vit ici (côté Rust), l'interface n'en reçoit
//! que des instantanés et envoie des commandes.

use super::autosave::{self, AutosaveJob};
use super::io::{self, backup_path};
use super::media::{self, Resolution};
use super::model::*;
use super::names::validate_project_name;
use super::paths::{clean, path_to_string};
use super::snapshot::{MediaView, ProjectSnapshot, RecoveryOffer};
use super::validate;
use crate::error::{AppError, AppResult};
use crate::preferences::AutosavePrefs;
use crate::timeutil;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct NewProjectParams {
    pub name: String,
    pub parent_dir: PathBuf,
    pub width: u32,
    pub height: u32,
    pub frame_rate_num: u32,
    pub frame_rate_den: u32,
    pub sample_rate: u32,
}

#[derive(Debug, Clone)]
pub struct SettingsUpdate {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate_num: u32,
    pub frame_rate_den: u32,
    pub sample_rate: u32,
}

#[derive(Debug)]
pub struct Session {
    doc: KitoDocument,
    path: PathBuf,
    revision: u64,
    saved_revision: u64,
    last_saved_at: Option<String>,
    resolutions: Vec<Resolution>,
    migrated_from: Option<u32>,
    premigration_backup_pending: bool,
    last_autosaved_revision: Option<u64>,
    last_autosave_attempt: Instant,
}

fn absolute_file(path: &Path) -> AppResult<PathBuf> {
    std::path::absolute(path)
        .map(|p| clean(&p))
        .map_err(|e| AppError::from_io("Résolution du chemin du projet", e))
}

fn check_kito_extension(path: &Path) -> AppResult<()> {
    let ok = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("kito"));
    if ok {
        Ok(())
    } else {
        Err(AppError::InvalidInput(
            "Le fichier de projet doit avoir l'extension .kito.".into(),
        ))
    }
}

impl Session {
    fn build(doc: KitoDocument, path: PathBuf, revision: u64, saved_revision: u64) -> Self {
        let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let resolutions = media::resolve_all(&dir, &doc.media);
        Self {
            doc,
            path,
            revision,
            saved_revision,
            last_saved_at: None,
            resolutions,
            migrated_from: None,
            premigration_backup_pending: false,
            last_autosaved_revision: None,
            last_autosave_attempt: Instant::now(),
        }
    }

    pub fn dir(&self) -> PathBuf {
        self.path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.doc.project.name
    }

    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision
    }

    pub fn project_id(&self) -> &str {
        &self.doc.project.id
    }

    /// Crée `<parent>/<nom>/<nom>.kito` et l'enregistre immédiatement.
    pub fn create(params: &NewProjectParams) -> AppResult<Self> {
        let name = validate_project_name(&params.name)?;
        validate::validate_video(
            params.width,
            params.height,
            params.frame_rate_num,
            params.frame_rate_den,
        )?;
        validate::validate_sample_rate(params.sample_rate)?;

        if !params.parent_dir.is_absolute() {
            return Err(AppError::InvalidInput(
                "L'emplacement du projet doit être un chemin absolu.".into(),
            ));
        }
        let meta = fs::metadata(&params.parent_dir)
            .map_err(|e| AppError::from_io("Lecture de l'emplacement choisi", e))?;
        if !meta.is_dir() {
            return Err(AppError::InvalidInput(
                "L'emplacement choisi n'est pas un dossier.".into(),
            ));
        }

        let project_dir = clean(&params.parent_dir.join(&name));
        fs::create_dir(&project_dir).map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => AppError::AlreadyExists(format!(
                "Un dossier « {name} » existe déjà à cet emplacement. Choisissez un autre nom ou un autre emplacement."
            )),
            _ => AppError::from_io(format!("Création du dossier « {name} »"), e),
        })?;

        let file = project_dir.join(format!("{name}.kito"));
        let doc = KitoDocument::new(&NewDocument {
            name,
            width: params.width,
            height: params.height,
            frame_rate_num: params.frame_rate_num,
            frame_rate_den: params.frame_rate_den,
            sample_rate: params.sample_rate,
        });
        let written = validate::validate(&doc)
            .and_then(|_| io::serialize_document(&doc))
            .and_then(|bytes| io::write_atomic(&file, &bytes, false));
        if let Err(e) = written {
            let _ = fs::remove_dir(&project_dir); // ne laisse pas de dossier vide orphelin
            return Err(e);
        }

        let mut session = Self::build(doc, file, 0, 0);
        session.last_saved_at = Some(timeutil::now_iso());
        Ok(session)
    }

    pub fn open(path: &Path) -> AppResult<Self> {
        let path = absolute_file(path)?;
        let loaded = io::read_document(&path)?;
        let mut session = Self::build(loaded.doc, path, 0, 0);
        if loaded.report.migrated {
            session.migrated_from = Some(loaded.report.from_version);
            session.premigration_backup_pending = true;
        }
        Ok(session)
    }

    /// Charge une copie automatique comme état de travail NON enregistré du projet.
    /// Le fichier `.kito` d'origine n'est pas touché tant que l'utilisateur n'enregistre pas.
    pub fn restore_from_autosave(project_path: &Path, autosave_path: &Path) -> AppResult<Self> {
        let project_path = absolute_file(project_path)?;
        let autosave_path = absolute_file(autosave_path)?;
        let expected_dir = clean(&autosave::autosave_dir(&project_path));
        if autosave_path.parent().map(clean).as_deref() != Some(expected_dir.as_path()) {
            return Err(AppError::InvalidInput(
                "Cette copie ne fait pas partie des sauvegardes automatiques de ce projet.".into(),
            ));
        }
        let loaded = io::read_document(&autosave_path)?;
        let mut session = Self::build(loaded.doc, project_path, 1, 0);
        if loaded.report.migrated {
            session.migrated_from = Some(loaded.report.from_version);
        }
        Ok(session)
    }

    fn touch(&mut self) {
        self.doc.project.modified_at = timeutil::now_iso();
        self.revision += 1;
    }

    fn refresh_resolutions(&mut self) {
        self.resolutions = media::resolve_all(&self.dir(), &self.doc.media);
    }

    pub fn update_settings(&mut self, update: &SettingsUpdate) -> AppResult<()> {
        let name = validate_project_name(&update.name)?;
        let mut next = self.doc.clone();
        next.project.name = name;
        next.settings.video.width = update.width;
        next.settings.video.height = update.height;
        next.settings.video.frame_rate.num = update.frame_rate_num;
        next.settings.video.frame_rate.den = update.frame_rate_den;
        next.settings.audio.sample_rate = update.sample_rate;
        validate::validate(&next)?;
        if next == self.doc {
            return Ok(());
        }
        self.doc = next;
        self.touch();
        Ok(())
    }

    fn prepare_save(&self, new_dir: &Path) -> AppResult<(KitoDocument, Vec<u8>)> {
        let mut candidate = self.doc.clone();
        media::refresh_paths(
            &self.dir(),
            new_dir,
            &mut candidate.media,
            &self.resolutions,
        );
        candidate.saved_by.app = "Kito Studio".to_string();
        candidate.saved_by.version = env!("CARGO_PKG_VERSION").to_string();
        validate::validate(&candidate)?;
        let bytes = io::serialize_document(&candidate)?;
        Ok((candidate, bytes))
    }

    fn mark_saved(&mut self) {
        self.saved_revision = self.revision;
        self.last_saved_at = Some(timeutil::now_iso());
        self.premigration_backup_pending = false;
        self.refresh_resolutions();
    }

    /// Enregistre sur place, de façon atomique, en gardant l'ancienne version en `.bak`.
    pub fn save(&mut self) -> AppResult<()> {
        let (candidate, bytes) = self.prepare_save(&self.dir())?;

        if let (true, Some(from), true) = (
            self.premigration_backup_pending,
            self.migrated_from,
            self.path.is_file(),
        ) {
            let name = self
                .path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let legacy = self.path.with_file_name(format!("{name}.v{from}.bak"));
            if !legacy.exists() {
                fs::copy(&self.path, &legacy)
                    .map_err(|e| AppError::from_io("Copie de l'ancienne version du projet", e))?;
            }
        }

        io::write_atomic(&self.path, &bytes, true)?;
        self.doc = candidate;
        self.mark_saved();
        Ok(())
    }

    pub fn save_as(&mut self, new_path: &Path) -> AppResult<()> {
        let new_path = absolute_file(new_path)?;
        check_kito_extension(&new_path)?;
        let new_dir = new_path.parent().map(Path::to_path_buf).unwrap_or_default();
        let meta = fs::metadata(&new_dir)
            .map_err(|e| AppError::from_io("Lecture du dossier de destination", e))?;
        if !meta.is_dir() {
            return Err(AppError::InvalidInput(
                "Le dossier de destination n'est pas valide.".into(),
            ));
        }
        let (candidate, bytes) = self.prepare_save(&new_dir)?;
        io::write_atomic(&new_path, &bytes, true)?;
        self.doc = candidate;
        self.path = new_path;
        self.migrated_from = None;
        self.last_autosaved_revision = None;
        self.mark_saved();
        Ok(())
    }

    pub fn relink(&mut self, media_id: &str, file: &Path) -> AppResult<()> {
        let dir = self.dir();
        let index = self
            .doc
            .media
            .iter()
            .position(|m| m.id == media_id)
            .ok_or_else(|| {
                AppError::NotFound(format!("Média « {media_id} » inconnu dans ce projet."))
            })?;
        media::relink_file(&dir, &mut self.doc.media[index], file)?;
        self.touch();
        self.refresh_resolutions();
        Ok(())
    }

    pub fn relink_in_folder(&mut self, folder: &Path) -> AppResult<usize> {
        let dir = self.dir();
        let count = media::relink_in_folder(&dir, &mut self.doc.media, &self.resolutions, folder)?;
        if count > 0 {
            self.touch();
        }
        self.refresh_resolutions();
        Ok(count)
    }

    /// Prépare une copie automatique si elle est due (sérialise sous le verrou, écrit hors verrou).
    pub fn prepare_autosave(
        &mut self,
        prefs: &AutosavePrefs,
        now: Instant,
    ) -> AppResult<Option<AutosaveJob>> {
        let due = autosave::is_due(
            prefs.enabled,
            self.is_dirty(),
            self.revision,
            self.last_autosaved_revision,
            self.last_autosave_attempt,
            Duration::from_secs(u64::from(prefs.interval_minutes) * 60),
            now,
        );
        if !due {
            return Ok(None);
        }
        self.last_autosave_attempt = now; // évite de réessayer chaque seconde en cas d'échec
        let bytes = io::serialize_document(&self.doc)?;
        Ok(Some(AutosaveJob {
            project_id: self.doc.project.id.clone(),
            revision: self.revision,
            dir: autosave::autosave_dir(&self.path),
            stem: autosave::stem_of(&self.path),
            bytes,
        }))
    }

    pub fn mark_autosaved(&mut self, project_id: &str, revision: u64) {
        if self.doc.project.id == project_id {
            self.last_autosaved_revision = Some(revision);
        }
    }

    pub fn recovery_offer(&self) -> Option<RecoveryOffer> {
        autosave::find_recovery(&self.path).map(|info| RecoveryOffer {
            autosave_path: info.path,
            autosaved_at: info.modified_at,
        })
    }

    pub fn snapshot(&self) -> ProjectSnapshot {
        let media = self
            .doc
            .media
            .iter()
            .zip(&self.resolutions)
            .map(|(m, r)| MediaView {
                id: m.id.clone(),
                kind: m.kind.clone(),
                file_name: m
                    .path
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or(&m.path)
                    .to_string(),
                path: m.path.clone(),
                status: r.status,
                resolved_path: r.resolved_path.as_deref().map(path_to_string),
            })
            .collect();
        let v = &self.doc.settings.video;
        ProjectSnapshot {
            id: self.doc.project.id.clone(),
            name: self.doc.project.name.clone(),
            path: path_to_string(&self.path),
            directory: path_to_string(&self.dir()),
            width: v.width,
            height: v.height,
            frame_rate_num: v.frame_rate.num,
            frame_rate_den: v.frame_rate.den,
            sample_rate: self.doc.settings.audio.sample_rate,
            created_at: self.doc.project.created_at.clone(),
            modified_at: self.doc.project.modified_at.clone(),
            dirty: self.is_dirty(),
            last_saved_at: self.last_saved_at.clone(),
            format_version: self.doc.format_version,
            migrated_from: self.migrated_from,
            newer_format: self.doc.format_version > CURRENT_FORMAT_VERSION,
            media,
        }
    }

    /// Supprime les copies automatiques plus récentes que le fichier (modifications abandonnées).
    pub fn discard_unsaved_autosaves(&self) -> AppResult<usize> {
        autosave::discard_newer_than_project(&self.path)
    }

    pub fn backup_file(&self) -> PathBuf {
        backup_path(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::media::MediaStatus;
    use crate::testutil::{document_with_unknown_fields, TempDir};
    use serde_json::Value;

    pub fn params(parent: &Path, name: &str) -> NewProjectParams {
        NewProjectParams {
            name: name.into(),
            parent_dir: parent.to_path_buf(),
            width: 1920,
            height: 1080,
            frame_rate_num: 30,
            frame_rate_den: 1,
            sample_rate: 48_000,
        }
    }

    fn update_from(s: &Session) -> SettingsUpdate {
        let snap = s.snapshot();
        SettingsUpdate {
            name: snap.name,
            width: snap.width,
            height: snap.height,
            frame_rate_num: snap.frame_rate_num,
            frame_rate_den: snap.frame_rate_den,
            sample_rate: snap.sample_rate,
        }
    }

    fn prefs(interval: u32) -> AutosavePrefs {
        AutosavePrefs {
            enabled: true,
            interval_minutes: interval,
            keep_copies: 5,
        }
    }

    #[test]
    fn create_writes_project_folder_and_file_then_reopens_identically() {
        let t = TempDir::new("s-create");
        let s = Session::create(&params(t.path(), "Mon film")).unwrap();
        assert_eq!(s.path(), t.path().join("Mon film").join("Mon film.kito"));
        assert!(!s.is_dirty());
        assert!(s.snapshot().last_saved_at.is_some());
        let reopened = Session::open(s.path()).unwrap();
        assert_eq!(reopened.doc, s.doc);
        assert_eq!(reopened.snapshot().width, 1920);
    }

    #[test]
    fn create_refuses_existing_folder_without_touching_it() {
        let t = TempDir::new("s-exists");
        let existing = t.path().join("Film");
        fs::create_dir(&existing).unwrap();
        fs::write(existing.join("precieux.txt"), b"x").unwrap();
        let err = Session::create(&params(t.path(), "Film")).unwrap_err();
        assert_eq!(err.code(), "ALREADY_EXISTS");
        assert_eq!(fs::read(existing.join("precieux.txt")).unwrap(), b"x");
        assert!(!existing.join("Film.kito").exists());
    }

    #[test]
    fn create_validates_every_input() {
        let t = TempDir::new("s-invalid");
        let code = |p: NewProjectParams| Session::create(&p).unwrap_err().code();
        assert_eq!(code(params(t.path(), "a/b")), "INVALID_INPUT");
        assert_eq!(code(params(&t.path().join("absent"), "Film")), "NOT_FOUND");
        assert_eq!(code(params(Path::new("relatif"), "Film")), "INVALID_INPUT");
        let mut p = params(t.path(), "Film");
        p.width = 4;
        assert_eq!(code(p), "INVALID_FORMAT");
        let mut p = params(t.path(), "Film");
        p.sample_rate = 5;
        assert_eq!(code(p), "INVALID_FORMAT");
        let file = t.path().join("fichier.txt");
        fs::write(&file, b"x").unwrap();
        assert_eq!(code(params(&file, "Film")), "INVALID_INPUT");
        assert_eq!(
            fs::read_dir(t.path()).unwrap().count(),
            1,
            "aucun dossier créé par une entrée invalide"
        );
    }

    #[test]
    fn editing_settings_marks_dirty_and_save_persists_and_keeps_a_backup() {
        let t = TempDir::new("s-save");
        let mut s = Session::create(&params(t.path(), "Film")).unwrap();
        let mut u = update_from(&s);
        u.width = 3840;
        u.height = 2160;
        u.frame_rate_num = 60;
        u.name = "Film final".into();
        s.update_settings(&u).unwrap();
        assert!(s.is_dirty() && s.snapshot().dirty);

        s.save().unwrap();
        assert!(!s.is_dirty());
        let on_disk = io::read_document(s.path()).unwrap().doc;
        assert_eq!(
            (
                on_disk.settings.video.width,
                on_disk.settings.video.frame_rate.num
            ),
            (3840, 60)
        );
        assert_eq!(on_disk.project.name, "Film final");
        let previous = io::read_document(&s.backup_file()).unwrap().doc;
        assert_eq!(
            previous.settings.video.width, 1920,
            "le .bak contient la version précédente"
        );
    }

    #[test]
    fn identical_settings_do_not_dirty_the_project() {
        let t = TempDir::new("s-noop");
        let mut s = Session::create(&params(t.path(), "Film")).unwrap();
        s.update_settings(&update_from(&s)).unwrap();
        assert!(!s.is_dirty());
    }

    #[test]
    fn invalid_settings_are_rejected_and_leave_the_session_unchanged() {
        let t = TempDir::new("s-badsettings");
        let mut s = Session::create(&params(t.path(), "Film")).unwrap();
        let mut u = update_from(&s);
        u.width = 1;
        assert_eq!(s.update_settings(&u).unwrap_err().code(), "INVALID_FORMAT");
        let mut u = update_from(&s);
        u.name = "con".into();
        assert_eq!(s.update_settings(&u).unwrap_err().code(), "INVALID_INPUT");
        assert!(!s.is_dirty());
        assert_eq!(s.snapshot().width, 1920);
    }

    #[test]
    fn a_failed_save_keeps_the_session_dirty_and_the_file_intact() {
        let t = TempDir::new("s-failsave");
        let mut s = Session::create(&params(t.path(), "Film")).unwrap();
        let before = fs::read(s.path()).unwrap();
        let mut u = update_from(&s);
        u.width = 1280;
        s.update_settings(&u).unwrap();
        // Le dossier du projet disparaît : l'écriture doit échouer proprement.
        fs::remove_dir_all(s.dir()).unwrap();
        let err = s.save().unwrap_err();
        assert_eq!(err.code(), "NOT_FOUND");
        assert!(
            s.is_dirty(),
            "les modifications ne sont pas considérées comme enregistrées"
        );
        // Une fois le dossier recréé, l'utilisateur peut réessayer.
        fs::create_dir_all(s.dir()).unwrap();
        s.save().unwrap();
        assert!(!s.is_dirty());
        assert_ne!(fs::read(s.path()).unwrap(), before);
    }

    #[test]
    fn unknown_fields_survive_open_edit_save() {
        let t = TempDir::new("s-unknown");
        let file = t.path().join("Externe.kito");
        let original = document_with_unknown_fields();
        fs::write(&file, serde_json::to_vec_pretty(&original).unwrap()).unwrap();

        let mut s = Session::open(&file).unwrap();
        let mut u = update_from(&s);
        u.frame_rate_num = 25;
        s.update_settings(&u).unwrap();
        s.save().unwrap();

        let saved: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        for pointer in [
            "/topLevelFuture",
            "/savedBy/build",
            "/project/futureProjectField",
            "/settings/proxy",
            "/settings/video/colorSpace",
            "/settings/video/frameRate/dropFrame",
            "/settings/audio/channels",
            "/media/0/proxyPath",
            "/timeline/markers",
            "/timeline/sequences/0/color",
            "/timeline/sequences/0/tracks/0/locked",
            "/timeline/sequences/0/tracks/0/clips/0/label",
            "/timeline/sequences/0/tracks/0/clips/0/effects",
            "/timeline/sequences/0/tracks/0/clips/0/transitions",
            "/timeline/sequences/0/tracks/0/clips/0/keyframes",
        ] {
            assert_eq!(
                saved.pointer(pointer),
                original.pointer(pointer),
                "{pointer}"
            );
        }
        assert_eq!(
            saved["settings"]["video"]["frameRate"]["num"], 25,
            "la modification est bien appliquée"
        );
    }

    #[test]
    fn save_as_creates_a_new_file_keeps_the_old_one_and_rebases_media_paths() {
        let t = TempDir::new("s-saveas");
        let mut s = Session::create(&params(t.path(), "Film")).unwrap();
        let clip = t.path().join("Rushes").join("a.mp4");
        fs::create_dir_all(clip.parent().unwrap()).unwrap();
        fs::write(&clip, b"abc").unwrap();
        s.doc
            .media
            .push(media::media_ref_from_file(&s.dir(), "m1".into(), "video", &clip).unwrap());
        s.refresh_resolutions();
        s.save().unwrap();
        let old_path = s.path().to_path_buf();
        let old_bytes = fs::read(&old_path).unwrap();

        let dest_dir = t.path().join("Archive").join("2026");
        fs::create_dir_all(&dest_dir).unwrap();
        s.save_as(&dest_dir.join("Copie.kito")).unwrap();

        assert_eq!(s.path(), dest_dir.join("Copie.kito"));
        assert_eq!(
            fs::read(&old_path).unwrap(),
            old_bytes,
            "l'ancien fichier est inchangé"
        );
        let saved = io::read_document(s.path()).unwrap().doc;
        assert_eq!(
            saved.media[0].relative_path.as_deref(),
            Some("../../Rushes/a.mp4")
        );
        assert_eq!(s.snapshot().media[0].status, MediaStatus::Ok);
    }

    #[test]
    fn save_as_validates_the_destination() {
        let t = TempDir::new("s-saveas-bad");
        let mut s = Session::create(&params(t.path(), "Film")).unwrap();
        assert_eq!(
            s.save_as(&t.path().join("x.txt")).unwrap_err().code(),
            "INVALID_INPUT"
        );
        assert_eq!(
            s.save_as(&t.path().join("absent").join("x.kito"))
                .unwrap_err()
                .code(),
            "NOT_FOUND"
        );
        assert!(
            s.path().ends_with("Film.kito"),
            "la session n'a pas bougé après un échec"
        );
    }

    #[test]
    fn missing_media_are_detected_and_can_be_relinked_by_hand_or_by_folder() {
        let t = TempDir::new("s-relink");
        let mut s = Session::create(&params(t.path(), "Film")).unwrap();
        let clip = t.path().join("Rushes").join("plan.mp4");
        fs::create_dir_all(clip.parent().unwrap()).unwrap();
        fs::write(&clip, b"abcd").unwrap();
        s.doc
            .media
            .push(media::media_ref_from_file(&s.dir(), "m1".into(), "video", &clip).unwrap());
        s.doc.media.push({
            let c2 = t.path().join("Rushes").join("deux.mp4");
            fs::write(&c2, b"xy").unwrap();
            media::media_ref_from_file(&s.dir(), "m2".into(), "video", &c2).unwrap()
        });
        s.save().unwrap();

        // Les médias sont déplacés : à la réouverture, ils manquent.
        let moved = t.path().join("Deplace");
        fs::rename(t.path().join("Rushes"), &moved).unwrap();
        let mut reopened = Session::open(s.path()).unwrap();
        let statuses: Vec<_> = reopened.snapshot().media.iter().map(|m| m.status).collect();
        assert_eq!(statuses, [MediaStatus::Missing, MediaStatus::Missing]);

        reopened.relink("m1", &moved.join("plan.mp4")).unwrap();
        assert_eq!(reopened.snapshot().media[0].status, MediaStatus::Ok);
        assert!(reopened.is_dirty(), "relier un média modifie le projet");
        assert_eq!(
            reopened
                .relink("nope", &moved.join("plan.mp4"))
                .unwrap_err()
                .code(),
            "NOT_FOUND"
        );

        assert_eq!(reopened.relink_in_folder(&moved).unwrap(), 1);
        assert!(reopened
            .snapshot()
            .media
            .iter()
            .all(|m| m.status == MediaStatus::Ok));

        reopened.save().unwrap();
        let again = Session::open(s.path()).unwrap();
        assert!(again
            .snapshot()
            .media
            .iter()
            .all(|m| m.status == MediaStatus::Ok));
    }

    #[test]
    fn autosave_flow_never_touches_the_manual_file_and_supports_recovery() {
        let t = TempDir::new("s-autosave");
        let mut s = Session::create(&params(t.path(), "Film")).unwrap();
        let manual_bytes = fs::read(s.path()).unwrap();

        // Rien à sauvegarder tant que le projet est propre.
        let later = Instant::now() + Duration::from_secs(60 * 10);
        assert!(s.prepare_autosave(&prefs(5), later).unwrap().is_none());

        let mut u = update_from(&s);
        u.width = 1280;
        u.height = 720;
        s.update_settings(&u).unwrap();
        assert!(
            s.prepare_autosave(&prefs(5), Instant::now())
                .unwrap()
                .is_none(),
            "trop tôt"
        );
        assert!(s
            .prepare_autosave(
                &AutosavePrefs {
                    enabled: false,
                    ..prefs(5)
                },
                later
            )
            .unwrap()
            .is_none());

        let job = s
            .prepare_autosave(&prefs(5), later)
            .unwrap()
            .expect("copie due");
        let copy = autosave::write_autosave(&job, 5).unwrap();
        s.mark_autosaved(&job.project_id, job.revision);
        assert!(copy.starts_with(t.path().join("Film").join("Autosave")));
        assert_eq!(
            fs::read(s.path()).unwrap(),
            manual_bytes,
            "le fichier manuel est intact"
        );
        assert!(
            s.prepare_autosave(&prefs(5), later + Duration::from_secs(3600))
                .unwrap()
                .is_none(),
            "même révision : pas de doublon"
        );

        // Crash : on abandonne la session sans enregistrer. À la réouverture, la copie est proposée.
        let mtime_after = std::time::SystemTime::now() + Duration::from_secs(5);
        fs::File::options()
            .write(true)
            .open(&copy)
            .unwrap()
            .set_modified(mtime_after)
            .unwrap();
        drop(s);
        let reopened = Session::open(&t.path().join("Film").join("Film.kito")).unwrap();
        assert_eq!(
            reopened.snapshot().width,
            1920,
            "ouverture normale = dernière sauvegarde manuelle"
        );
        let offer = reopened.recovery_offer().expect("récupération proposée");
        assert_eq!(PathBuf::from(&offer.autosave_path), copy);

        let mut restored =
            Session::restore_from_autosave(reopened.path(), Path::new(&offer.autosave_path))
                .unwrap();
        assert_eq!(restored.snapshot().width, 1280, "contenu de la copie");
        assert!(restored.is_dirty(), "à enregistrer explicitement");
        assert_eq!(
            fs::read(restored.path()).unwrap(),
            manual_bytes,
            "toujours rien d'écrasé"
        );

        restored.save().unwrap();
        assert_eq!(
            io::read_document(restored.path())
                .unwrap()
                .doc
                .settings
                .video
                .width,
            1280
        );
        assert_eq!(
            io::read_document(&restored.backup_file())
                .unwrap()
                .doc
                .settings
                .video
                .width,
            1920
        );
    }

    #[test]
    fn restore_refuses_files_outside_the_autosave_folder() {
        let t = TempDir::new("s-restore-guard");
        let s = Session::create(&params(t.path(), "Film")).unwrap();
        let other = t.path().join("Film").join("Film.kito");
        assert_eq!(
            Session::restore_from_autosave(s.path(), &other)
                .unwrap_err()
                .code(),
            "INVALID_INPUT"
        );
        let traversal = t
            .path()
            .join("Film")
            .join("Autosave")
            .join("..")
            .join("Film.kito");
        assert_eq!(
            Session::restore_from_autosave(s.path(), &traversal)
                .unwrap_err()
                .code(),
            "INVALID_INPUT"
        );
    }

    #[test]
    fn discarding_removes_unsaved_autosaves_so_they_are_not_offered_again() {
        let t = TempDir::new("s-discard");
        let mut s = Session::create(&params(t.path(), "Film")).unwrap();
        let mut u = update_from(&s);
        u.width = 1280;
        s.update_settings(&u).unwrap();
        let later = Instant::now() + Duration::from_secs(3600);
        let job = s.prepare_autosave(&prefs(1), later).unwrap().unwrap();
        let copy = autosave::write_autosave(&job, 5).unwrap();
        fs::File::options()
            .write(true)
            .open(&copy)
            .unwrap()
            .set_modified(std::time::SystemTime::now() + Duration::from_secs(5))
            .unwrap();
        assert!(s.recovery_offer().is_some());
        assert_eq!(s.discard_unsaved_autosaves().unwrap(), 1);
        assert!(s.recovery_offer().is_none());
    }

    #[test]
    fn a_migrated_project_keeps_a_one_time_backup_of_the_old_version() {
        let t = TempDir::new("s-premig");
        let mut s = Session::create(&params(t.path(), "Film")).unwrap();
        // Simule un projet ouvert après migration depuis la version 0 (étape fictive).
        s.migrated_from = Some(0);
        s.premigration_backup_pending = true;
        let original = fs::read(s.path()).unwrap();
        s.save().unwrap();
        let legacy = s.path().with_file_name("Film.kito.v0.bak");
        assert_eq!(fs::read(&legacy).unwrap(), original);
        s.touch();
        s.save().unwrap();
        assert_eq!(
            fs::read(&legacy).unwrap(),
            original,
            "la copie d'origine n'est jamais réécrite"
        );
    }

    #[test]
    fn newer_compatible_projects_open_and_keep_their_format_version() {
        let t = TempDir::new("s-newer");
        let file = t.path().join("Futur.kito");
        let mut v = document_with_unknown_fields();
        v["formatVersion"] = 2.into();
        v["minReaderVersion"] = 1.into();
        fs::write(&file, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
        let mut s = Session::open(&file).unwrap();
        assert!(s.snapshot().newer_format);
        s.save().unwrap();
        let saved: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        assert_eq!(saved["formatVersion"], 2);
    }
}
