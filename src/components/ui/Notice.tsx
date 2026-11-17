import type { ReactNode } from "react";

type NoticeProps = {
  tone: "error" | "info";
  title: string;
  message: string;
  suggestion?: string | null;
  onDismiss?: () => void;
  /** Boutons d'action supplémentaires. */
  children?: ReactNode;
};

export function Notice({ tone, title, message, suggestion, onDismiss, children }: NoticeProps) {
  return (
    <div className={`notice notice-${tone}`} role={tone === "error" ? "alert" : "status"}>
      <div className="notice-body">
        <strong>{title}</strong>
        <p>{message}</p>
        {suggestion ? <small>{suggestion}</small> : null}
      </div>
      <div className="notice-actions">
        {children}
        {onDismiss ? (
          <button className="secondary-button" onClick={onDismiss}>
            Fermer
          </button>
        ) : null}
      </div>
    </div>
  );
}
