//! Tests d'intégration du cycle de vie d'un projet, via l'API publique, sur de vrais fichiers.

mod common;

use common::*;
use kito_studio_lib::preferences::{AutosavePrefs, Preferences, PreferencesStore};
use kito_studio_lib::project::autosave::{self, AUTOSAVE_DIR};
use kito_studio_lib::project::io;
use kito_studio_lib::project::media::{self, MediaStatus};
use kito_studio_lib::project::recent::RecentStore;
use kito_studio_lib::project::session::{NewProjectParams, Session, SettingsUpdate};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

fn params(parent: &Path, name: &str) -> NewProjectParams {
    NewProjectParams {
        name: name.into(),
        parent_dir: parent.to_path_buf(),
        width: 1920,
        height: 1080,
        frame_rate_num: 24,
        frame_rate_den: 1,
        sample_rate: 48_000,
    }
}

fn settings(s: &Session, f: impl FnOnce(&mut SettingsUpdate)) -> SettingsUpdate {
    let snap = s.snapshot();
    let mut u = SettingsUpdate {
        name: snap.name,
        width: snap.width,
        height: snap.height,
        frame_rate_num: snap.frame_rate_num,
        frame_rate_den: snap.frame_rate_den,
        sample_rate: snap.sample_rate,
    };
    f(&mut u);
    u
}

fn touch_mtime(path: &Path, secs_ahead: u64) {
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(SystemTime::now() + Duration::from_secs(secs_ahead))
        .unwrap();
}

#[test]
fn full_lifecycle_create_edit_save_close_reopen() {
    let t = TempDir::new("lifecycle");
    let appdata = t.path().join("appdata");
    let recents = RecentStore::new(&appdata);

    let mut s = Session::create(&params(t.path(), "Court métrage")).unwrap();
    recents.record(s.path(), s.name()).unwrap();
    assert_eq!(
        list_names(&t.path().join("Court métrage")),
        ["Court métrage.kito"]
    );

    let u = settings(&s, |u| {
        u.name = "Court métrage v2".into();
        u.width = 2560;
        u.height = 1440;
        u.frame_rate_num = 50;
        u.sample_rate = 96_000;
    });
    s.update_settings(&u).unwrap();
    assert!(s.snapshot().dirty);
    s.save().unwrap();
    assert!(!s.snapshot().dirty);
    let path = s.path().to_path_buf();
    drop(s);

    let reopened = Session::open(&path).unwrap();
    let snap = reopened.snapshot();
    assert_eq!(
        (
            snap.name.as_str(),
            snap.width,
            snap.height,
            snap.frame_rate_num,
            snap.sample_rate
        ),
        ("Court métrage v2", 2560, 1440, 50, 96_000)
    );
    assert!(!snap.dirty && snap.media.is_empty() && snap.migrated_from.is_none());
    assert_eq!(recents.list().len(), 1);
}

#[test]
fn repeated_saves_never_leave_temporary_files_behind() {
    let t = TempDir::new("no-tmp");
    let mut s = Session::create(&params(t.path(), "Film")).unwrap();
    for w in [1280, 1920, 3840, 1280] {
        let u = settings(&s, |u| u.width = w);
        s.update_settings(&u).unwrap();
        s.save().unwrap();
    }
    assert_eq!(
        list_names(&t.path().join("Film")),
        ["Film.kito", "Film.kito.bak"]
    );
}

#[test]
fn project_moved_to_another_computer_finds_its_media_through_relative_paths() {
    let t = TempDir::new("portable");
    let pc1 = t.path().join("pc1");
    fs::create_dir_all(&pc1).unwrap();
    write_file(&pc1.join("Rushes").join("interview.mp4"), b"0123456789");
    write_file(&pc1.join("Rushes").join("sous").join("musique.wav"), b"abc");

    let mut s = Session::create(&params(&pc1, "Documentaire")).unwrap();
    let dir = s.dir();
    let m1 = media::media_ref_from_file(
        &dir,
        "m1".into(),
        "video",
        &pc1.join("Rushes").join("interview.mp4"),
    )
    .unwrap();
    let m2 = media::media_ref_from_file(
        &dir,
        "m2".into(),
        "audio",
        &pc1.join("Rushes").join("sous").join("musique.wav"),
    )
    .unwrap();
    let mut doc = io::read_document(s.path()).unwrap().doc;
    doc.media = vec![m1, m2];
    fs::write(s.path(), io::serialize_document(&doc).unwrap()).unwrap();
    s = Session::open(s.path()).unwrap();
    assert!(s
        .snapshot()
        .media
        .iter()
        .all(|m| m.status == MediaStatus::Ok));

    // « Autre ordinateur » : tout l'arbre est copié ailleurs, l'ancien est supprimé.
    let pc2 = t.path().join("pc2").join("D").join("Mes projets");
    copy_tree(&pc1, &pc2);
    fs::remove_dir_all(&pc1).unwrap();

    let moved = Session::open(&pc2.join("Documentaire").join("Documentaire.kito")).unwrap();
    let snap = moved.snapshot();
    assert!(
        snap.media.iter().all(|m| m.status == MediaStatus::Ok),
        "{:?}",
        snap.media
    );
    assert_eq!(
        snap.media[0].resolved_path.as_deref().map(Path::new),
        Some(pc2.join("Rushes").join("interview.mp4").as_path())
    );
    assert!(
        snap.media[0].path.contains("pc1"),
        "le chemin absolu d'origine reste visible tant que le projet n'est pas réenregistré"
    );

    // Réenregistré sur le nouvel ordinateur, le projet se « rebase » sur sa nouvelle position.
    let mut moved = moved;
    moved.save().unwrap();
    let saved = io::read_document(moved.path()).unwrap().doc;
    assert!(saved.media[0].path.starts_with(pc2.to_str().unwrap()));
    assert_eq!(
        saved.media[0].relative_path.as_deref(),
        Some("../Rushes/interview.mp4")
    );
}

