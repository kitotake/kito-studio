/**
 * Codes d'erreur de l'application.
 *
 * - IO, INVALID_INPUT, NOT_FOUND, INTERNAL : émis par le backend Rust
 *   (voir `src-tauri/src/error.rs`, les deux listes doivent rester synchronisées).
 * - UNAVAILABLE, UNKNOWN : produits côté interface.
 */
export type ErrorCode =
  | "IO"
  | "INVALID_INPUT"
  | "NOT_FOUND"
  | "INTERNAL"
  | "UNAVAILABLE"
  | "UNKNOWN";

/** Forme JSON d'une erreur renvoyée par une commande Tauri. */
export type SerializedAppError = {
  code: string;
  message: string;
};
