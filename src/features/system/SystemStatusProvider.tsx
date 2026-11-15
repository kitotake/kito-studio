import { useEffect, useState, type ReactNode } from "react";
import { toKitoError } from "@/lib/errors";
import { isNativeRuntime, nativeApi } from "@/lib/tauri";
import { SystemStatusContext, type SystemStatus } from "./SystemStatusContext";

/** Interroge une seule fois le backend et partage le résultat avec toute l'interface. */
export function SystemStatusProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<SystemStatus>(() =>
    isNativeRuntime() ? { kind: "loading" } : { kind: "browser" }
  );

  useEffect(() => {
    if (!isNativeRuntime()) return;
    let cancelled = false;

    Promise.all([nativeApi.getRuntimeInfo(), nativeApi.checkMediaTools()])
      .then(([runtime, tools]) => {
        if (!cancelled) setStatus({ kind: "ready", runtime, tools });
      })
      .catch((error: unknown) => {
        if (!cancelled) setStatus({ kind: "error", error: toKitoError(error) });
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return <SystemStatusContext value={status}>{children}</SystemStatusContext>;
}
