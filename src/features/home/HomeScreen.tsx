import { RecentProjects } from "@/features/project/RecentProjects";
import type { RecentProject } from "@/types/project";
import { DevelopmentPanel } from "./DevelopmentPanel";
import { QuickCards } from "./QuickCards";

type HomeScreenProps = {
  recent: { items: RecentProject[]; loaded: boolean };
  onCreateProject: () => void;
  onOpenProject: () => void;
  onOpenRecent: (path: string) => void;
  onRemoveRecent: (path: string) => void;
  onClearRecent: () => void;
};

export function HomeScreen({
  recent,
  onCreateProject,
  onOpenProject,
  onOpenRecent,
  onRemoveRecent,
  onClearRecent
}: HomeScreenProps) {
  return (
    <div className="home-content">
      <div className="welcome-row">
        <div>
          <div className="eyebrow">ESPACE CRÉATIF</div>
          <h1>Donnez vie à vos idées.</h1>
          <p>Votre espace de montage vidéo, pensé pour créer sans limites.</p>
        </div>
        <div className="welcome-actions">
          <button className="secondary-button" onClick={onOpenProject} title="Ctrl+O">
            Ouvrir…
          </button>
          <button className="primary-button" onClick={onCreateProject} title="Ctrl+N">
            <span>＋</span> Nouveau projet
          </button>
        </div>
      </div>
      <QuickCards onCreateProject={onCreateProject} onOpenProject={onOpenProject} />
      <RecentProjects
        items={recent.items}
        loaded={recent.loaded}
        onOpen={onOpenRecent}
        onRemove={onRemoveRecent}
        onClear={onClearRecent}
      />
      <DevelopmentPanel />
    </div>
  );
}
