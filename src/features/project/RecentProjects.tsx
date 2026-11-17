import { Panel } from "@/components/ui/Panel";
import type { RecentProject } from "@/types/project";
import { formatDateTime } from "./format";

type Props = {
  items: RecentProject[];
  loaded: boolean;
  onOpen: (path: string) => void;
  onRemove: (path: string) => void;
  onClear: () => void;
};

export function RecentProjects({ items, loaded, onOpen, onRemove, onClear }: Props) {
  return (
    <Panel
      className="recent-panel"
      title="PROJETS RÉCENTS"
      aside={items.length > 0 ? `${items.length} projet${items.length > 1 ? "s" : ""}` : undefined}
    >
      {items.length === 0 ? (
        <div className="recent-empty">
          <strong>{loaded ? "Aucun projet récent" : "Chargement…"}</strong>
          <small>Les projets que vous créez ou ouvrez apparaîtront ici.</small>
        </div>
      ) : (
        <>
          <ul className="recent-list">
            {items.map((item) => (
              <li className={`recent-item ${item.exists ? "" : "is-missing"}`} key={item.path}>
                <button
                  className="recent-open"
                  disabled={!item.exists}
                  onClick={() => onOpen(item.path)}
                  title={item.exists ? `Ouvrir ${item.path}` : "Ce projet est introuvable à cet emplacement"}
                >
                  <strong>{item.name}</strong>
                  <small>{item.path}</small>
                </button>
                <span className="recent-meta">
                  {item.exists ? formatDateTime(item.lastOpenedAt) : "Introuvable"}
                </span>
                <button
                  className="close-button"
                  aria-label={`Retirer ${item.name} des projets récents`}
                  title="Retirer de la liste (le fichier n'est pas supprimé)"
                  onClick={() => onRemove(item.path)}
                >
                  ×
                </button>
              </li>
            ))}
          </ul>
          <div className="recent-footer">
            <button className="text-button" onClick={onClear}>
              Vider la liste
            </button>
          </div>
        </>
      )}
    </Panel>
  );
}
