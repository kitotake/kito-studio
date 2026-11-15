import { DevelopmentPanel } from "./DevelopmentPanel";
import { QuickCards } from "./QuickCards";

export function HomeScreen({ onCreateProject }: { onCreateProject: () => void }) {
  return (
    <div className="home-content">
      <div className="welcome-row">
        <div>
          <div className="eyebrow">ESPACE CRÉATIF</div>
          <h1>Donnez vie à vos idées.</h1>
          <p>Votre espace de montage vidéo, pensé pour créer sans limites.</p>
        </div>
        <button className="primary-button" onClick={onCreateProject}>
          <span>＋</span> Nouveau projet
        </button>
      </div>
      <QuickCards onCreateProject={onCreateProject} />
      <DevelopmentPanel />
    </div>
  );
}
