import { useCallback, useState } from "react";
import { Sidebar } from "@/components/layout/Sidebar";
import { Topbar } from "@/components/layout/Topbar";
import { Notice } from "@/components/ui/Notice";
import { EditorScreen } from "@/features/editor/EditorScreen";
import { HomeScreen } from "@/features/home/HomeScreen";
import { CreateProjectDialog } from "@/features/project/CreateProjectDialog";
import { MissingMediaDialog } from "@/features/project/MissingMediaDialog";
import { PreferencesDialog } from "@/features/project/PreferencesDialog";
import { ProjectSettingsDialog } from "@/features/project/ProjectSettingsDialog";
import { ProjectToolbar } from "@/features/project/ProjectToolbar";
import { RecoveryDialog } from "@/features/project/RecoveryDialog";
import { SaveStatus } from "@/features/project/SaveStatus";
import { UnsavedChangesDialog } from "@/features/project/UnsavedChangesDialog";
import { useCloseGuard } from "@/features/project/useCloseGuard";
import { useProject } from "@/features/project/useProject";
import { useProjectShortcuts } from "@/features/project/useProjectShortcuts";
import { useRecentProjects } from "@/features/project/useRecentProjects";
import { useUnsavedGuard } from "@/features/project/useUnsavedGuard";
import { destroyWindow, nativeApi } from "@/lib/tauri";
import { formatDateTime } from "@/features/project/format";
import type { AutosaveInfo, MediaView, NewProjectInput, SettingsInput } from "@/types/project";
import { useWorkspace } from "./useWorkspace";

/** Erreurs affichées dans leur dialogue plutôt que dans le bandeau. */
const DIALOG_CONTEXTS = ["create", "settings", "relink"];

