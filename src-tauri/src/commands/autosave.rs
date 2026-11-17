//! Boucle de sauvegarde automatique (thread dédié, une vérification par seconde).
//!
//! La décision et la sérialisation se font sous le verrou (rapide) ; l'écriture disque
//! se fait hors verrou pour ne jamais bloquer l'interface.

use crate::project::autosave;
use crate::state::{lock, PreferencesState, ProjectState};
use serde::Serialize;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

pub const EVENT_AUTOSAVED: &str = "project:autosaved";
pub const EVENT_AUTOSAVE_FAILED: &str = "project:autosave-failed";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Autosaved {
    saved_at: String,
    path: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AutosaveFailed {
    message: String,
}

pub fn spawn(app: AppHandle) {
    let _ = thread::Builder::new()
        .name("kito-autosave".into())
        .spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
            tick(&app);
        });
}

fn fail(app: &AppHandle, message: String) {
    let _ = app.emit(EVENT_AUTOSAVE_FAILED, AutosaveFailed { message });
}

fn tick(app: &AppHandle) {
    let project = app.state::<ProjectState>();
    let prefs = lock(&app.state::<PreferencesState>().0).autosave.clone();

    let job = {
        let mut guard = lock(&project.session);
        let Some(session) = guard.as_mut() else {
            return;
        };
        match session.prepare_autosave(&prefs, Instant::now()) {
            Ok(job) => job,
            Err(e) => return fail(app, e.to_string()),
        }
    };
    let Some(job) = job else { return };

    match autosave::write_autosave(&job, prefs.keep_copies as usize) {
        Ok(path) => {
            if let Some(session) = lock(&project.session).as_mut() {
                session.mark_autosaved(&job.project_id, job.revision);
            }
            let _ = app.emit(
                EVENT_AUTOSAVED,
                Autosaved {
                    saved_at: crate::timeutil::now_iso(),
                    path: path.to_string_lossy().into_owned(),
                },
            );
        }
        Err(e) => fail(app, format!("Sauvegarde automatique impossible : {e}")),
    }
}
