import type { ProjectSnapshot } from "@/types/project";
import type { BusyState } from "./ProjectContext";
import { formatTime } from "./format";

type Props = {
  project: ProjectSnapshot;
  busy: BusyState;
  lastAutosaveAt: string | null;
  autosaveError: string | null;
};

type View = { tone: "saved" | "dirty" | "saving" | "error"; text: string; title?: string };

export function describeSaveStatus({ project, busy, lastAutosaveAt, autosaveError }: Props): View {
  if (busy === "saving") return { tone: "saving", text: "Enregistrement…" };
  if (autosaveError) {
    return { tone: "error", text: "Sauvegarde automatique échouée", title: autosaveError };
  }
  if (project.dirty) {
    return {
      tone: "dirty",
      text: lastAutosaveAt
        ? `Modifications non enregistrées · copie auto ${formatTime(lastAutosaveAt)}`
        : "Modifications non enregistrées"
    };
  }
  return {
    tone: "saved",
    text: project.lastSavedAt ? `Enregistré à ${formatTime(project.lastSavedAt)}` : "Enregistré"
  };
}

export function SaveStatus(props: Props) {
  const view = describeSaveStatus(props);
  return (
    <span className={`save-status save-${view.tone}`} role="status" title={view.title}>
      <i />
      {view.text}
    </span>
  );
}