export function Workspace() {
  const p = useProject();
  const ws = useWorkspace();
  const { project } = p;
  const [mediaDismissedFor, setMediaDismissedFor] = useState<string | null>(null);
  const [infoDismissedFor, setInfoDismissedFor] = useState<string | null>(null);
  /** Copie automatique proposée quand un projet ne s'ouvre pas (corrompu ou supprimé). */
  const [rescue, setRescue] = useState<{ projectPath: string; autosave: AutosaveInfo } | null>(null);
  const recents = useRecentProjects(project ? `${project.path}|${project.lastSavedAt}` : null);

  const guard = useUnsavedGuard({
    isDirty: project?.dirty ?? false,
    save: p.saveProject,
    discard: () => p.closeProject(true)
  });

  const showEditor = ws.view === "editor" && project !== null;
  const unresolved: MediaView[] = project?.media.filter((m) => m.status !== "ok") ?? [];

  const openPath = useCallback(
    async (path: string) => {
      setRescue(null);
      if (await p.openProject(path)) {
        ws.showEditor();
        return;
      }
      try {
        const copies = await nativeApi.project.listAutosaves(path);
        const newest = Array.isArray(copies) ? copies[0] : undefined;
        if (newest) setRescue({ projectPath: path, autosave: newest });
      } catch {
        // Aucune proposition de secours possible : l'erreur d'ouverture reste affichée.
      }
    },
    [p, ws]
  );

  const requestCreate = () =>
    guard.run(() => {
      p.dismissError();
      ws.openDialog("create");
    });

  const submitCreate = async (input: NewProjectInput) => {
    if (await p.createProject(input)) {
      ws.closeDialog();
      ws.showEditor();
    }
  };

  const requestOpen = async () => {
    try {
      const path = await nativeApi.dialogs.pickProjectFile();
      if (path) guard.run(() => openPath(path));
    } catch (cause) {
      p.reportError(cause, "open");
    }
  };

  const openRecent = (path: string) => guard.run(() => openPath(path));

  const save = () => {
    if (project) void p.saveProject();
  };

  const saveAs = async () => {
    if (!project) return;
    try {
      const path = await nativeApi.dialogs.pickSaveProjectPath(project.path);
      if (path) await p.saveProjectAs(path);
    } catch (cause) {
      p.reportError(cause, "save");
    }
  };

  const closeProject = () =>
    guard.run(async () => {
      await p.closeProject(false);
      ws.showHome();
    });

  const submitSettings = async (input: SettingsInput) => {
    if (await p.updateSettings(input)) ws.closeDialog();
  };

  const browseFolder = async (title: string, context: "create" | "relink") => {
    try {
      return await nativeApi.dialogs.pickFolder(title);
    } catch (cause) {
      p.reportError(cause, context);
      return null;
    }
  };

  const locateMedia = async (media: MediaView) => {
    try {
      const file = await nativeApi.dialogs.pickMediaFile(`Localiser « ${media.fileName} »`);
      if (file) await p.relinkMedia(media.id, file);
    } catch (cause) {
      p.reportError(cause, "relink");
    }
  };

  const searchFolder = async () => {
    const folder = await browseFolder("Dossier contenant les médias", "relink");
    return folder ? p.relinkMediaInFolder(folder) : null;
  };

  useCloseGuard(project?.dirty ?? false, () => guard.run(destroyWindow));
  useProjectShortcuts({
    onNew: requestCreate,
    onOpen: () => void requestOpen(),
    onSave: save,
    onSaveAs: () => void saveAs()
  });

  const error = p.error;
  const saveErrorInGuard = guard.isPending && error?.context === "save";
  const bannerError = error && !DIALOG_CONTEXTS.includes(error.context) && !saveErrorInGuard ? error : null;
  const showRecovery = p.recovery !== null && project !== null && !guard.isPending;
  const showMissingMedia =
    project !== null && unresolved.length > 0 && mediaDismissedFor !== project.id && !showRecovery && !guard.isPending;
  const projectNotice =
    project && infoDismissedFor !== project.id
      ? project.migratedFrom !== null
        ? `Ce projet a été converti depuis le format ${project.migratedFrom}. L'ancienne version sera conservée dans un fichier .bak à côté du projet lors du prochain enregistrement.`
        : project.newerFormat
          ? "Ce projet a été créé avec une version plus récente de Kito Studio. Ses données inconnues seront conservées à l'enregistrement."
          : null
      : null;

  return (
    <main className="app-shell">
      <Sidebar
        view={showEditor ? "editor" : "home"}
        hasProject={project !== null}
        onNavigate={(next) => (next === "home" ? ws.showHome() : ws.showEditor())}
        onOpenPreferences={() => ws.openDialog("preferences")}
      />

      <section className="main-area">
        <Topbar
          title={showEditor && project ? project.name : "Accueil"}
          dirty={project?.dirty ?? false}
          status={
            project ? (
              <SaveStatus
                project={project}
                busy={p.busy}
                lastAutosaveAt={p.lastAutosaveAt}
                autosaveError={p.autosaveError}
              />
            ) : null
          }
        />

        {showEditor && project ? (
          <ProjectToolbar
            project={project}
            busy={p.busy}
            unresolvedMedia={unresolved.length}
            onSave={save}
            onSaveAs={() => void saveAs()}
            onSettings={() => ws.openDialog("settings")}
            onShowMissingMedia={() => setMediaDismissedFor(null)}
            onClose={closeProject}
          />
        ) : null}

        {bannerError ? (
          <Notice
            tone="error"
            title={bannerError.title}
            message={bannerError.message}
            suggestion={bannerError.suggestion}
            onDismiss={() => {
              setRescue(null);
              p.dismissError();
            }}
          >
            {bannerError.context === "open" && rescue ? (
              <button
                className="secondary-button"
                onClick={() =>
                  void p.restoreAutosave(rescue.projectPath, rescue.autosave.path).then((ok) => {
                    if (ok) {
                      setRescue(null);
                      ws.showEditor();
                    }
                  })
                }
              >
                Restaurer la copie automatique du {formatDateTime(rescue.autosave.modifiedAt)}
              </button>
            ) : null}
            {bannerError.context === "save" ? (
              <>
                <button className="secondary-button" onClick={save}>
                  Réessayer
                </button>
                <button className="secondary-button" onClick={() => void saveAs()}>
                  Enregistrer sous…
                </button>
              </>
            ) : null}
          </Notice>
        ) : null}

        {projectNotice && project ? (
          <Notice
            tone="info"
            title="Information sur le projet"
            message={projectNotice}
            onDismiss={() => setInfoDismissedFor(project.id)}
          />
        ) : null}

        {showEditor && project ? (
          <EditorScreen project={project} />
        ) : (
          <HomeScreen
            recent={{ items: recents.items, loaded: recents.loaded }}
            onCreateProject={requestCreate}
            onOpenProject={() => void requestOpen()}
            onOpenRecent={openRecent}
            onRemoveRecent={(path) => void recents.remove(path)}
            onClearRecent={() => void recents.clear()}
          />
        )}
      </section>

      {ws.dialog === "create" ? (
        <CreateProjectDialog
          busy={p.busy === "creating"}
          error={error?.context === "create" ? error : null}
          onSubmit={(input) => void submitCreate(input)}
          onClose={() => {
            p.dismissError();
            ws.closeDialog();
          }}
          onBrowse={() => browseFolder("Choisir l'emplacement du projet", "create")}
        />
      ) : null}

      {ws.dialog === "settings" && project ? (
        <ProjectSettingsDialog
          project={project}
          busy={p.busy === "updating"}
          error={error?.context === "settings" ? error : null}
          onSubmit={(input) => void submitSettings(input)}
          onClose={() => {
            p.dismissError();
            ws.closeDialog();
          }}
        />
      ) : null}

      {ws.dialog === "preferences" ? <PreferencesDialog onClose={ws.closeDialog} /> : null}

      {showRecovery && project && p.recovery ? (
        <RecoveryDialog
          projectName={project.name}
          autosavedAt={p.recovery.autosavedAt}
          busy={p.busy !== "idle"}
          onRestore={() => void p.acceptRecovery()}
          onIgnore={p.declineRecovery}
        />
      ) : null}

      {showMissingMedia && project ? (
        <MissingMediaDialog
          media={unresolved}
          busy={p.busy === "relinking"}
          error={error?.context === "relink" ? error : null}
          onLocate={(media) => void locateMedia(media)}
          onSearchFolder={searchFolder}
          onClose={() => setMediaDismissedFor(project.id)}
        />
      ) : null}

      {guard.isPending && project ? (
        <UnsavedChangesDialog
          projectName={project.name}
          working={guard.working}
          error={saveErrorInGuard ? error : null}
          onChoose={(choice) => void guard.resolve(choice)}
        />
      ) : null}
    </main>
  );
}
