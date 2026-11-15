use crate::error::AppResult;
use crate::media::tools::{self, MediaToolsStatus};
use crate::system::{self, RuntimeInfo};

#[tauri::command]
pub fn get_runtime_info() -> AppResult<RuntimeInfo> {
    Ok(system::runtime_info())
}

#[tauri::command]
pub fn check_media_tools() -> AppResult<MediaToolsStatus> {
    Ok(tools::detect())
}
