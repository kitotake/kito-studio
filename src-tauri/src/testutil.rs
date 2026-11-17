//! Utilitaires de test (dossiers temporaires supprimés automatiquement).

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::{env, fs};

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(label: &str) -> Self {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!("kito-test-{label}-{}-{n}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Document complet contenant des champs inconnus à CHAQUE niveau.
pub fn document_with_unknown_fields() -> Value {
    json!({
        "format": "kito-project",
        "formatVersion": 1,
        "minReaderVersion": 1,
        "savedBy": { "app": "Kito Studio", "version": "9.9.9", "build": "x" },
        "project": {
            "id": "p1", "name": "N", "createdAt": "2026-01-01T00:00:00Z",
            "modifiedAt": "2026-01-01T00:00:00Z", "futureProjectField": [1, 2, { "a": null }]
        },
        "settings": {
            "video": {
                "width": 1920, "height": 1080,
                "frameRate": { "num": 30, "den": 1, "dropFrame": false },
                "colorSpace": "rec709"
            },
            "audio": { "sampleRate": 48000, "channels": 2 },
            "proxy": { "enabled": true }
        },
        "media": [{
            "id": "m1", "kind": "video", "path": "/a/b.mp4", "relativePath": "../a/b.mp4",
            "size": 10, "modifiedAt": "2026-01-01T00:00:00Z", "proxyPath": "/p/b.mp4"
        }],
        "timeline": {
            "ticksPerSecond": 705600000,
            "markers": [{ "at": 5 }],
            "sequences": [{
                "id": "s1", "name": "S", "color": "#fff",
                "tracks": [{
                    "id": "t1", "kind": "video", "name": "V1", "locked": true,
                    "clips": [{
                        "id": "c1", "mediaId": "m1", "start": 0, "duration": 705600000,
                        "sourceIn": 0, "label": "plan 1",
                        "effects": [{ "type": "blur", "params": { "radius": 4.5 }, "future": 1 }],
                        "transitions": [{ "type": "cross-dissolve", "duration": 100 }],
                        "keyframes": [{ "type": "opacity", "points": [[0, 0.0], [10, 1.0]] }]
                    }]
                }]
            }]
        },
        "topLevelFuture": { "nested": { "deep": [true, "x", 3] } }
    })
}
