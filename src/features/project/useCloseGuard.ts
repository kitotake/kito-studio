import { useEffect, useRef } from "react";
import { onCloseRequested } from "@/lib/tauri";

/**
 * Empêche la fermeture de la fenêtre tant que des modifications ne sont pas enregistrées
 * et délègue la décision à `onDirtyClose`.
 */
export function useCloseGuard(isDirty: boolean, onDirtyClose: () => void): void {
  const dirtyRef = useRef(isDirty);
  const callbackRef = useRef(onDirtyClose);

  useEffect(() => {
    dirtyRef.current = isDirty;
    callbackRef.current = onDirtyClose;
  });

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    onCloseRequested((prevent) => {
      if (dirtyRef.current) {
        prevent();
        callbackRef.current();
      }
    })
      .then((off) => (cancelled ? off() : (unlisten = off)))
      .catch(() => undefined);
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);
}
