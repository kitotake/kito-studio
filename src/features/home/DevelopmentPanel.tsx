type Milestone = {
  title: string;
  description: string;
  done: boolean;
};

const MILESTONES: readonly Milestone[] = [
  { title: "Interface de base", description: "Structure et thème sombre", done: true },
  { title: "Backend Rust", description: "Commandes natives initiales", done: true },
  { title: "Gestion des projets", description: "Format .kito et sauvegarde", done: true },
  { title: "Importation média", description: "Analyse et prévisualisation", done: false }
];

export function DevelopmentPanel() {
  return (
    <section className="bottom-panel">
      <div className="section-heading">
        <div>
          <div className="eyebrow">DÉVELOPPEMENT</div>
          <h2>Votre studio prend forme</h2>
        </div>
        <span className="phase-tag">PHASE 1 · PROJETS</span>
      </div>
      <div className="milestones">
        {MILESTONES.map((milestone, index) => (
          <div className={`milestone ${milestone.done ? "done" : ""}`} key={milestone.title}>
            <span>{milestone.done ? "✓" : String(index + 1).padStart(2, "0")}</span>
            <div>
              <strong>{milestone.title}</strong>
              <small>{milestone.description}</small>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
