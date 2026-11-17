import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { CreateProjectDialog } from "./CreateProjectDialog";
import { saveLastLocation } from "./lastLocation";

function setup(props: Partial<React.ComponentProps<typeof CreateProjectDialog>> = {}) {
  const handlers = {
    onSubmit: vi.fn(),
    onClose: vi.fn(),
    onBrowse: vi.fn(async () => "/choisi/par/dialogue")
  };
  render(<CreateProjectDialog busy={false} error={null} {...handlers} {...props} />);
  return handlers;
}

const location = () => screen.getByLabelText("Emplacement du projet") as HTMLInputElement;
const submit = () => screen.getByRole("button", { name: "Créer le projet" });

describe("CreateProjectDialog", () => {
  it("n'autorise pas la création sans emplacement", () => {
    setup();
    expect(submit()).toBeDisabled();
  });

  it("envoie les valeurs par défaut (1080p, 30 FPS, 48 kHz) avec l'emplacement saisi", () => {
    const h = setup();
    fireEvent.change(location(), { target: { value: "  /mes/projets " } });
    fireEvent.click(submit());
    expect(h.onSubmit).toHaveBeenCalledWith({
      name: "Mon premier montage", parentDir: "/mes/projets",
      width: 1920, height: 1080, frameRateNum: 30, frameRateDen: 1, sampleRate: 48_000
    });
  });

  it("propose les 4 résolutions, 5 cadences et 3 fréquences audio demandées", () => {
    setup();
    const options = (label: string) =>
      Array.from((screen.getByLabelText(label) as HTMLSelectElement).options).map((o) => o.textContent);
    expect(options("Résolution").join("|")).toMatch(/720p.*1080p.*1440p.*4K/);
    expect(options("Fréquence d'images")).toEqual(["24 FPS", "25 FPS", "30 FPS", "50 FPS", "60 FPS"]);
    expect(options("Fréquence d'échantillonnage audio")).toEqual(["44,1 kHz", "48 kHz", "96 kHz"]);
  });

  it("transmet les choix de l'utilisateur (4K, 60 FPS, 96 kHz)", () => {
    const h = setup();
    fireEvent.change(screen.getByLabelText("Résolution"), { target: { value: "2160p" } });
    fireEvent.change(screen.getByLabelText("Fréquence d'images"), { target: { value: "60/1" } });
    fireEvent.change(screen.getByLabelText("Fréquence d'échantillonnage audio"), { target: { value: "96000" } });
    fireEvent.change(location(), { target: { value: "C:\\Projets" } });
    fireEvent.click(submit());
    expect(h.onSubmit).toHaveBeenCalledWith(
      expect.objectContaining({ width: 3840, height: 2160, frameRateNum: 60, sampleRate: 96_000, parentDir: "C:\\Projets" })
    );
  });

  it("refuse un nom invalide avec une explication", () => {
    const h = setup();
    fireEvent.change(location(), { target: { value: "/p" } });
    fireEvent.change(screen.getByLabelText("Nom du projet"), { target: { value: "a/b" } });
    expect(screen.getByRole("alert")).toHaveTextContent("« / » est interdit");
    expect(submit()).toBeDisabled();
    fireEvent.click(submit());
    expect(h.onSubmit).not.toHaveBeenCalled();
  });

  it("affiche le chemin du fichier qui sera créé", () => {
    setup();
    fireEvent.change(location(), { target: { value: "C:\\Projets\\" } });
    expect(screen.getByText("C:\\Projets\\Mon premier montage\\Mon premier montage.kito")).toBeInTheDocument();
  });

  it("remplit l'emplacement depuis le sélecteur natif", async () => {
    const h = setup();
    fireEvent.click(screen.getByRole("button", { name: "Parcourir…" }));
    await waitFor(() => expect(location().value).toBe("/choisi/par/dialogue"));
    expect(h.onBrowse).toHaveBeenCalled();
  });

  it("garde l'emplacement actuel si le sélecteur est annulé", async () => {
    const onBrowse = vi.fn(async () => null);
    setup({ onBrowse });
    fireEvent.change(location(), { target: { value: "/avant" } });
    fireEvent.click(screen.getByRole("button", { name: "Parcourir…" }));
    await waitFor(() => expect(onBrowse).toHaveBeenCalled());
    expect(location().value).toBe("/avant");
  });

  it("préremplit le dernier emplacement utilisé", () => {
    saveLastLocation("/dernier/emplacement");
    setup();
    expect(location().value).toBe("/dernier/emplacement");
  });

  it("affiche l'erreur de création et bloque pendant l'opération", () => {
    setup({
      busy: true,
      error: { code: "ALREADY_EXISTS", context: "create", title: "Impossible de créer le projet", message: "Un dossier « X » existe déjà.", suggestion: "Choisissez un autre nom." }
    });
    expect(screen.getByRole("alert")).toHaveTextContent("Un dossier « X » existe déjà.");
    expect(screen.getByRole("alert")).toHaveTextContent("Choisissez un autre nom.");
    expect(screen.getByRole("button", { name: "Création…" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Annuler" })).toBeDisabled();
  });
});
