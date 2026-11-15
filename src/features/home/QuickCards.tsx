import { useSystemStatus } from "@/features/system/useSystemStatus";
import type { SystemStatus } from "@/features/system/SystemStatusContext";

function describeMediaTools(status: SystemStatus): string {
  switch (status.kind) {
    case "loading":
      return "Vérification en cours…";
    case "browser":
      return "Vérification disponible dans l'application native";
    case "error":
      return `Vérification impossible : ${status.error.message}`;
    case "ready": {
      const { ffmpegFound, ffprobeFound } = status.tools;
      return `FFmpeg : ${ffmpegFound ? "détecté" : "non détecté"} · FFprobe : ${ffprobeFound ? "détecté" : "non détecté"}`;
    }
  }
}

export function QuickCards({ onCreateProject }: { onCreateProject: () => void }) {
  const systemStatus = useSystemStatus();

  return (
    <section className="quick-grid">
      <article className="quick-card accent-card">
        <div className="card-icon">✦</div>
        <div className="card-kicker">COMMENCER</div>
        <h2>Créer un montage</h2>
        <p>Configurez une séquence et préparez votre espace de travail.</p>
        <button className="text-button" onClick={onCreateProject}>
          Créer un projet <span>→</span>
        </button>
      </article>

      <article className="quick-card">
        <div className="card-icon muted-icon">▧</div>
        <div className="card-kicker">VOTRE ESPACE</div>
        <h2>Projets récents</h2>
        <p>
          La gestion persistante des projets arrive dans la première étape du moteur projet.
        </p>
        <span className="subtle-note">Aucun projet enregistré</span>
      </article>

      <article className="quick-card">
        <div className="card-icon muted-icon">◈</div>
        <div className="card-kicker">RESSOURCES</div>
        <h2>Outils média</h2>
        <p>Le moteur FFmpeg sera connecté pour analyser et traiter vos fichiers.</p>
        <span className="subtle-note">{describeMediaTools(systemStatus)}</span>
      </article>
    </section>
  );
}
