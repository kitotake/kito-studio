import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { makeSnapshot } from "@/test/fixtures";
import { describeSaveStatus, SaveStatus } from "./SaveStatus";

const base = { busy: "idle" as const, lastAutosaveAt: null, autosaveError: null };

describe("indicateur de sauvegarde", () => {
  it("indique « Enregistré à HH:MM » quand le projet est propre", () => {
    expect(describeSaveStatus({ ...base, project: makeSnapshot() })).toMatchObject({ tone: "saved" });
    render(<SaveStatus {...base} project={makeSnapshot()} />);
    expect(screen.getByRole("status")).toHaveTextContent(/^Enregistré à \d{2}:\d{2}$/);
  });

  it("signale les modifications non enregistrées", () => {
    const view = describeSaveStatus({ ...base, project: makeSnapshot({ dirty: true }) });
    expect(view).toEqual({ tone: "dirty", text: "Modifications non enregistrées" });
  });

  it("mentionne la dernière copie automatique quand il y en a une", () => {
    const view = describeSaveStatus({
      ...base,
      project: makeSnapshot({ dirty: true }),
      lastAutosaveAt: "2026-11-15T10:05:00Z"
    });
    expect(view.text).toMatch(/Modifications non enregistrées · copie auto \d{2}:\d{2}/);
  });

  it("affiche l'enregistrement en cours en priorité", () => {
    expect(describeSaveStatus({ ...base, busy: "saving", project: makeSnapshot({ dirty: true }) }).tone).toBe("saving");
  });

  it("signale un échec de sauvegarde automatique avec son détail", () => {
    const view = describeSaveStatus({ ...base, project: makeSnapshot({ dirty: true }), autosaveError: "Disque plein" });
    expect(view).toMatchObject({ tone: "error", title: "Disque plein" });
  });
});
