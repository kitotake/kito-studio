/**
 * Codes d'erreur de l'application.
 *
 * - Émis par le backend Rust (voir `src-tauri/src/error.rs`, les deux listes doivent
 *   rester synchronisées) : IO, INVALID_INPUT, NOT_FOUND, ALREADY_EXISTS,
 *   PERMISSION_DENIED, DISK_FULL, INVALID_FORMAT, INCOMPATIBLE_VERSION, INTERNAL.
 * - Produits côté interface : UNAVAILABLE, UNKNOWN.
 */
export type ErrorCode =
  | "IO"
  | "INVALID_INPUT"
  | "NOT_FOUND"
  | "ALREADY_EXISTS"
  | "PERMISSION_DENIED"
  | "DISK_FULL"
  | "INVALID_FORMAT"
  | "INCOMPATIBLE_VERSION"
  | "INTERNAL"
  | "UNAVAILABLE"
  | "UNKNOWN";

/** Forme JSON d'une erreur renvoyée par une commande Tauri. */
export type SerializedAppError = {
  code: string;
  message: string;
};
