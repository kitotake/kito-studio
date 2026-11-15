import type { ReactNode } from "react";

type ModalProps = {
  /** Identifiant du titre, référencé par aria-labelledby. */
  titleId: string;
  onClose: () => void;
  children: ReactNode;
};

export function Modal({ titleId, onClose, children }: ModalProps) {
  return (
    <div
      className="modal-backdrop"
      role="presentation"
      onMouseDown={(event) => event.target === event.currentTarget && onClose()}
    >
      <section className="modal" role="dialog" aria-modal="true" aria-labelledby={titleId}>
        {children}
      </section>
    </div>
  );
}
