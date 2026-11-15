import type { ReactNode } from "react";

type PanelProps = {
  title: string;
  /** Texte secondaire affiché à droite du titre. */
  aside?: string;
  className?: string;
  children: ReactNode;
};

export function Panel({ title, aside, className, children }: PanelProps) {
  return (
    <section className={className ? `panel ${className}` : "panel"}>
      <div className="panel-title">
        {title}
        {aside !== undefined && <span>{aside}</span>}
      </div>
      {children}
    </section>
  );
}
