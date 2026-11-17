import { useEffect, useId, useState } from "react";
import { Modal } from "@/components/ui/Modal";
import { Notice } from "@/components/ui/Notice";
import { toKitoError } from "@/lib/errors";
import { nativeApi } from "@/lib/tauri";
import type { Preferences } from "@/types/project";
import { describeProjectError, type DescribedError } from "./projectErrors";

export const AUTOSAVE_INTERVALS: readonly number[] = [1, 2, 5, 10, 15, 30];

export function PreferencesDialog({ onClose }: { onClose: () => void }) {
  const titleId = useId();
  const [prefs, setPrefs] = useState<Preferences | null>(null);
  const [error, setError] = useState<DescribedError | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let cancelled = false;
    nativeApi.preferences
      .get()
      .then((loaded) => !cancelled && setPrefs(loaded))
      .catch((cause: unknown) => !cancelled && setError(describeProjectError(toKitoError(cause), "other")));
    return () => {
      cancelled = true;
    };
  }, []);

  async function save() {
    if (!prefs) return;
    setSaving(true);
    setError(null);
    try {
      await nativeApi.preferences.set(prefs);
      onClose();
    } catch (cause) {
      setError(describeProjectError(toKitoError(cause), "other"));
    } finally {
      setSaving(false);
    }
  }

  const autosave = prefs?.autosave;
  const update = (patch: Partial<NonNullable<typeof autosave>>) =>
    setPrefs((current) => (current ? { ...current, autosave: { ...current.autosave, ...patch } } : current));

  return (
    <Modal titleId={titleId} onClose={onClose}>
      <div className="modal-top">
        <div>
          <div className="eyebrow">PRÉFÉRENCES</div>
          <h2 id={titleId}>Sauvegarde automatique</h2>
        </div>
        <button className="close-button" onClick={onClose} aria-label="Fermer">
          ×
        </button>
      </div>

      {autosave ? (
        <>
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={autosave.enabled}
              onChange={(event) => update({ enabled: event.target.checked })}
            />
            Activer la sauvegarde automatique
          </label>

          <label>
            Intervalle
            <select
              value={autosave.intervalMinutes}
              disabled={!autosave.enabled}
              onChange={(event) => update({ intervalMinutes: Number(event.target.value) })}
            >
              {AUTOSAVE_INTERVALS.map((minutes) => (
                <option value={minutes} key={minutes}>
                  Toutes les {minutes} minute{minutes > 1 ? "s" : ""}
                </option>
              ))}
            </select>
          </label>

          <label>
            Copies conservées par projet
            <input
              type="number"
              min={1}
              max={100}
              value={autosave.keepCopies}
              disabled={!autosave.enabled}
              onChange={(event) => update({ keepCopies: Number(event.target.value) })}
            />
          </label>

          <div className="modal-note">
            Les copies sont écrites dans le dossier « Autosave » du projet et n'écrasent jamais votre
            dernier enregistrement. Elles permettent de récupérer votre travail après une fermeture
            inattendue.
          </div>
        </>
      ) : error ? null : (
        <p className="modal-lead">Chargement…</p>
      )}

      {error ? <Notice tone="error" title={error.title} message={error.message} suggestion={error.suggestion} /> : null}

      <div className="modal-actions">
        <button className="secondary-button" onClick={onClose}>
          Annuler
        </button>
        <button className="primary-button" onClick={() => void save()} disabled={!prefs || saving}>
          Enregistrer
        </button>
      </div>
    </Modal>
  );
}
