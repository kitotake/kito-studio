import { useCallback, useState } from "react";

export type GuardChoice = "save" | "discard" | "cancel";
type Action = () => void | Promise<void>;

type Options = {
  isDirty: boolean;
  /** Enregistre le projet ; retourne `false` en cas d'échec (l'action reste alors en attente). */
  save: () => Promise<boolean>;
  /** Abandonne les modifications (ferme le projet et nettoie les copies auto). */
  discard: () => Promise<boolean>;
};

/**
 * Exécute `action` tout de suite si le projet est propre ; sinon demande à l'utilisateur
 * d'enregistrer, d'abandonner ou d'annuler avant de l'exécuter.
 */
export function useUnsavedGuard({ isDirty, save, discard }: Options) {
  const [pending, setPending] = useState<Action | null>(null);
  const [working, setWorking] = useState(false);

  const run = useCallback(
    (action: Action) => {
      if (!isDirty) {
        void action();
        return;
      }
      setPending(() => action);
    },
    [isDirty]
  );

  const resolve = useCallback(
    async (choice: GuardChoice) => {
      if (pending === null) return;
      if (choice === "cancel") {
        setPending(null);
        return;
      }
      setWorking(true);
      try {
        const done = choice === "save" ? await save() : await discard();
        if (done) {
          setPending(null);
          await pending();
        }
      } finally {
        setWorking(false);
      }
    },
    [pending, save, discard]
  );

  return { run, resolve, isPending: pending !== null, working };
}