#[test]
fn corrupted_project_is_refused_without_touching_it_and_the_backup_stays_usable() {
    let t = TempDir::new("corrupt");
    let mut s = Session::create(&params(t.path(), "Film")).unwrap();
    let u = settings(&s, |u| u.width = 1280);
    s.update_settings(&u).unwrap();
    s.save().unwrap(); // crée Film.kito.bak (version à 1920)
    let path = s.path().to_path_buf();
    let good = fs::read(&path).unwrap();

    let truncated = &good[..good.len() / 3];
    fs::write(&path, truncated).unwrap();
    let err = Session::open(&path).unwrap_err();
    assert_eq!(err.code(), "INVALID_FORMAT");
    assert_eq!(
        fs::read(&path).unwrap(),
        truncated,
        "le fichier corrompu n'est jamais modifié ni supprimé"
    );

    let from_backup = Session::open(&s.backup_file_for_test()).unwrap();
    assert_eq!(
        from_backup.snapshot().width,
        1920,
        "la copie de secours .bak est directement ouvrable"
    );
}

trait BackupPath {
    fn backup_file_for_test(&self) -> std::path::PathBuf;
}
impl BackupPath for Session {
    fn backup_file_for_test(&self) -> std::path::PathBuf {
        io::backup_path(self.path())
    }
}

#[test]
fn version_checks_on_real_files() {
    let t = TempDir::new("versions");
    let write_version = |name: &str, version: u64, min: u64| {
        let mut v = handwritten_project(json!([]));
        v["formatVersion"] = version.into();
        v["minReaderVersion"] = min.into();
        v["futureStuff"] = json!({ "kept": true });
        let p = t.path().join(name);
        fs::write(&p, serde_json::to_vec_pretty(&v).unwrap()).unwrap();
        p
    };

    let current = write_version("courant.kito", 1, 1);
    Session::open(&current).unwrap();

    let newer_ok = write_version("futur-compatible.kito", 3, 1);
    let mut s = Session::open(&newer_ok).unwrap();
    assert!(s.snapshot().newer_format);
    s.save().unwrap();
    let saved: Value = serde_json::from_slice(&fs::read(&newer_ok).unwrap()).unwrap();
    assert_eq!(saved["formatVersion"], 3);
    assert_eq!(
        saved["futureStuff"],
        json!({ "kept": true }),
        "données du futur conservées"
    );

    let newer_bad = write_version("futur-incompatible.kito", 4, 3);
    let before = fs::read(&newer_bad).unwrap();
    let err = Session::open(&newer_bad).unwrap_err();
    assert_eq!(err.code(), "INCOMPATIBLE_VERSION");
    assert!(err.to_string().contains("version plus récente"));
    assert_eq!(fs::read(&newer_bad).unwrap(), before);
}

