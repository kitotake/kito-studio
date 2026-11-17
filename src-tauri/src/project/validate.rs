//! Validation structurelle et sémantique d'un document déjà typé.
//!
//! Sécurité : les effets, transitions et keyframes doivent être des objets JSON
//! déclaratifs avec un `type` en identifiant strict. Rien n'est jamais exécuté.

use super::model::*;
use crate::error::{AppError, AppResult};
use serde_json::Value;
use std::collections::HashSet;

pub const MAX_JSON_DEPTH: usize = 16;
const MAX_ID_CHARS: usize = 128;
const MAX_PATH_CHARS: usize = 4096;

fn bad<T>(path: &str, message: impl AsRef<str>) -> AppResult<T> {
    Err(AppError::InvalidFormat(format!(
        "{path} : {}",
        message.as_ref()
    )))
}

fn check_id(path: &str, id: &str) -> AppResult<()> {
    if id.is_empty() || id.chars().count() > MAX_ID_CHARS || id.chars().any(|c| c.is_control()) {
        return bad(path, "identifiant vide, trop long ou invalide");
    }
    Ok(())
}

fn check_label(path: &str, value: &str, max: usize) -> AppResult<()> {
    if value.trim().is_empty()
        || value.chars().count() > max
        || value.chars().any(|c| c.is_control())
    {
        return bad(
            path,
            format!("texte vide, trop long (max {max}) ou invalide"),
        );
    }
    Ok(())
}

fn check_path_string(path: &str, value: &str) -> AppResult<()> {
    if value.is_empty() || value.chars().count() > MAX_PATH_CHARS || value.contains('\0') {
        return bad(path, "chemin vide, trop long ou invalide");
    }
    Ok(())
}

fn check_range(path: &str, value: i64, min: i64, max: i64) -> AppResult<()> {
    if value < min || value > max {
        return bad(
            path,
            format!("doit être compris entre {min} et {max} (valeur : {value})"),
        );
    }
    Ok(())
}

/// Identifiant de type déclaratif : `blur`, `cross-dissolve`, `color.lut_3d`…
pub fn is_valid_type_id(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    value.len() <= 64
        && first.is_ascii_lowercase()
        && chars
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '-'))
}

pub fn json_depth(value: &Value) -> usize {
    match value {
        Value::Array(items) => 1 + items.iter().map(json_depth).max().unwrap_or(0),
        Value::Object(map) => 1 + map.values().map(json_depth).max().unwrap_or(0),
        _ => 0,
    }
}

fn check_declarative(path: &str, items: &[Value]) -> AppResult<()> {
    for (index, item) in items.iter().enumerate() {
        let here = format!("{path}[{index}]");
        let Value::Object(map) = item else {
            return bad(
                &here,
                "doit être un objet déclaratif { \"type\": \"...\", ... }",
            );
        };
        match map.get("type") {
            Some(Value::String(t)) if is_valid_type_id(t) => {}
            _ => {
                return bad(
                    &here,
                    "le champ « type » est obligatoire (identifiant en minuscules)",
                )
            }
        }
        if json_depth(item) > MAX_JSON_DEPTH {
            return bad(
                &here,
                format!("imbrication trop profonde (max {MAX_JSON_DEPTH})"),
            );
        }
    }
    Ok(())
}

pub fn validate_video(width: u32, height: u32, num: u32, den: u32) -> AppResult<()> {
    check_range("settings.video.width", width.into(), 16, 16_384)?;
    check_range("settings.video.height", height.into(), 16, 16_384)?;
    check_range("settings.video.frameRate.num", num.into(), 1, 1_000)?;
    check_range("settings.video.frameRate.den", den.into(), 1, 1_001)?;
    Ok(())
}

pub fn validate_sample_rate(rate: u32) -> AppResult<()> {
    check_range("settings.audio.sampleRate", rate.into(), 8_000, 384_000)
}

