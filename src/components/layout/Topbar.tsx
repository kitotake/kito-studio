import type { ReactNode } from "react";

type TopbarProps = {
  title: string;
  /** Pastille « non enregistré » à côté du titre. */
  dirty?: boolean;
  /** Indicateur de sauvegarde, affiché à droite. */
  status?: ReactNode;
};

export function Topbar({ title, dirty = false, status }: TopbarProps) {
  return (
    <header className="topbar">
      <div className="breadcrumb">
        Kito Studio <span>/</span> {title}
        {dirty ? <i className="dirty-dot" title="Modifications non enregistrées" aria-hidden="true" /> : null}
      </div>
      <div className="topbar-right">
        {status}
        <span className="local-pill">
          <i /> Projet local
        </span>
        <div className="avatar">K</div>
      </div>
    </header>
  );
}