#[test]
fn crash_recovery_uses_the_autosave_without_overwriting_the_manual_save() {
    let t = TempDir::new("crash");
    let mut s = Session::create(&params(t.path(), "Film")).unwrap();
    let manual_before = fs::read(s.path()).unwrap();

    let u = settings(&s, |u| {
        u.name = "Film modifié avant le crash".into();
        u.width = 3840;
        u.height = 2160;
    });
    s.update_settings(&u).unwrap();

    let prefs = AutosavePrefs {
        enabled: true,
        interval_minutes: 1,
        keep_copies: 3,
    };
    let job = s
        .prepare_autosave(&prefs, Instant::now() + Duration::from_secs(61))
        .unwrap()
        .expect("copie due");
    let copy = autosave::write_autosave(&job, 3).unwrap();
    touch_mtime(&copy, 5);
    let project_path = s.path().to_path_buf();
    drop(s); // « crash » : aucun enregistrement manuel

    assert_eq!(fs::read(&project_path).unwrap(), manual_before);
    let reopened = Session::open(&project_path).unwrap();
    assert_eq!(reopened.snapshot().width, 1920);
    let offer = reopened
        .recovery_offer()
        .expect("la récupération est proposée");

    let mut restored =
        Session::restore_from_autosave(&project_path, Path::new(&offer.autosave_path)).unwrap();
    assert_eq!(restored.snapshot().name, "Film modifié avant le crash");
    assert!(restored.snapshot().dirty);
    assert_eq!(
        fs::read(&project_path).unwrap(),
        manual_before,
        "toujours intact avant l'enregistrement"
    );

    restored.save().unwrap();
    assert_eq!(Session::open(&project_path).unwrap().snapshot().width, 3840);
    assert_eq!(
        io::read_document(&io::backup_path(&project_path))
            .unwrap()
            .doc
            .settings
            .video
            .width,
        1920
    );
}

