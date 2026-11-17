import { useId, useState } from "react";
import { Modal } from "@/components/ui/Modal";
import { Notice } from "@/components/ui/Notice";
import type { ProjectSnapshot, SettingsInput } from "@/types/project";
import type { DescribedError } from "./projectErrors";
import { ProjectSettingsFields } from "./ProjectSettingsFields";
import { settingsFormFromProject, toSettingsInput } from "./settingsForm";
import { validateProjectName } from "./validateProjectName";

type Props = {
  project: ProjectSnapshot;
  busy: boolean;
  error: DescribedError | null;
  onSubmit: (input: SettingsInput) => void;
  onClose: () => void;
};

export function ProjectSettingsDialog({ project, busy, error, onSubmit, onClose }: Props) {
  const titleId = useId();
  const [{ value: initial, extras }] = useState(() => settingsFormFromProject(project));
  const [value, setValue] = useState(initial);
  const canSubmit = !busy && validateProjectName(value.name) === null;

  return (
    <Modal titleId={titleId} onClose={onClose}>
      <div className="modal-top">
        <div>
          <div className="eyebrow">PARAMÈTRES DU PROJET</div>
          <h2 id={titleId}>{project.name}</h2>
        </div>
        <button className="close-button" onClick={onClose} aria-label="Fermer">
          ×
        </button>
      </div>

      <ProjectSettingsFields value={value} extras={extras} onChange={setValue} />

      <div className="modal-note">
        Les modifications s'appliquent au projet ouvert et doivent être enregistrées. Changer le
        nom ne renomme pas le fichier ni le dossier du projet.
      </div>

      {error ? <Notice tone="error" title={error.title} message={error.message} suggestion={error.suggestion} /> : null}

      <div className="modal-actions">
        <button className="secondary-button" onClick={onClose}>
          Annuler
        </button>
        <button className="primary-button" onClick={() => onSubmit(toSettingsInput(value, extras))} disabled={!canSubmit}>
          Appliquer
        </button>
      </div>
    </Modal>
  );
}
