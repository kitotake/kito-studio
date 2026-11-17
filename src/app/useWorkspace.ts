import { useCallback, useState } from "react";
import type { WorkspaceView } from "@/types/workspace";

export type WorkspaceDialog = "create" | "settings" | "preferences";

/**
 * État de navigation et de dialogues. Le projet ouvert vit dans `ProjectProvider` :
 * revenir à l'accueil ne ferme jamais le projet.
 */
export function useWorkspace() {
  const [view, setView] = useState<WorkspaceView>("home");
  const [dialog, setDialog] = useState<WorkspaceDialog | null>(null);

  const showHome = useCallback(() => setView("home"), []);
  const showEditor = useCallback(() => setView("editor"), []);
  const openDialog = useCallback((next: WorkspaceDialog) => setDialog(next), []);
  const closeDialog = useCallback(() => setDialog(null), []);

  return { view, dialog, showHome, showEditor, openDialog, closeDialog };
}
