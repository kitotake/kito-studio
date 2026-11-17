import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { makeMedia } from "@/test/fixtures";
import { MissingMediaDialog } from "./MissingMediaDialog";

const media = [
  makeMedia({ id: "a", fileName: "plan1.mp4", status: "missing", resolvedPath: null }),
  makeMedia({ id: "b", fileName: "voix.wav", status: "changed", kind: "audio" })
];

function setup(onSearchFolder = vi.fn(async () => 1 as number | null)) {
  const h = { onLocate: vi.fn(), onSearchFolder, onClose: vi.fn() };
  render(<MissingMediaDialog media={media} busy={false} error={null} {...h} />);
  return h;
}

describe("MissingMediaDialog", () => {
  it("liste les médias introuvables et modifiés avec leur état", () => {
    setup();
    expect(screen.getByRole("heading", { name: "2 médias à retrouver" })).toBeInTheDocument();
    expect(screen.getByText("Introuvable")).toBeInTheDocument();
    expect(screen.getByText("Modifié")).toBeInTheDocument();
  });

  it("permet de localiser un média à la main", () => {
    const h = setup();
    fireEvent.click(screen.getAllByRole("button", { name: "Localiser…" })[1]!);
    expect(h.onLocate).toHaveBeenCalledWith(media[1]);
  });

  it("annonce le résultat de la recherche dans un dossier", async () => {
    setup(vi.fn(async () => 2));
    fireEvent.click(screen.getByRole("button", { name: "Rechercher dans un dossier…" }));
    expect(await screen.findByText("2 médias reliés automatiquement.")).toBeInTheDocument();
  });

  it("indique honnêtement qu'aucune correspondance n'a été trouvée", async () => {
    setup(vi.fn(async () => 0));
    fireEvent.click(screen.getByRole("button", { name: "Rechercher dans un dossier…" }));
    expect(await screen.findByText(/Aucun média correspondant/)).toBeInTheDocument();
  });

  it("n'affiche rien si la recherche est annulée", async () => {
    const search = vi.fn(async () => null);
    setup(search);
    fireEvent.click(screen.getByRole("button", { name: "Rechercher dans un dossier…" }));
    await waitFor(() => expect(search).toHaveBeenCalled());
    expect(screen.queryByText("Recherche terminée")).not.toBeInTheDocument();
  });

  it("peut être reporté", () => {
    const h = setup();
    fireEvent.click(screen.getByRole("button", { name: "Plus tard" }));
    expect(h.onClose).toHaveBeenCalled();
  });
});
