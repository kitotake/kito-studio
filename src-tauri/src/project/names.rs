//! Validation du nom d'un projet (il devient un nom de dossier et de fichier).

use crate::error::{AppError, AppResult};

pub const MAX_NAME_CHARS: usize = 100;
const FORBIDDEN: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
const RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Retourne le nom nettoyé (espaces de bord retirés) ou une erreur lisible.
pub fn validate_project_name(raw: &str) -> AppResult<String> {
    let name = raw.trim();
    let invalid = |why: &str| {
        Err(AppError::InvalidInput(format!(
            "Nom de projet invalide : {why}."
        )))
    };

    if name.is_empty() {
        return invalid("le nom est vide");
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return invalid(&format!("{MAX_NAME_CHARS} caractères maximum"));
    }
    if let Some(c) = name.chars().find(|c| FORBIDDEN.contains(c)) {
        return invalid(&format!("le caractère « {c} » est interdit"));
    }
    if name.chars().any(|c| c.is_control()) {
        return invalid("caractères de contrôle interdits");
    }
    if name == "." || name == ".." || name.ends_with('.') {
        return invalid("le nom ne peut pas se terminer par un point");
    }
    let base = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    if RESERVED.contains(&base.as_str()) {
        return invalid("ce nom est réservé par Windows");
    }
    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_names_and_trims() {
        assert_eq!(
            validate_project_name("  Mon film 2026 ").unwrap(),
            "Mon film 2026"
        );
        assert_eq!(validate_project_name("Été – vidéo").unwrap(), "Été – vidéo");
    }

    #[test]
    fn rejects_bad_names() {
        for bad in [
            "",
            "   ",
            "a/b",
            "a\\b",
            "a:b",
            "a*b",
            "a?b",
            "a<b",
            "a|b",
            "a\"b",
            "nom.",
            "..",
            "con",
            "NUL",
            "com1.txt",
            "lpt9",
            "a\u{0007}b",
        ] {
            let err = validate_project_name(bad).expect_err(bad);
            assert_eq!(err.code(), "INVALID_INPUT", "{bad:?}");
        }
        assert!(validate_project_name(&"x".repeat(101)).is_err());
        assert!(validate_project_name(&"x".repeat(100)).is_ok());
    }

    #[test]
    fn reserved_check_only_applies_to_whole_base_name() {
        assert!(validate_project_name("console").is_ok());
        assert!(validate_project_name("auxiliaire").is_ok());
    }
}
