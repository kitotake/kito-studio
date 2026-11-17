import { useId } from "react";
import { Modal } from "@/components/ui/Modal";
import { Notice } from "@/components/ui/Notice";
import type { GuardChoice } from "./useUnsavedGuard";
import type { DescribedError } from "./projectErrors";

type Props = {
  projectName: string;
  working: boolean;
  error: DescribedError | null;
  onChoose: (choice: GuardChoice) => void;
};

export function UnsavedChangesDialog({ projectName, working, error, onChoose }: Props) {
  const titleId = useId();
  return (
    <Modal titleId={titleId} onClose={() => !working && onChoose("cancel")}>
      <div className="modal-top">
        <div>
          <div className="eyebrow">MODIFICATIONS NON ENREGISTRÉES</div>
          <h2 id={titleId}>Enregistrer « {projectName} » ?</h2>
        </div>
      </div>
      <p className="modal-lead">Vos dernières modifications seront perdues si vous ne les enregistrez pas.</p>

      {error ? <Notice tone="error" title={error.title} message={error.message} suggestion={error.suggestion} /> : null}

      <div className="modal-actions">
        <button className="secondary-button" onClick={() => onChoose("cancel")} disabled={working}>
          Annuler
        </button>
        <button className="danger-button" onClick={() => onChoose("discard")} disabled={working}>
          Ne pas enregistrer
        </button>
        <button className="primary-button" onClick={() => onChoose("save")} disabled={working}>
          {working ? "Enregistrement…" : "Enregistrer"}
        </button>
      </div>
    </Modal>
  );
}
