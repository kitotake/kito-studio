import type {
  AutosaveInfo,
  NewProjectInput,
  OpenOutcome,
  ProjectSnapshot,
  RecentProject,
  RelinkReport,
  SettingsInput
} from "@/types/project";
import { invokeCommand } from "./client";

export const projectApi = {
  create: (input: NewProjectInput) =>
    invokeCommand<ProjectSnapshot>("create_project", { input }),
  open: (path: string) => invokeCommand<OpenOutcome>("open_project", { path }),
  current: () => invokeCommand<ProjectSnapshot | null>("get_current_project"),
  save: () => invokeCommand<ProjectSnapshot>("save_project"),
  saveAs: (path: string) => invokeCommand<ProjectSnapshot>("save_project_as", { path }),
  updateSettings: (input: SettingsInput) =>
    invokeCommand<ProjectSnapshot>("update_project_settings", { input }),
  close: (discardChanges: boolean) =>
    invokeCommand<void>("close_project", { discardChanges }),
  listAutosaves: (projectPath: string) =>
    invokeCommand<AutosaveInfo[]>("list_autosaves", { projectPath }),
  restoreAutosave: (projectPath: string, autosavePath: string) =>
    invokeCommand<ProjectSnapshot>("restore_autosave", { projectPath, autosavePath }),
  relinkMedia: (mediaId: string, filePath: string) =>
    invokeCommand<ProjectSnapshot>("relink_media", { mediaId, filePath }),
  relinkMediaInFolder: (folderPath: string) =>
    invokeCommand<RelinkReport>("relink_media_in_folder", { folderPath }),
  recent: () => invokeCommand<RecentProject[]>("get_recent_projects"),
  removeRecent: (path: string) => invokeCommand<void>("remove_recent_project", { path }),
  clearRecent: () => invokeCommand<void>("clear_recent_projects")
} as const;
