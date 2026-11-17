import { open, save } from "@tauri-apps/plugin-dialog";
import { KitoError, toKitoError } from "@/lib/errors";
import { isNativeRuntime } from "./client";

const PROJECT_FILTER = [{ name: "Projet Kito Studio", extensions: ["kito"] }];

async function run<T>(action: () => Promise<T>): Promise<T> {
  if (!isNativeRuntime()) {
    throw new KitoError("UNAVAILABLE", "Cette fonction nécessite l'application native Kito Studio.");
  }
  try {
    return await action();
  } catch (error) {
    throw toKitoError(error);
  }
}

async function pick(options: Parameters<typeof open>[0]): Promise<string | null> {
  const result = await run(() => open({ multiple: false, ...options }));
  return typeof result === "string" ? result : null;
}

/** Sélecteurs de fichiers natifs. Retournent `null` si l'utilisateur annule. */
export const dialogs = {
  pickFolder: (title: string) => pick({ directory: true, title }),
  pickProjectFile: () => pick({ title: "Ouvrir un projet", filters: PROJECT_FILTER }),
  pickMediaFile: (title: string) => pick({ title }),
  pickSaveProjectPath: (defaultPath?: string) =>
    run(() => save({ title: "Enregistrer le projet sous", defaultPath, filters: PROJECT_FILTER }))
} as const;
