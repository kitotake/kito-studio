import { dialogs } from "./dialogs";
import { preferencesApi } from "./preferences";
import { projectApi } from "./project";
import { systemApi } from "./system";

/**
 * API native typée : une fonction par commande Rust, regroupée par domaine.
 * Les composants React n'utilisent jamais `invoke` directement.
 */
export const nativeApi = {
  ...systemApi,
  project: projectApi,
  preferences: preferencesApi,
  dialogs
} as const;
