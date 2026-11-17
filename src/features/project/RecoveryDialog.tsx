import { useId } from "react";
import { Modal } from "@/components/ui/Modal";
import { formatDateTime } from "./format";

type Props = {
  projectName: string;
  autosavedAt: string;
  busy: boolean;
  onRestore: () => void;
  onIgnore: () => void;
};

export function RecoveryDialog({ projectName, autosavedAt, busy, onRestore, onIgnore }: Props) {
  const titleId = useId();
  return (
    <Modal titleId={titleId} onClose={onIgnore}>
      <div className="modal-top">
        <div>
          <div className="eyebrow">RÉCUPÉRATION</div>
          <h2 id={titleId}>Récupérer un travail non enregistré ?</h2>
        </div>
      </div>
      <p className="modal-lead">
        Une sauvegarde automatique du <strong>{formatDateTime(autosavedAt)}</strong> est plus récente
        que le dernier enregistrement de « {projectName} ». Elle correspond probablement à des
        modifications perdues après une fermeture inattendue.
      </p>
      <div className="modal-note">
        La version enregistrée du projet n'est pas modifiée tant que vous n'enregistrez pas.
      </div>
      <div className="modal-actions">
        <button className="secondary-button" onClick={onIgnore} disabled={busy}>
          Ignorer
        </button>
        <button className="primary-button" onClick={onRestore} disabled={busy}>
          Restaurer la copie
        </button>
      </div>
    </Modal>
  );
}
