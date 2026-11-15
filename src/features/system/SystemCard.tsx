import { useSystemStatus } from "./useSystemStatus";

function describe(status: ReturnType<typeof useSystemStatus>): { title: string; detail: string } {
  switch (status.kind) {
    case "loading":
      return { title: "Application native", detail: "Connexion en cours…" };
    case "browser":
      return { title: "Mode navigateur", detail: "Aucune liaison native détectée" };
    case "ready":
      return {
        title: "Application native",
        detail: `${status.runtime.os} · ${status.runtime.architecture}`
      };
    case "error":
      return { title: "Liaison native indisponible", detail: status.error.message };
  }
}

export function SystemCard() {
  const { title, detail } = describe(useSystemStatus());

  return (
    <div className="system-card">
      <span className="status-dot" />
      <div>
        <strong>{title}</strong>
        <small>{detail}</small>
      </div>
    </div>
  );
}
