import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { toKitoError } from "@/lib/errors";
import { listenAutosave, nativeApi } from "@/lib/tauri";
import type { ProjectSnapshot, RecoveryOffer } from "@/types/project";
import { ProjectContext, type BusyState, type ProjectContextValue } from "./ProjectContext";
import { describeProjectError, type DescribedError, type ProjectErrorContext } from "./projectErrors";
import { saveLastLocation } from "./lastLocation";

/** Détient le projet ouvert (instantané du backend), l'état d'occupation et la dernière erreur. */
export function ProjectProvider({ children }: { children: ReactNode }) {
  const [project, setProject] = useState<ProjectSnapshot | null>(null);
  const [busy, setBusy] = useState<BusyState>("idle");
  const [error, setError] = useState<DescribedError | null>(null);
  const [recovery, setRecovery] = useState<RecoveryOffer | null>(null);
  const [lastAutosaveAt, setLastAutosaveAt] = useState<string | null>(null);
  const [autosaveError, setAutosaveError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    listenAutosave({
      onSaved: (event) => {
        setLastAutosaveAt(event.savedAt);
        setAutosaveError(null);
      },
      onFailed: (event) => setAutosaveError(event.message)
    })
      .then((off) => (cancelled ? off() : (unlisten = off)))
      .catch(() => undefined);
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const reportError = useCallback((cause: unknown, context: ProjectErrorContext) => {
    setError(describeProjectError(toKitoError(cause), context));
  }, []);

  const dismissError = useCallback(() => setError(null), []);

  /** Exécute une action native avec état « occupé » et conversion des erreurs. */
  const run = useCallback(
    async <T,>(state: BusyState, context: ProjectErrorContext, action: () => Promise<T>) => {
      setBusy(state);
      setError(null);
      try {
        return await action();
      } catch (cause) {
        reportError(cause, context);
        return null;
      } finally {
        setBusy("idle");
      }
    },
    [reportError]
  );

  const adopt = useCallback((snapshot: ProjectSnapshot, offer: RecoveryOffer | null = null) => {
    setProject((previous) => {
      if (previous?.id !== snapshot.id) {
        setLastAutosaveAt(null);
        setAutosaveError(null);
      }
      return snapshot;
    });
    setRecovery(offer);
  }, []);

  const value = useMemo<ProjectContextValue>(
    () => ({
      project,
      busy,
      error,
      recovery,
      lastAutosaveAt,
      autosaveError,

      createProject: async (input) => {
        const snapshot = await run("creating", "create", () => nativeApi.project.create(input));
        if (!snapshot) return false;
        saveLastLocation(input.parentDir);
        adopt(snapshot);
        return true;
      },

      openProject: async (path) => {
        const outcome = await run("opening", "open", () => nativeApi.project.open(path));
        if (!outcome) return false;
        adopt(outcome.snapshot, outcome.recovery);
        return true;
      },

      saveProject: async () => {
        const snapshot = await run("saving", "save", () => nativeApi.project.save());
        if (!snapshot) return false;
        adopt(snapshot);
        return true;
      },

      saveProjectAs: async (path) => {
        const snapshot = await run("saving", "save", () => nativeApi.project.saveAs(path));
        if (!snapshot) return false;
        adopt(snapshot);
        return true;
      },

      updateSettings: async (input) => {
        const snapshot = await run("updating", "settings", () => nativeApi.project.updateSettings(input));
        if (!snapshot) return false;
        adopt(snapshot);
        return true;
      },

      closeProject: async (discardChanges) => {
        const done = await run("idle", "other", async () => {
          await nativeApi.project.close(discardChanges);
          return true;
        });
        if (!done) return false;
        setProject(null);
        setRecovery(null);
        setLastAutosaveAt(null);
        setAutosaveError(null);
        return true;
      },

      relinkMedia: async (mediaId, filePath) => {
        const snapshot = await run("relinking", "relink", () => nativeApi.project.relinkMedia(mediaId, filePath));
        if (!snapshot) return false;
        adopt(snapshot);
        return true;
      },

      relinkMediaInFolder: async (folderPath) => {
        const report = await run("relinking", "relink", () => nativeApi.project.relinkMediaInFolder(folderPath));
        if (!report) return null;
        adopt(report.snapshot);
        return report.relinked;
      },

      acceptRecovery: async () => {
        if (!project || !recovery) return false;
        const snapshot = await run("opening", "open", () =>
          nativeApi.project.restoreAutosave(project.path, recovery.autosavePath)
        );
        if (!snapshot) return false;
        adopt(snapshot);
        return true;
      },

      restoreAutosave: async (projectPath, autosavePath) => {
        const snapshot = await run("opening", "open", () =>
          nativeApi.project.restoreAutosave(projectPath, autosavePath)
        );
        if (!snapshot) return false;
        adopt(snapshot);
        return true;
      },

      declineRecovery: () => setRecovery(null),
      reportError,
      dismissError
    }),
    [project, busy, error, recovery, lastAutosaveAt, autosaveError, run, adopt, reportError, dismissError]
  );

  return <ProjectContext value={value}>{children}</ProjectContext>;
}
