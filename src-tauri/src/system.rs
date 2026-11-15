//! Informations sur l'environnement d'exécution (aucune dépendance à Tauri).

use serde::Serialize;
use std::env;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
    pub app_name: String,
    pub app_version: String,
    pub os: String,
    pub architecture: String,
}

pub fn runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        app_name: "Kito Studio".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os: env::consts::OS.to_string(),
        architecture: env::consts::ARCH.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_camel_case_keys() {
        let json = serde_json::to_value(runtime_info()).unwrap();
        for key in ["appName", "appVersion", "os", "architecture"] {
            assert!(json.get(key).is_some(), "clé manquante : {key}");
        }
    }
}
