import { listen } from "@tauri-apps/api/event";
import type { AutosaveFailedEvent, AutosavedEvent } from "@/types/project";
import { isNativeRuntime } from "./client";

const noop = () => {};

/** Noms d'événements émis par le backend (voir `commands/autosave.rs`). */
export const EVENT_AUTOSAVED = "project:autosaved";
export const EVENT_AUTOSAVE_FAILED = "project:autosave-failed";

/** S'abonne aux résultats de la sauvegarde automatique. Retourne la fonction de désabonnement. */
export async function listenAutosave(handlers: {
  onSaved: (event: AutosavedEvent) => void;
  onFailed: (event: AutosaveFailedEvent) => void;
}): Promise<() => void> {
  if (!isNativeRuntime()) return noop;
  const [offSaved, offFailed] = await Promise.all([
    listen<AutosavedEvent>(EVENT_AUTOSAVED, (e) => handlers.onSaved(e.payload)),
    listen<AutosaveFailedEvent>(EVENT_AUTOSAVE_FAILED, (e) => handlers.onFailed(e.payload))
  ]);
  return () => {
    offSaved();
    offFailed();
  };
}