pub fn validate(doc: &KitoDocument) -> AppResult<()> {
    check_label("project.name", &doc.project.name, 200)?;
    check_id("project.id", &doc.project.id)?;

    let v = &doc.settings.video;
    validate_video(v.width, v.height, v.frame_rate.num, v.frame_rate.den)?;
    validate_sample_rate(doc.settings.audio.sample_rate)?;

    if doc.timeline.ticks_per_second != TICKS_PER_SECOND {
        return bad(
            "timeline.ticksPerSecond",
            format!("doit valoir {TICKS_PER_SECOND} pour ce format"),
        );
    }

    let mut media_ids = HashSet::new();
    for (i, m) in doc.media.iter().enumerate() {
        let here = format!("media[{i}]");
        check_id(&format!("{here}.id"), &m.id)?;
        if !media_ids.insert(m.id.as_str()) {
            return bad(
                &format!("{here}.id"),
                format!("identifiant « {} » en double", m.id),
            );
        }
        check_label(&format!("{here}.kind"), &m.kind, 32)?;
        check_path_string(&format!("{here}.path"), &m.path)?;
        if let Some(rel) = &m.relative_path {
            check_path_string(&format!("{here}.relativePath"), rel)?;
        }
    }

    let mut ids: HashSet<&str> = HashSet::new();
    for (si, seq) in doc.timeline.sequences.iter().enumerate() {
        let sp = format!("timeline.sequences[{si}]");
        check_id(&format!("{sp}.id"), &seq.id)?;
        if !ids.insert(&seq.id) {
            return bad(&format!("{sp}.id"), "identifiant en double");
        }
        check_label(&format!("{sp}.name"), &seq.name, 200)?;
        for (ti, track) in seq.tracks.iter().enumerate() {
            let tp = format!("{sp}.tracks[{ti}]");
            check_id(&format!("{tp}.id"), &track.id)?;
            if !ids.insert(&track.id) {
                return bad(&format!("{tp}.id"), "identifiant en double");
            }
            check_label(&format!("{tp}.kind"), &track.kind, 32)?;
            check_label(&format!("{tp}.name"), &track.name, 200)?;
            for (ci, clip) in track.clips.iter().enumerate() {
                let cp = format!("{tp}.clips[{ci}]");
                check_id(&format!("{cp}.id"), &clip.id)?;
                if !ids.insert(&clip.id) {
                    return bad(&format!("{cp}.id"), "identifiant en double");
                }
                if let Some(media_id) = &clip.media_id {
                    if !media_ids.contains(media_id.as_str()) {
                        return bad(
                            &format!("{cp}.mediaId"),
                            format!("média « {media_id} » inconnu"),
                        );
                    }
                }
                check_range(&format!("{cp}.start"), clip.start, 0, MAX_SAFE_TICKS)?;
                check_range(&format!("{cp}.duration"), clip.duration, 0, MAX_SAFE_TICKS)?;
                check_range(&format!("{cp}.sourceIn"), clip.source_in, 0, MAX_SAFE_TICKS)?;
                if clip
                    .start
                    .checked_add(clip.duration)
                    .is_none_or(|end| end > MAX_SAFE_TICKS)
                {
                    return bad(&cp, "position + durée dépasse la limite supportée");
                }
                check_declarative(&format!("{cp}.effects"), &clip.effects)?;
                check_declarative(&format!("{cp}.transitions"), &clip.transitions)?;
                check_declarative(&format!("{cp}.keyframes"), &clip.keyframes)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_doc() -> KitoDocument {
        let mut doc = KitoDocument::new(&NewDocument {
            name: "T".into(),
            width: 1920,
            height: 1080,
            frame_rate_num: 30,
            frame_rate_den: 1,
            sample_rate: 48_000,
        });
        doc.media.push(MediaRef {
            id: "m1".into(),
            kind: "video".into(),
            path: "/a.mp4".into(),
            relative_path: None,
            size: None,
            modified_at: None,
            extra: Extra::new(),
        });
        doc.timeline.sequences[0].tracks[0].clips.push(Clip {
            id: "c1".into(),
            media_id: Some("m1".into()),
            start: 0,
            duration: 100,
            source_in: 0,
            effects: vec![json!({ "type": "blur", "params": { "radius": 3 } })],
            transitions: vec![],
            keyframes: vec![],
            extra: Extra::new(),
        });
        doc
    }

    fn expect_err(doc: &KitoDocument, needle: &str) {
        let err = validate(doc).expect_err(needle);
        assert_eq!(err.code(), "INVALID_FORMAT");
        assert!(
            err.to_string().contains(needle),
            "« {err} » ne contient pas « {needle} »"
        );
    }

    #[test]
    fn accepts_a_valid_document() {
        validate(&valid_doc()).unwrap();
    }

    #[test]
    fn rejects_out_of_range_settings() {
        let mut d = valid_doc();
        d.settings.video.width = 8;
        expect_err(&d, "settings.video.width");
        let mut d = valid_doc();
        d.settings.video.frame_rate.den = 0;
        expect_err(&d, "frameRate.den");
        let mut d = valid_doc();
        d.settings.audio.sample_rate = 1;
        expect_err(&d, "sampleRate");
        let mut d = valid_doc();
        d.timeline.ticks_per_second = 1000;
        expect_err(&d, "ticksPerSecond");
    }

    #[test]
    fn rejects_dangling_or_duplicate_references() {
        let mut d = valid_doc();
        d.timeline.sequences[0].tracks[0].clips[0].media_id = Some("nope".into());
        expect_err(&d, "média « nope » inconnu");
        let mut d = valid_doc();
        let dup = d.media[0].clone();
        d.media.push(dup);
        expect_err(&d, "en double");
        let mut d = valid_doc();
        d.timeline.sequences[0].tracks[1].id = d.timeline.sequences[0].tracks[0].id.clone();
        expect_err(&d, "en double");
    }

    #[test]
    fn rejects_bad_times() {
        let mut d = valid_doc();
        d.timeline.sequences[0].tracks[0].clips[0].start = -1;
        expect_err(&d, ".start");
        let mut d = valid_doc();
        d.timeline.sequences[0].tracks[0].clips[0].start = MAX_SAFE_TICKS;
        d.timeline.sequences[0].tracks[0].clips[0].duration = 1;
        expect_err(&d, "dépasse la limite");
    }

    #[test]
    fn declarative_entries_must_be_typed_objects() {
        for bad_entry in [
            json!("rm -rf /"),
            json!(42),
            json!({ "params": {} }),
            json!({ "type": "../../evil.exe" }),
            json!({ "type": "Blur" }),
            json!({ "type": "" }),
            json!({ "type": 3 }),
        ] {
            let mut d = valid_doc();
            d.timeline.sequences[0].tracks[0].clips[0].transitions = vec![bad_entry.clone()];
            expect_err(&d, "transitions[0]");
        }
    }

    #[test]
    fn script_like_content_is_inert_data_not_rejected_nor_run() {
        // Un champ « command » est une simple donnée : conservée, jamais interprétée.
        let mut d = valid_doc();
        d.timeline.sequences[0].tracks[0].clips[0].effects =
            vec![json!({ "type": "custom", "command": "calc.exe", "script": "alert(1)" })];
        validate(&d).unwrap();
    }

    #[test]
    fn rejects_overly_deep_nesting() {
        let mut deep = json!(1);
        for _ in 0..(MAX_JSON_DEPTH + 2) {
            deep = json!({ "x": deep });
        }
        let mut d = valid_doc();
        d.timeline.sequences[0].tracks[0].clips[0].keyframes =
            vec![json!({ "type": "k", "data": deep })];
        expect_err(&d, "imbrication");
    }

    #[test]
    fn type_id_grammar() {
        for ok in ["blur", "cross-dissolve", "color.lut_3d", "a1"] {
            assert!(is_valid_type_id(ok), "{ok}");
        }
        for bad in ["", "Blur", "1abc", "a b", "a/b", "é", &"a".repeat(65)] {
            assert!(!is_valid_type_id(bad), "{bad}");
        }
    }
}
