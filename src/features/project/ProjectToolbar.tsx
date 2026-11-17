import type { ProjectSnapshot } from "@/types/project";
import type { BusyState } from "./ProjectContext";

type Props = {
  project: ProjectSnapshot;
  busy: BusyState;
  unresolvedMedia: number;
  onSave: () => void;
  onSaveAs: () => void;
  onSettings: () => void;
  onShowMissingMedia: () => void;
  onClose: () => void;
};

export function ProjectToolbar({
  project,
  busy,
  unresolvedMedia,
  onSave,
  onSaveAs,
  onSettings,
  onShowMissingMedia,
  onClose
}: Props) {
  const working = busy !== "idle";
  return (
    <div className="project-bar">
      {unresolvedMedia > 0 ? (
        <button className="toolbar-button toolbar-warning" onClick={onShowMissingMedia}>
          {unresolvedMedia} média{unresolvedMedia > 1 ? "s" : ""} à retrouver
        </button>
      ) : null}
      <button className="toolbar-button" onClick={onSettings} disabled={working}>
        Paramètres
      </button>
      <button className="toolbar-button" onClick={onSaveAs} disabled={working} title="Ctrl+Maj+S">
        Enregistrer sous…
      </button>
      <button className="toolbar-button toolbar-primary" onClick={onSave} disabled={working || !project.dirty} title="Ctrl+S">
        Enregistrer
      </button>
      <button className="toolbar-button" onClick={onClose} disabled={working}>
        Fermer le projet
      </button>
    </div>
  );
}
