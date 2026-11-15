import { Panel } from "@/components/ui/Panel";

export function PreviewPanel() {
  return (
    <Panel className="preview-panel" title="MONITEUR PROGRAMME" aside="PRÉVISUALISATION">
      <div className="preview-screen">
        <div className="preview-logo">K</div>
        <span>AUCUNE SÉQUENCE À LIRE</span>
      </div>
      <div className="transport">
        {"◀\u3000▶\u3000▶\u3000 "}
        <span>00:00:00:00</span>
      </div>
    </Panel>
  );
}
