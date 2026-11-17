import { getCurrentWindow } from "@tauri-apps/api/window";
import { isNativeRuntime } from "./client";

const noop = () => {};

/**
 * Intercepte la demande de fermeture de la fenêtre. Le gestionnaire reçoit une fonction
 * `prevent` : l'appeler empêche la fermeture (à rouvrir ensuite avec `destroyWindow`).
 */
export async function onCloseRequested(handler: (prevent: () => void) => void): Promise<() => void> {
  if (!isNativeRuntime()) return noop;
  return getCurrentWindow().onCloseRequested((event) => handler(() => event.preventDefault()));
}

export async function destroyWindow(): Promise<void> {
  if (isNativeRuntime()) await getCurrentWindow().destroy();
}
