use crate::error::AppResult;
use crate::preferences::{Preferences, PreferencesStore};
use crate::state::{lock, AppStorage, PreferencesState};
use tauri::State;

#[tauri::command(async)]
pub fn get_preferences(prefs: State<'_, PreferencesState>) -> AppResult<Preferences> {
    Ok(lock(&prefs.0).clone())
}

#[tauri::command(async)]
pub fn set_preferences(
    preferences: Preferences,
    prefs: State<'_, PreferencesState>,
    storage: State<'_, AppStorage>,
) -> AppResult<Preferences> {
    let validated = preferences.validated()?;
    PreferencesStore::new(&storage.dir).save(&validated)?;
    *lock(&prefs.0) = validated.clone();
    Ok(validated)
}
