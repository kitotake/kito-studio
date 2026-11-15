import { useCallback, useState } from "react";
import type { ProjectProfile } from "@/types/project";
import type { WorkspaceView } from "@/types/workspace";

/**
 * État de navigation et projet courant.
 * La vue est distincte du projet : revenir à l'accueil ne ferme pas le projet.
 */
export function useWorkspace() {
  const [project, setProject] = useState<ProjectProfile | null>(null);
  const [view, setView] = useState<WorkspaceView>("home");
  const [isCreateOpen, setCreateOpen] = useState(false);

  const openProject = useCallback((next: ProjectProfile) => {
    setProject(next);
    setView("editor");
    setCreateOpen(false);
  }, []);

  const navigate = useCallback(
    (next: WorkspaceView) => {
      if (next === "editor" && project === null) return;
      setView(next);
    },
    [project]
  );

  const openCreateDialog = useCallback(() => setCreateOpen(true), []);
  const closeCreateDialog = useCallback(() => setCreateOpen(false), []);

  return { project, view, isCreateOpen, openProject, navigate, openCreateDialog, closeCreateDialog };
}
