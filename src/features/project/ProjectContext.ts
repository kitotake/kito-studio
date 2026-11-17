import { createContext } from "react";
import type { NewProjectInput, ProjectSnapshot, RecoveryOffer, SettingsInput } from "@/types/project";
import type { DescribedError, ProjectErrorContext } from "./projectErrors";

export type BusyState = "idle" | "creating" | "opening" | "saving" | "updating" | "relinking";

export type ProjectContextValue = {
  project: ProjectSnapshot | null;
  busy: BusyState;
  error: DescribedError | null;
  /** Copie automatique proposée à l'ouverture (après un arrêt brutal). */
  recovery: RecoveryOffer | null;
  lastAutosaveAt: string | null;
  autosaveError: string | null;
  createProject: (input: NewProjectInput) => Promise<boolean>;
  openProject: (path: string) => Promise<boolean>;
  saveProject: () => Promise<boolean>;
  saveProjectAs: (path: string) => Promise<boolean>;
  updateSettings: (input: SettingsInput) => Promise<boolean>;
  closeProject: (discardChanges: boolean) => Promise<boolean>;
  relinkMedia: (mediaId: string, filePath: string) => Promise<boolean>;
  /** Nombre de médias reliés, ou `null` en cas d'erreur. */
  relinkMediaInFolder: (folderPath: string) => Promise<number | null>;
  acceptRecovery: () => Promise<boolean>;
  /** Restaure une copie automatique d'un projet qui ne s'ouvre plus (fichier corrompu ou supprimé). */
  restoreAutosave: (projectPath: string, autosavePath: string) => Promise<boolean>;
  declineRecovery: () => void;
  reportError: (error: unknown, context: ProjectErrorContext) => void;
  dismissError: () => void;
};

export const ProjectContext = createContext<ProjectContextValue | null>(null);
