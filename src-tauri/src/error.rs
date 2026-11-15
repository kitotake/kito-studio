//! Gestion centralisée des erreurs.
//!
//! Convention : toute commande Tauri retourne [`AppResult<T>`]. L'erreur est
//! sérialisée en JSON `{ "code": "...", "message": "..." }` et convertie côté
//! interface en `KitoError` (voir `src/lib/errors.ts` : les codes doivent rester
//! synchronisés).

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt;
use std::io;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    /// Échec d'une opération sur le système de fichiers ou un processus.
    Io { context: String, source: io::Error },
    /// Donnée fournie par l'interface invalide.
    InvalidInput(String),
    /// Ressource demandée introuvable (fichier, projet, média…).
    NotFound(String),
    /// Erreur interne inattendue (bug, invariant violé).
    Internal(String),
}

impl AppError {
    /// Code stable lu par l'interface. Ne jamais renommer sans migrer le frontend.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Io { .. } => "IO",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Internal(_) => "INTERNAL",
        }
    }

    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { context, source } => write!(f, "{context} : {source}"),
            Self::InvalidInput(message) | Self::NotFound(message) | Self::Internal(message) => {
                f.write_str(message)
            }
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(source: io::Error) -> Self {
        Self::io("Erreur d'entrée/sortie", source)
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn serializes_to_code_and_message() {
        let json = serde_json::to_value(AppError::NotFound("Projet introuvable".into())).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "code": "NOT_FOUND", "message": "Projet introuvable" })
        );
    }

    #[test]
    fn every_variant_has_a_distinct_code() {
        let codes = [
            AppError::io("x", io::Error::other("y")).code(),
            AppError::InvalidInput(String::new()).code(),
            AppError::NotFound(String::new()).code(),
            AppError::Internal(String::new()).code(),
        ];
        let mut unique = codes.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), codes.len());
    }

    #[test]
    fn io_error_keeps_context_and_source() {
        let error = AppError::io("Lecture du projet", io::Error::other("disque plein"));
        assert_eq!(error.to_string(), "Lecture du projet : disque plein");
        assert!(error.source().is_some());
        assert_eq!(error.code(), "IO");
    }

    #[test]
    fn converts_from_std_io_error() {
        let error: AppError = io::Error::from(io::ErrorKind::PermissionDenied).into();
        assert_eq!(error.code(), "IO");
    }
}
