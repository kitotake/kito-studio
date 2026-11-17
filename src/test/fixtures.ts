import type { MediaView, ProjectSnapshot, RecentProject } from "@/types/project";

export function makeSnapshot(overrides: Partial<ProjectSnapshot> = {}): ProjectSnapshot {
  return {
    id: "project-1",
    name: "Mon film",
    path: "/projets/Mon film/Mon film.kito",
    directory: "/projets/Mon film",
    width: 1920,
    height: 1080,
    frameRateNum: 30,
    frameRateDen: 1,
    sampleRate: 48_000,
    createdAt: "2026-11-15T10:00:00Z",
    modifiedAt: "2026-11-15T10:00:00Z",
    dirty: false,
    lastSavedAt: "2026-11-15T10:00:00Z",
    formatVersion: 1,
    migratedFrom: null,
    newerFormat: false,
    media: [],
    ...overrides
  };
}

export function makeMedia(overrides: Partial<MediaView> = {}): MediaView {
  return {
    id: "m1",
    kind: "video",
    fileName: "plan1.mp4",
    path: "/rushes/plan1.mp4",
    status: "ok",
    resolvedPath: "/rushes/plan1.mp4",
    ...overrides
  };
}

export function makeRecent(overrides: Partial<RecentProject> = {}): RecentProject {
  return {
    path: "/projets/Mon film/Mon film.kito",
    name: "Mon film",
    lastOpenedAt: "2026-11-15T10:00:00Z",
    exists: true,
    ...overrides
  };
}
