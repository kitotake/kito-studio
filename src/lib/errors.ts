import type { ErrorCode, SerializedAppError } from "@/types/errors";

const KNOWN_CODES: readonly ErrorCode[] = [
  "IO",
  "INVALID_INPUT",
  "NOT_FOUND",
  "ALREADY_EXISTS",
  "PERMISSION_DENIED",
  "DISK_FULL",
  "INVALID_FORMAT",
  "INCOMPATIBLE_VERSION",
  "INTERNAL",
  "UNAVAILABLE",
  "UNKNOWN"
];

/** Erreur applicative unique manipulée par l'interface. */
export class KitoError extends Error {
  readonly code: ErrorCode;

  constructor(code: ErrorCode, message: string, options?: { cause?: unknown }) {
    super(message, options);
    this.name = "KitoError";
    this.code = code;
  }
}

function isSerializedAppError(value: unknown): value is SerializedAppError {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Record<string, unknown>;
  return typeof candidate.code === "string" && typeof candidate.message === "string";
}

/**
 * Normalise n'importe quelle valeur levée (erreur Rust sérialisée, Error JS,
 * chaîne…) en `KitoError`. À utiliser dans tous les `catch`.
 */
export function toKitoError(value: unknown): KitoError {
  if (value instanceof KitoError) return value;

  if (isSerializedAppError(value)) {
    const code = KNOWN_CODES.find((known) => known === value.code) ?? "UNKNOWN";
    return new KitoError(code, value.message);
  }

  if (value instanceof Error) {
    return new KitoError("UNKNOWN", value.message, { cause: value });
  }

  if (typeof value === "string") {
    return new KitoError("UNKNOWN", value);
  }

  return new KitoError("UNKNOWN", "Erreur inconnue.", { cause: value });
}
