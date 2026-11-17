//! Vues sérialisables envoyées à l'interface (jamais le document complet).

use super::media::MediaStatus;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaView {
    pub id: String,
    pub kind: String,
    pub file_name: String,
    pub path: String,
    pub status: MediaStatus,
    pub resolved_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    pub id: String,
    pub name: String,
    pub path: String,
    pub directory: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate_num: u32,
    pub frame_rate_den: u32,
    pub sample_rate: u32,
    pub created_at: String,
    pub modified_at: String,
    pub dirty: bool,
    pub last_saved_at: Option<String>,
    pub format_version: u32,
    /// Version d'origine si le fichier a été migré à l'ouverture.
    pub migrated_from: Option<u32>,
    /// Fichier créé par une version plus récente mais compatible.
    pub newer_format: bool,
    pub media: Vec<MediaView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryOffer {
    pub autosave_path: String,
    pub autosaved_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOutcome {
    pub snapshot: ProjectSnapshot,
    pub recovery: Option<RecoveryOffer>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelinkReport {
    pub relinked: usize,
    pub snapshot: ProjectSnapshot,
}
