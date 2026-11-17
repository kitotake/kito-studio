//! Modèle du document `.kito` (format v1).
//!
//! Règle d'or : chaque objet garde ses champs inconnus dans `extra` et les réécrit
//! tels quels, pour ne jamais perdre de données créées par une version plus récente.
//!
//! Sécurité : le document ne contient que des DONNÉES. Les effets, transitions et
//! keyframes sont des objets JSON déclaratifs (`{ "type": "...", ... }`) ; aucun champ
//! n'est jamais interprété comme du code, une commande ou une URL à exécuter.

use crate::timeutil;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const FORMAT_NAME: &str = "kito-project";
pub const CURRENT_FORMAT_VERSION: u32 = 1;
/// 705 600 000 : divisible exactement par 24/25/30/50/60 i/s (et 23,976/29,97/59,94)
/// ainsi que par 44,1/48/96 kHz. Garde les positions entières, donc sans dérive.
pub const TICKS_PER_SECOND: i64 = 705_600_000;
/// Plus grand entier exactement représentable côté JavaScript (2^53 - 1).
pub const MAX_SAFE_TICKS: i64 = 9_007_199_254_740_991;

pub type Extra = Map<String, Value>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KitoDocument {
    pub format: String,
    pub format_version: u32,
    pub min_reader_version: u32,
    pub saved_by: SavedBy,
    pub project: ProjectMeta,
    pub settings: Settings,
    #[serde(default)]
    pub media: Vec<MediaRef>,
    pub timeline: Timeline,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedBy {
    pub app: String,
    pub version: String,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMeta {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub modified_at: String,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub video: VideoSettings,
    pub audio: AudioSettings,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSettings {
    pub width: u32,
    pub height: u32,
    pub frame_rate: FrameRate,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameRate {
    pub num: u32,
    pub den: u32,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSettings {
    pub sample_rate: u32,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaRef {
    pub id: String,
    pub kind: String,
    /// Chemin absolu tel qu'il était sur l'ordinateur d'origine (indicatif).
    pub path: String,
    /// Chemin relatif au dossier du projet, séparateur `/` (portable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    pub ticks_per_second: i64,
    #[serde(default)]
    pub sequences: Vec<Sequence>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sequence {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tracks: Vec<Track>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub clips: Vec<Clip>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_id: Option<String>,
    pub start: i64,
    pub duration: i64,
    #[serde(default)]
    pub source_in: i64,
    /// Entrées déclaratives `{ "type": "...", ... }` — jamais exécutées.
    #[serde(default)]
    pub effects: Vec<Value>,
    #[serde(default)]
    pub transitions: Vec<Value>,
    #[serde(default)]
    pub keyframes: Vec<Value>,
    #[serde(flatten)]
    pub extra: Extra,
}

/// Paramètres de création d'un document vierge.
#[derive(Debug, Clone, PartialEq)]
pub struct NewDocument {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate_num: u32,
    pub frame_rate_den: u32,
    pub sample_rate: u32,
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

impl KitoDocument {
    pub fn new(input: &NewDocument) -> Self {
        let now = timeutil::now_iso();
        let track = |kind: &str, name: &str| Track {
            id: new_id(),
            kind: kind.to_string(),
            name: name.to_string(),
            clips: Vec::new(),
            extra: Extra::new(),
        };
        Self {
            format: FORMAT_NAME.to_string(),
            format_version: CURRENT_FORMAT_VERSION,
            min_reader_version: CURRENT_FORMAT_VERSION,
            saved_by: SavedBy::current(),
            project: ProjectMeta {
                id: new_id(),
                name: input.name.clone(),
                created_at: now.clone(),
                modified_at: now,
                extra: Extra::new(),
            },
            settings: Settings {
                video: VideoSettings {
                    width: input.width,
                    height: input.height,
                    frame_rate: FrameRate {
                        num: input.frame_rate_num,
                        den: input.frame_rate_den,
                        extra: Extra::new(),
                    },
                    extra: Extra::new(),
                },
                audio: AudioSettings {
                    sample_rate: input.sample_rate,
                    extra: Extra::new(),
                },
                extra: Extra::new(),
            },
            media: Vec::new(),
            timeline: Timeline {
                ticks_per_second: TICKS_PER_SECOND,
                sequences: vec![Sequence {
                    id: new_id(),
                    name: "Séquence 01".to_string(),
                    tracks: vec![
                        track("video", "V1"),
                        track("video", "V2"),
                        track("audio", "A1"),
                        track("audio", "A2"),
                    ],
                    extra: Extra::new(),
                }],
                extra: Extra::new(),
            },
            extra: Extra::new(),
        }
    }
}

impl SavedBy {
    pub fn current() -> Self {
        Self {
            app: "Kito Studio".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            extra: Extra::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> NewDocument {
        NewDocument {
            name: "Test".into(),
            width: 1920,
            height: 1080,
            frame_rate_num: 30,
            frame_rate_den: 1,
            sample_rate: 48_000,
        }
    }

    #[test]
    fn new_document_has_default_sequence_with_four_tracks() {
        let doc = KitoDocument::new(&sample_input());
        assert_eq!(doc.format, FORMAT_NAME);
        assert_eq!(doc.timeline.ticks_per_second, TICKS_PER_SECOND);
        let tracks: Vec<&str> = doc.timeline.sequences[0]
            .tracks
            .iter()
            .map(|t| t.name.as_str())
            .collect();
        assert_eq!(tracks, ["V1", "V2", "A1", "A2"]);
    }

    #[test]
    fn ticks_divide_evenly_by_common_rates() {
        for rate in [24, 25, 30, 50, 60, 44_100, 48_000, 96_000] {
            assert_eq!(TICKS_PER_SECOND % rate, 0, "{rate}");
        }
        // 23,976 = 24000/1001 ; 29,97 = 30000/1001 ; 59,94 = 60000/1001
        for num in [24_000, 30_000, 60_000] {
            assert_eq!((TICKS_PER_SECOND * 1001) % num, 0, "{num}");
        }
        // La limite JavaScript (2^53 - 1) autorise environ 150 jours de montage continu.
        assert!(MAX_SAFE_TICKS / TICKS_PER_SECOND / 86_400 >= 140);
    }

    #[test]
    fn unknown_fields_survive_a_round_trip_at_every_level() {
        let original = crate::testutil::document_with_unknown_fields();
        let doc: KitoDocument = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(doc.extra["topLevelFuture"]["nested"]["deep"][2], 3);
        assert_eq!(serde_json::to_value(&doc).unwrap(), original);
    }

    #[test]
    fn missing_optional_lists_default_to_empty() {
        let mut v = crate::testutil::document_with_unknown_fields();
        let clip = &mut v["timeline"]["sequences"][0]["tracks"][0]["clips"][0];
        let clip_obj = clip.as_object_mut().unwrap();
        clip_obj.remove("effects");
        clip_obj.remove("transitions");
        clip_obj.remove("keyframes");
        clip_obj.remove("sourceIn");
        let doc: KitoDocument = serde_json::from_value(v).unwrap();
        let clip = &doc.timeline.sequences[0].tracks[0].clips[0];
        assert!(
            clip.effects.is_empty() && clip.transitions.is_empty() && clip.keyframes.is_empty()
        );
        assert_eq!(clip.source_in, 0);
    }
}
