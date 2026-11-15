import { Panel } from "@/components/ui/Panel";

export function MediaPanel() {
  return (
    <Panel className="media-panel" title="MÉDIAS" aside="0 élément">
      <div className="empty-state">
        <div>▧</div>
        <strong>Bibliothèque vide</strong>
        <small>L'importation de médias sera le prochain module.</small>
      </div>
    </Panel>
  );
}
