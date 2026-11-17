import { useId, useState } from "react";
import { Modal } from "@/components/ui/Modal";
import { Notice } from "@/components/ui/Notice";
import type { NewProjectInput } from "@/types/project";
import { loadLastLocation } from "./lastLocation";
import type { DescribedError } from "./projectErrors";
import { ProjectSettingsFields } from "./ProjectSettingsFields";
import { defaultSettingsForm, toSettingsInput } from "./settingsForm";
import { validateProjectName } from "./validateProjectName";

type CreateProjectDialogProps = {
  busy: boolean;
  error: DescribedError | null;
  onSubmit: (input: NewProjectInput) => void;
  onClose: () => void;
  /** Ouvre le sélecteur de dossier natif ; `null` si annulé. */
  onBrowse: () => Promise<string | null>;
};

function previewPath(location: string, name: string): string {
  const separator = location.includes("\\") ? "\\" : "/";
  const base = location.replace(/[\\/]+$/, "");
  return `${base}${separator}${name}${separator}${name}.kito`;
}

export function CreateProjectDialog({ busy, error, onSubmit, onClose, onBrowse }: CreateProjectDialogProps) {
  const titleId = useId();
  const [{ value: initialValue, extras }] = useState(() => defaultSettingsForm("Mon premier montage"));
  const [value, setValue] = useState(initialValue);
  const [location, setLocation] = useState(loadLastLocation);

  const name = value.name.trim();
  const canSubmit = !busy && validateProjectName(value.name) === null && location.trim() !== "";

  async function browse() {
    const picked = await onBrowse();
    if (picked) setLocation(picked);
  }

  function submit() {
    if (!canSubmit) return;
    onSubmit({ ...toSettingsInput(value, extras), parentDir: location.trim() });
  }

  return (
    <Modal titleId={titleId} onClose={busy ? () => undefined : onClose}>
      <div className="modal-top">
        <div>
          <div className="eyebrow">NOUVEAU PROJET</div>
          <h2 id={titleId}>Configurer votre projet</h2>
        </div>
        <button className="close-button" onClick={onClose} aria-label="Fermer" disabled={busy}>
          ×
        </button>
      </div>

      <ProjectSettingsFields value={value} extras={extras} onChange={setValue} autoFocusName />

      <label>
        Emplacement
        <span className="path-row">
          <input
            value={location}
            onChange={(event) => setLocation(event.target.value)}
            placeholder="Dossier où créer le projet"
            aria-label="Emplacement du projet"
          />
          <button type="button" className="secondary-button" onClick={() => void browse()}>
            Parcourir…
          </button>
        </span>
      </label>

      {location.trim() !== "" && name !== "" ? (
        <div className="path-preview" title={previewPath(location.trim(), name)}>
          Fichier créé : <code>{previewPath(location.trim(), name)}</code>
        </div>
      ) : null}

      {error ? <Notice tone="error" title={error.title} message={error.message} suggestion={error.suggestion} /> : null}

      <div className="modal-actions">
        <button className="secondary-button" onClick={onClose} disabled={busy}>
          Annuler
        </button>
        <button className="primary-button" onClick={submit} disabled={!canSubmit}>
          {busy ? "Création…" : "Créer le projet"}
        </button>
      </div>
    </Modal>
  );
}
