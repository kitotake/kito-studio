import { useId, useState } from "react";
import { Modal } from "@/components/ui/Modal";
import type { NewProjectInput } from "./createProjectProfile";
import { DEFAULT_FRAME_RATE, FRAME_RATES, VIDEO_PROFILES } from "./projectProfiles";

type CreateProjectDialogProps = {
  onSubmit: (input: NewProjectInput) => void;
  onClose: () => void;
};

export function CreateProjectDialog({ onSubmit, onClose }: CreateProjectDialogProps) {
  const titleId = useId();
  const [name, setName] = useState("Mon premier montage");
  const [profileIndex, setProfileIndex] = useState(0);
  const [frameRate, setFrameRate] = useState(DEFAULT_FRAME_RATE);

  function handleSubmit() {
    const profile = VIDEO_PROFILES[profileIndex];
    if (!profile) return;
    onSubmit({ name, width: profile.width, height: profile.height, frameRate });
  }

  return (
    <Modal titleId={titleId} onClose={onClose}>
      <div className="modal-top">
        <div>
          <div className="eyebrow">NOUVEAU PROJET</div>
          <h2 id={titleId}>Configurer votre séquence</h2>
        </div>
        <button className="close-button" onClick={onClose} aria-label="Fermer">
          ×
        </button>
      </div>

      <label>
        Nom du projet
        <input value={name} onChange={(event) => setName(event.target.value)} autoFocus />
      </label>

      <label>
        Profil vidéo
        <select
          value={profileIndex}
          onChange={(event) => setProfileIndex(Number(event.target.value))}
        >
          {VIDEO_PROFILES.map((profile, index) => (
            <option value={index} key={profile.label}>
              {profile.label} — {profile.width} × {profile.height}
            </option>
          ))}
        </select>
      </label>

      <label>
        Fréquence d'images
        <select value={frameRate} onChange={(event) => setFrameRate(Number(event.target.value))}>
          {FRAME_RATES.map((rate) => (
            <option value={rate} key={rate}>
              {rate} FPS
            </option>
          ))}
        </select>
      </label>

      <div className="modal-note">
        Cette première version crée une session de travail en mémoire. La sauvegarde native au
        format .kito sera ajoutée dans le module de gestion des projets.
      </div>

      <div className="modal-actions">
        <button className="secondary-button" onClick={onClose}>
          Annuler
        </button>
        <button className="primary-button" onClick={handleSubmit}>
          Créer le projet
        </button>
      </div>
    </Modal>
  );
}
