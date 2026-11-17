import type { ReactNode } from "react";

type ModalProps = {
  /** Identifiant du titre, référencé par aria-labelledby. */
  titleId: string;
  onClose: () => void;
  wide?: boolean;
  children: ReactNode;
};

export function Modal({ titleId, onClose, wide = false, children }: ModalProps) {
  return (
    <div
      className="modal-backdrop"
      role="presentation"
      onMouseDown={(event) => event.target === event.currentTarget && onClose()}
    >
      <section
        className={wide ? "modal modal-wide" : "modal"}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
      >
        {children}
      </section>
    </div>
  );
}
