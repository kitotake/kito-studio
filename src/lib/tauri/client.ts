import { invoke, isTauri } from "@tauri-apps/api/core";
import { KitoError, toKitoError } from "@/lib/errors";

/** Vrai uniquement dans l'application Tauri (faux dans un navigateur). */
export function isNativeRuntime(): boolean {
  return isTauri();
}

/**
 * Seul point d'appel vers les commandes Rust.
 * Toute erreur est convertie en `KitoError`.
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  if (!isTauri()) {
    throw new KitoError(
      "UNAVAILABLE",
      "Cette fonction nécessite l'application native Kito Studio."
    );
  }

  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toKitoError(error);
  }
}
