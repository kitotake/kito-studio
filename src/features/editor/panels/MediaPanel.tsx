import { Panel } from "@/components/ui/Panel";
import type { MediaView } from "@/types/project";

export function MediaPanel({ media }: { media: MediaView[] }) {
  return (
    <Panel
      className="media-panel"
      title="MÉDIAS"
      aside={`${media.length} élément${media.length > 1 ? "s" : ""}`}
    >
      {media.length === 0 ? (
        <div className="empty-state">
          <div>▧</div>
          <strong>Bibliothèque vide</strong>
          <small>L'importation de médias sera le prochain module.</small>
        </div>
      ) : (
        <ul className="media-list">
          {media.map((item) => (
            <li className="media-row" key={item.id}>
              <div>
                <strong>{item.fileName}</strong>
                <small title={item.resolvedPath ?? item.path}>{item.resolvedPath ?? item.path}</small>
              </div>
              {item.status !== "ok" ? (
                <span className={`status-chip chip-${item.status}`}>
                  {item.status === "missing" ? "Introuvable" : "Modifié"}
                </span>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </Panel>
  );
}
