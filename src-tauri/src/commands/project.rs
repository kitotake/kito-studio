//! Commandes de gestion de projet : validation d'entrée, délégation à `project::*`.
//!
//! `#[tauri::command(async)]` exécute ces fonctions sur un thread du pool : les accès
//! disque ne bloquent jamais le thread de l'interface.

use crate::error::{AppError, AppResult};
use crate::project::autosave::{self, AutosaveInfo};
use crate::project::recent::RecentStore;
use crate::project::session::{NewProjectParams, Session, SettingsUpdate};
use crate::project::snapshot::{OpenOutcome, ProjectSnapshot, RelinkReport};
use crate::state::{lock, AppStorage, ProjectState};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProjectInput {
    pub name: String,
    pub parent_dir: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate_num: u32,
    pub frame_rate_den: u32,
    pub sample_rate: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate_num: u32,
    pub frame_rate_den: u32,
    pub sample_rate: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened_at: String,
    pub exists: bool,
}

fn no_project() -> AppError {
    AppError::NotFound("Aucun projet n'est ouvert.".into())
}

fn with_session<R>(
    state: &ProjectState,
    f: impl FnOnce(&mut Session) -> AppResult<R>,
) -> AppResult<R> {
    let mut guard = lock(&state.session);
    f(guard.as_mut().ok_or_else(no_project)?)
}

/// Un échec d'écriture de la liste des récents ne doit jamais faire échouer l'action principale.
fn remember(storage: &AppStorage, session: &Session) {
    let _ = RecentStore::new(&storage.dir).record(session.path(), session.name());
}

#[tauri::command(async)]
pub fn create_project(
    input: NewProjectInput,
    state: State<'_, ProjectState>,
    storage: State<'_, AppStorage>,
) -> AppResult<ProjectSnapshot> {
    let session = Session::create(&NewProjectParams {
        name: input.name,
        parent_dir: PathBuf::from(input.parent_dir),
        width: input.width,
        height: input.height,
        frame_rate_num: input.frame_rate_num,
        frame_rate_den: input.frame_rate_den,
        sample_rate: input.sample_rate,
    })?;
    remember(&storage, &session);
    let snapshot = session.snapshot();
    *lock(&state.session) = Some(session);
    Ok(snapshot)
}

#[tauri::command(async)]
pub fn open_project(
    path: String,
    state: State<'_, ProjectState>,
    storage: State<'_, AppStorage>,
) -> AppResult<OpenOutcome> {
    let session = Session::open(Path::new(&path))?;
    remember(&storage, &session);
    let outcome = OpenOutcome {
        snapshot: session.snapshot(),
        recovery: session.recovery_offer(),
    };
    *lock(&state.session) = Some(session);
    Ok(outcome)
}

#[tauri::command(async)]
pub fn list_autosaves(project_path: String) -> AppResult<Vec<AutosaveInfo>> {
    Ok(autosave::list_autosaves(Path::new(&project_path)))
}

#[tauri::command(async)]
pub fn restore_autosave(
    project_path: String,
    autosave_path: String,
    state: State<'_, ProjectState>,
    storage: State<'_, AppStorage>,
) -> AppResult<ProjectSnapshot> {
    let session =
        Session::restore_from_autosave(Path::new(&project_path), Path::new(&autosave_path))?;
    remember(&storage, &session);
    let snapshot = session.snapshot();
    *lock(&state.session) = Some(session);
    Ok(snapshot)
}

#[tauri::command(async)]
pub fn get_current_project(state: State<'_, ProjectState>) -> AppResult<Option<ProjectSnapshot>> {
    Ok(lock(&state.session).as_ref().map(Session::snapshot))
}

#[tauri::command(async)]
pub fn save_project(state: State<'_, ProjectState>) -> AppResult<ProjectSnapshot> {
    with_session(&state, |s| {
        s.save()?;
        Ok(s.snapshot())
    })
}

#[tauri::command(async)]
pub fn save_project_as(
    path: String,
    state: State<'_, ProjectState>,
    storage: State<'_, AppStorage>,
) -> AppResult<ProjectSnapshot> {
    with_session(&state, |s| {
        s.save_as(Path::new(&path))?;
        remember(&storage, s);
        Ok(s.snapshot())
    })
}

#[tauri::command(async)]
pub fn update_project_settings(
    input: SettingsInput,
    state: State<'_, ProjectState>,
) -> AppResult<ProjectSnapshot> {
    with_session(&state, |s| {
        s.update_settings(&SettingsUpdate {
            name: input.name,
            width: input.width,
            height: input.height,
            frame_rate_num: input.frame_rate_num,
            frame_rate_den: input.frame_rate_den,
            sample_rate: input.sample_rate,
        })?;
        Ok(s.snapshot())
    })
}

/// Ferme le projet. Avec `discard_changes`, les copies automatiques plus récentes que
/// le fichier sont supprimées : l'utilisateur a choisi d'abandonner ces modifications.
#[tauri::command(async)]
pub fn close_project(discard_changes: bool, state: State<'_, ProjectState>) -> AppResult<()> {
    let session = lock(&state.session).take();
    if let (Some(session), true) = (session, discard_changes) {
        session.discard_unsaved_autosaves()?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn relink_media(
    media_id: String,
    file_path: String,
    state: State<'_, ProjectState>,
) -> AppResult<ProjectSnapshot> {
    with_session(&state, |s| {
        s.relink(&media_id, Path::new(&file_path))?;
        Ok(s.snapshot())
    })
}

#[tauri::command(async)]
pub fn relink_media_in_folder(
    folder_path: String,
    state: State<'_, ProjectState>,
) -> AppResult<RelinkReport> {
    with_session(&state, |s| {
        let relinked = s.relink_in_folder(Path::new(&folder_path))?;
        Ok(RelinkReport {
            relinked,
            snapshot: s.snapshot(),
        })
    })
}

#[tauri::command(async)]
pub fn get_recent_projects(storage: State<'_, AppStorage>) -> AppResult<Vec<RecentProject>> {
    Ok(RecentStore::new(&storage.dir)
        .list()
        .into_iter()
        .map(|e| RecentProject {
            exists: Path::new(&e.path).is_file(),
            path: e.path,
            name: e.name,
            last_opened_at: e.last_opened_at,
        })
        .collect())
}

#[tauri::command(async)]
pub fn remove_recent_project(path: String, storage: State<'_, AppStorage>) -> AppResult<()> {
    RecentStore::new(&storage.dir).remove(&path)
}

#[tauri::command(async)]
pub fn clear_recent_projects(storage: State<'_, AppStorage>) -> AppResult<()> {
    RecentStore::new(&storage.dir).clear()
}
