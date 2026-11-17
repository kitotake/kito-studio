import { useId, useState } from "react";
import { Modal } from "@/components/ui/Modal";
import { Notice } from "@/components/ui/Notice";
import type { MediaView } from "@/types/project";
import type { DescribedError } from "./projectErrors";

type Props = {
  /** Médias introuvables ou modifiés. */
  media: MediaView[];
  busy: boolean;
  error: DescribedError | null;
  onLocate: (media: MediaView) => void;
  /** Lance la recherche dans un dossier ; retourne le nombre de médias reliés (`null` si annulé). */
  onSearchFolder: () => Promise<number | null>;
  onClose: () => void;
};

export function MissingMediaDialog({ media, busy, error, onLocate, onSearchFolder, onClose }: Props) {
  const titleId = useId();
  const [searchResult, setSearchResult] = useState<string | null>(null);

  async function search() {
    const count = await onSearchFolder();
    if (count === null) return;
    setSearchResult(
      count === 0
        ? "Aucun média correspondant (même nom et même taille) trouvé dans ce dossier."
        : `${count} média${count > 1 ? "s" : ""} relié${count > 1 ? "s" : ""} automatiquement.`
    );
  }

  return (
    <Modal titleId={titleId} onClose={onClose} wide>
      <div className="modal-top">
        <div>
          <div className="eyebrow">MÉDIAS EXTERNES</div>
          <h2 id={titleId}>
            {media.length} média{media.length > 1 ? "s" : ""} à retrouver
          </h2>
        </div>
        <button className="close-button" onClick={onClose} aria-label="Fermer">
          ×
        </button>
      </div>

      <p className="modal-lead">
        Les médias restent à leur emplacement d'origine et ne sont jamais copiés dans le projet.
        Ceux-ci n'ont pas été retrouvés (projet déplacé, disque déconnecté…) ou ont changé.
      </p>

      <ul className="media-issue-list">
        {media.map((item) => (
          <li key={item.id}>
            <div>
              <strong>{item.fileName}</strong>
              <small title={item.path}>{item.path}</small>
            </div>
            <span className={`status-chip chip-${item.status}`}>
              {item.status === "missing" ? "Introuvable" : "Modifié"}
            </span>
            <button className="secondary-button" disabled={busy} onClick={() => onLocate(item)}>
              Localiser…
            </button>
          </li>
        ))}
      </ul>

      {searchResult ? <Notice tone="info" title="Recherche terminée" message={searchResult} /> : null}
      {error ? <Notice tone="error" title={error.title} message={error.message} suggestion={error.suggestion} /> : null}

      <div className="modal-actions">
        <button className="secondary-button" disabled={busy} onClick={() => void search()}>
          Rechercher dans un dossier…
        </button>
        <button className="primary-button" onClick={onClose}>
          Plus tard
        </button>
      </div>
    </Modal>
  );
}