#[test]
fn autosave_can_rescue_a_project_whose_main_file_is_corrupted_or_deleted() {
    let t = TempDir::new("rescue");
    let mut s = Session::create(&params(t.path(), "Film")).unwrap();
    let u = settings(&s, |u| u.width = 1280);
    s.update_settings(&u).unwrap();
    let prefs = AutosavePrefs {
        enabled: true,
        interval_minutes: 1,
        keep_copies: 3,
    };
    let job = s
        .prepare_autosave(&prefs, Instant::now() + Duration::from_secs(120))
        .unwrap()
        .unwrap();
    let copy = autosave::write_autosave(&job, 3).unwrap();
    let path = s.path().to_path_buf();
    drop(s);

    for damage in ["corrompu", "supprimé"] {
        if damage == "corrompu" {
            fs::write(&path, b"{ corrompu").unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
        assert!(Session::open(&path).is_err());
        let copies = autosave::list_autosaves(&path);
        assert_eq!(copies.len(), 1, "{damage}");
        let restored = Session::restore_from_autosave(&path, &copy).unwrap();
        assert_eq!(restored.snapshot().width, 1280, "{damage}");
    }
}

#[test]
fn abandoning_changes_deliberately_does_not_trigger_a_recovery_prompt_later() {
    let t = TempDir::new("abandon");
    let mut s = Session::create(&params(t.path(), "Film")).unwrap();
    let u = settings(&s, |u| u.width = 1280);
    s.update_settings(&u).unwrap();
    let prefs = AutosavePrefs {
        enabled: true,
        interval_minutes: 1,
        keep_copies: 3,
    };
    let job = s
        .prepare_autosave(&prefs, Instant::now() + Duration::from_secs(120))
        .unwrap()
        .unwrap();
    touch_mtime(&autosave::write_autosave(&job, 3).unwrap(), 5);
    assert!(s.recovery_offer().is_some());
    s.discard_unsaved_autosaves().unwrap();
    assert!(Session::open(s.path()).unwrap().recovery_offer().is_none());
}

#[test]
fn unknown_data_survives_the_whole_chain_open_edit_autosave_restore_save() {
    let t = TempDir::new("chain");
    let file = t.path().join("Projet.kito");
    let mut original = handwritten_project(json!([]));
    original["topLevel"] = json!({ "deep": [1, { "x": null }] });
    original["timeline"]["sequences"] = json!([{
        "id": "s1", "name": "S", "futureSeqField": 7,
        "tracks": [{ "id": "t1", "kind": "subtitle", "name": "ST", "clips": [{
            "id": "c1", "start": 0, "duration": 10, "sourceIn": 0, "label": "x",
            "effects": [{ "type": "future-effect", "params": { "k": [1, 2, 3] } }],
            "transitions": [], "keyframes": []
        }]}]
    }]);
    fs::write(&file, serde_json::to_vec_pretty(&original).unwrap()).unwrap();

    let mut s = Session::open(&file).unwrap();
    let u = settings(&s, |u| u.width = 1280);
    s.update_settings(&u).unwrap();
    let prefs = AutosavePrefs {
        enabled: true,
        interval_minutes: 1,
        keep_copies: 3,
    };
    let job = s
        .prepare_autosave(&prefs, Instant::now() + Duration::from_secs(120))
        .unwrap()
        .unwrap();
    let copy = autosave::write_autosave(&job, 3).unwrap();
    let mut restored = Session::restore_from_autosave(&file, &copy).unwrap();
    restored.save().unwrap();

    let saved: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    for p in [
        "/topLevel",
        "/timeline/sequences/0/futureSeqField",
        "/timeline/sequences/0/tracks/0/kind",
        "/timeline/sequences/0/tracks/0/clips/0/label",
        "/timeline/sequences/0/tracks/0/clips/0/effects",
    ] {
        assert_eq!(saved.pointer(p), original.pointer(p), "{p}");
    }
    assert_eq!(saved["settings"]["video"]["width"], 1280);
}

#[test]
fn hostile_project_content_is_treated_as_inert_data() {
    let t = TempDir::new("security");
    let evil_exe = t.path().join("evil.exe");
    write_file(&evil_exe, b"MZ not really a program");
    let mut doc = handwritten_project(json!([{
        "id": "m1", "kind": "video", "path": evil_exe.to_str().unwrap(), "relativePath": "evil.exe", "size": 23,
        "onLoad": "rm -rf /", "command": "calc.exe"
    }]));
    doc["autorun"] =
        json!({ "exec": "powershell -enc AAAA", "url": "http://evil.example/payload" });
    doc["timeline"]["sequences"] = json!([{ "id": "s", "name": "S", "tracks": [{ "id": "t", "kind": "video", "name": "V1", "clips": [{
        "id": "c", "mediaId": "m1", "start": 0, "duration": 5,
        "effects": [{ "type": "custom-shader", "source": "while(true){}", "command": "cmd /c calc", "script": "alert(1)" }],
        "transitions": [{ "type": "wipe", "url": "file:///etc/passwd" }],
        "keyframes": []
    }]}]}]);
    let dir = t.path().join("Projet");
    let file = dir.join("Projet.kito");
    write_file(&file, &serde_json::to_vec_pretty(&doc).unwrap());
    let names_before = list_names(&dir);

    // Ouvrir, enregistrer, ouvrir de nouveau : aucune exécution, aucun effet de bord.
    let mut s = Session::open(&file).unwrap();
    assert_eq!(
        s.snapshot().media[0].status,
        MediaStatus::Ok,
        "le média est seulement statué"
    );
    s.save().unwrap();
    let again: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    assert_eq!(
        again["autorun"], doc["autorun"],
        "conservé tel quel, jamais interprété"
    );
    let clip = "/timeline/sequences/0/tracks/0/clips/0";
    assert_eq!(
        again.pointer(&format!("{clip}/effects")),
        doc.pointer(&format!("{clip}/effects"))
    );
    assert_eq!(
        list_names(&dir),
        [names_before.clone(), vec!["Projet.kito.bak".to_string()]].concat()
    );
    assert_eq!(fs::read(&evil_exe).unwrap(), b"MZ not really a program");
}

#[test]
fn non_declarative_effects_are_refused() {
    let t = TempDir::new("declarative");
    for (label, entry) in [
        ("texte exécutable", json!("powershell -c calc")),
        ("sans type", json!({ "command": "calc.exe" })),
        ("type chemin", json!({ "type": "../../evil.exe" })),
        ("type majuscules", json!({ "type": "Custom" })),
    ] {
        let mut doc = handwritten_project(json!([]));
        doc["timeline"]["sequences"] = json!([{ "id": "s", "name": "S", "tracks": [{ "id": "t", "kind": "video", "name": "V1", "clips": [{
            "id": "c", "start": 0, "duration": 1, "effects": [entry], "transitions": [], "keyframes": []
        }]}]}]);
        let file = t.path().join(format!("{label}.kito"));
        fs::write(&file, serde_json::to_vec(&doc).unwrap()).unwrap();
        let err = Session::open(&file).expect_err(label);
        assert_eq!(err.code(), "INVALID_FORMAT", "{label}");
        assert!(err.to_string().contains("effects[0]"), "{label} : {err}");
    }
}

#[test]
fn preferences_and_recents_persist_across_restarts() {
    let t = TempDir::new("persist");
    let appdata = t.path().join("data");
    let prefs = Preferences {
        version: 1,
        autosave: AutosavePrefs {
            enabled: true,
            interval_minutes: 2,
            keep_copies: 7,
        },
    };
    PreferencesStore::new(&appdata).save(&prefs).unwrap();
    let s = Session::create(&params(t.path(), "Film")).unwrap();
    RecentStore::new(&appdata)
        .record(s.path(), s.name())
        .unwrap();

    assert_eq!(PreferencesStore::new(&appdata).load(), prefs);
    let recents = RecentStore::new(&appdata).list();
    assert_eq!(recents.len(), 1);
    assert_eq!(recents[0].name, "Film");
    assert!(Path::new(&recents[0].path).is_file());
    assert_eq!(AUTOSAVE_DIR, "Autosave");
}
