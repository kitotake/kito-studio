import { SystemCard } from "@/features/system/SystemCard";
import { APP_VERSION_LABEL } from "@/lib/appInfo";
import type { WorkspaceView } from "@/types/workspace";

type SidebarProps = {
  view: WorkspaceView;
  /** Le bouton « Éditeur » n'a de sens que si un projet est ouvert. */
  hasProject: boolean;
  onNavigate: (view: WorkspaceView) => void;
  onOpenPreferences: () => void;
};

export function Sidebar({ view, hasProject, onNavigate, onOpenPreferences }: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-mark">K</div>
        <div>
          <strong>KITO</strong>
          <span>STUDIO</span>
        </div>
      </div>

      <div className="sidebar-label">ESPACE DE TRAVAIL</div>
      <button
        className={`nav-item ${view === "home" ? "active" : ""}`}
        onClick={() => onNavigate("home")}
      >
        <span className="nav-icon">⌂</span> Accueil
      </button>
      <button
        className={`nav-item ${view === "editor" ? "active" : ""}`}
        onClick={() => onNavigate("editor")}
        disabled={!hasProject}
        title={hasProject ? undefined : "Créez ou ouvrez un projet pour accéder à l'éditeur"}
      >
        <span className="nav-icon">▤</span> Éditeur
      </button>
      <button className="nav-item" onClick={onOpenPreferences}>
        <span className="nav-icon">⚙</span> Préférences
      </button>

      <div className="sidebar-bottom">
        <div className="sidebar-label">SYSTÈME</div>
        <SystemCard />
        <div className="version">{APP_VERSION_LABEL}</div>
      </div>
    </aside>
  );
}
