import { useEffect, useRef } from "react";

type Shortcuts = {
  onNew: () => void;
  onOpen: () => void;
  onSave: () => void;
  onSaveAs: () => void;
};

/** Ctrl/Cmd + N, O, S et Ctrl/Cmd + Maj + S. */
export function useProjectShortcuts(shortcuts: Shortcuts): void {
  const ref = useRef(shortcuts);
  useEffect(() => {
    ref.current = shortcuts;
  });

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (!(event.ctrlKey || event.metaKey) || event.altKey || event.repeat) return;
      const key = event.key.toLowerCase();
      const handlers = ref.current;
      if (key === "n" && !event.shiftKey) handlers.onNew();
      else if (key === "o" && !event.shiftKey) handlers.onOpen();
      else if (key === "s") (event.shiftKey ? handlers.onSaveAs : handlers.onSave)();
      else return;
      event.preventDefault();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
