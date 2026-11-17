//! Contrôle de version et migrations du JSON brut (avant typage).
//!
//! - `formatVersion` : version d'écriture du fichier.
//! - `minReaderVersion` : plus ancienne version de Kito capable de le lire sans risque.
//!
//! Les migrations travaillent sur `serde_json::Value` : elles conservent donc
//! naturellement tous les champs qu'elles ne connaissent pas.

use super::model::{CURRENT_FORMAT_VERSION, FORMAT_NAME};
use crate::error::{AppError, AppResult};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationReport {
    /// Version trouvée dans le fichier.
    pub from_version: u32,
    /// Vrai si des étapes de migration ont été appliquées.
    pub migrated: bool,
    /// Vrai si le fichier vient d'une version plus récente mais reste lisible.
    pub newer_compatible: bool,
}

/// Une étape transforme un document de la version `from` vers `from + 1`.
pub type MigrationFn = fn(&mut Value) -> AppResult<()>;
pub type MigrationStep = (u32, MigrationFn);

/// Table des migrations réelles. Vide tant qu'il n'existe que la version 1.
pub const STEPS: &[MigrationStep] = &[];

pub fn migrate(value: &mut Value) -> AppResult<MigrationReport> {
    migrate_with(value, CURRENT_FORMAT_VERSION, STEPS)
}

fn read_version(value: &Value, key: &str) -> AppResult<Option<u32>> {
    match value.get(key) {
        None => Ok(None),
        Some(v) => v
            .as_u64()
            .filter(|n| (1..=u64::from(u32::MAX)).contains(n))
            .map(|n| Some(n as u32))
            .ok_or_else(|| {
                AppError::InvalidFormat(format!("« {key} » doit être un entier positif."))
            }),
    }
}

pub fn migrate_with(
    value: &mut Value,
    current: u32,
    steps: &[MigrationStep],
) -> AppResult<MigrationReport> {
    if !value.is_object() {
        return Err(AppError::InvalidFormat(
            "Le fichier ne contient pas un objet JSON de projet.".into(),
        ));
    }
    if value.get("format").and_then(Value::as_str) != Some(FORMAT_NAME) {
        return Err(AppError::InvalidFormat(
            "Ce fichier n'est pas un projet Kito Studio (champ « format » absent ou différent)."
                .into(),
        ));
    }
    let version = read_version(value, "formatVersion")?.ok_or_else(|| {
        AppError::InvalidFormat("Le champ « formatVersion » est manquant.".into())
    })?;
    let min_reader = read_version(value, "minReaderVersion")?.unwrap_or(version);

    if min_reader > current {
        return Err(AppError::IncompatibleVersion(format!(
            "Ce projet a été créé par une version plus récente de Kito Studio (format {version}, \
             lecture possible à partir du format {min_reader}). Cette version lit jusqu'au format {current}. \
             Le fichier n'a pas été modifié."
        )));
    }

    if version >= current {
        return Ok(MigrationReport {
            from_version: version,
            migrated: false,
            newer_compatible: version > current,
        });
    }

    value["minReaderVersion"] = Value::from(min_reader);
    let mut at = version;
    while at < current {
        let step = steps.iter().find(|(from, _)| *from == at).ok_or_else(|| {
            AppError::Internal(format!(
                "Migration manquante pour passer du format {at} au format {}.",
                at + 1
            ))
        })?;
        (step.1)(value)?;
        at += 1;
        value["formatVersion"] = Value::from(at);
    }
    Ok(MigrationReport {
        from_version: version,
        migrated: true,
        newer_compatible: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc(version: u64, min: Option<u64>) -> Value {
        let mut v =
            json!({ "format": "kito-project", "formatVersion": version, "keep": { "me": [1] } });
        if let Some(m) = min {
            v["minReaderVersion"] = json!(m);
        }
        v
    }

    fn v1_to_v2(v: &mut Value) -> AppResult<()> {
        let name = v["legacyName"].take();
        v["title"] = name;
        Ok(())
    }
    fn v2_to_v3(v: &mut Value) -> AppResult<()> {
        v["added"] = json!(true);
        v["minReaderVersion"] = json!(3);
        Ok(())
    }
    const FAKE: &[MigrationStep] = &[(1, v1_to_v2), (2, v2_to_v3)];

    #[test]
    fn current_version_is_untouched() {
        let mut v = doc(1, None);
        let r = migrate(&mut v).unwrap();
        assert_eq!(
            r,
            MigrationReport {
                from_version: 1,
                migrated: false,
                newer_compatible: false
            }
        );
    }

    #[test]
    fn rejects_non_kito_and_malformed_documents() {
        for mut v in [
            json!([]),
            json!({ "format": "other" }),
            json!({ "format": "kito-project" }),
        ] {
            assert_eq!(migrate(&mut v).unwrap_err().code(), "INVALID_FORMAT");
        }
        let mut v = doc(0, None);
        assert_eq!(migrate(&mut v).unwrap_err().code(), "INVALID_FORMAT");
        let mut v = json!({ "format": "kito-project", "formatVersion": "1" });
        assert_eq!(migrate(&mut v).unwrap_err().code(), "INVALID_FORMAT");
    }

    #[test]
    fn newer_but_compatible_is_accepted_and_flagged() {
        let mut v = doc(2, Some(1));
        let r = migrate(&mut v).unwrap();
        assert!(r.newer_compatible && !r.migrated);
        assert_eq!(v["formatVersion"], 2, "la version d'origine est conservée");
    }

    #[test]
    fn newer_and_incompatible_is_refused_without_changes() {
        let mut v = doc(5, Some(4));
        let before = v.clone();
        let err = migrate(&mut v).unwrap_err();
        assert_eq!(err.code(), "INCOMPATIBLE_VERSION");
        assert!(err.to_string().contains("version plus récente"));
        assert_eq!(v, before);
    }

    #[test]
    fn older_documents_are_migrated_step_by_step_keeping_unknown_data() {
        let mut v = doc(1, None);
        v["legacyName"] = json!("Ancien");
        let r = migrate_with(&mut v, 3, FAKE).unwrap();
        assert_eq!(
            r,
            MigrationReport {
                from_version: 1,
                migrated: true,
                newer_compatible: false
            }
        );
        assert_eq!(v["formatVersion"], 3);
        assert_eq!(v["title"], "Ancien");
        assert_eq!(v["added"], true);
        assert_eq!(v["minReaderVersion"], 3);
        assert_eq!(
            v["keep"],
            json!({ "me": [1] }),
            "données inconnues conservées"
        );
    }

    #[test]
    fn missing_migration_step_is_an_internal_error() {
        let mut v = doc(1, None);
        let err = migrate_with(&mut v, 3, &[(2, v2_to_v3)]).unwrap_err();
        assert_eq!(err.code(), "INTERNAL");
    }

    #[test]
    fn real_step_table_is_consistent_with_current_version() {
        // Pour chaque version < courante, une étape doit exister.
        for from in 1..CURRENT_FORMAT_VERSION {
            assert!(
                STEPS.iter().any(|(f, _)| *f == from),
                "étape {from} manquante"
            );
        }
    }
}
