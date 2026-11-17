export type MediaStatus = "ok" | "changed" | "missing";

export type MediaView = {
  id: string;
  kind: string;
  fileName: string;
  path: string;
  status: MediaStatus;
  resolvedPath: string | null;
};

/** Instantané du projet ouvert, envoyé par le backend (qui détient le document). */
export type ProjectSnapshot = {
  id: string;
  name: string;
  path: string;
  directory: string;
  width: number;
  height: number;
  frameRateNum: number;
  frameRateDen: number;
  sampleRate: number;
  createdAt: string;
  modifiedAt: string;
  dirty: boolean;
  lastSavedAt: string | null;
  formatVersion: number;
  migratedFrom: number | null;
  newerFormat: boolean;
  media: MediaView[];
};

export type NewProjectInput = {
  name: string;
  parentDir: string;
  width: number;
  height: number;
  frameRateNum: number;
  frameRateDen: number;
  sampleRate: number;
};

export type SettingsInput = {
  name: string;
  width: number;
  height: number;
  frameRateNum: number;
  frameRateDen: number;
  sampleRate: number;
};

export type RecoveryOffer = {
  autosavePath: string;
  autosavedAt: string;
};

export type OpenOutcome = {
  snapshot: ProjectSnapshot;
  recovery: RecoveryOffer | null;
};

export type AutosaveInfo = {
  path: string;
  modifiedAt: string;
};

export type RelinkReport = {
  relinked: number;
  snapshot: ProjectSnapshot;
};

export type RecentProject = {
  path: string;
  name: string;
  lastOpenedAt: string;
  exists: boolean;
};

export type AutosavePreferences = {
  enabled: boolean;
  intervalMinutes: number;
  keepCopies: number;
};

export type Preferences = {
  version: number;
  autosave: AutosavePreferences;
};

export type AutosavedEvent = { savedAt: string; path: string };
export type AutosaveFailedEvent = { message: string };
