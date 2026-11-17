import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { makeRecent } from "@/test/fixtures";
import { RecentProjects } from "./RecentProjects";

function setup(items = [makeRecent(), makeRecent({ path: "/p/Perdu/Perdu.kito", name: "Perdu", exists: false })], loaded = true) {
  const handlers = { onOpen: vi.fn(), onRemove: vi.fn(), onClear: vi.fn() };
  render(<RecentProjects items={items} loaded={loaded} {...handlers} />);
  return handlers;
}

describe("RecentProjects", () => {
  it("liste les projets avec leur chemin", () => {
    setup();
    expect(screen.getByText("Mon film")).toBeInTheDocument();
    expect(screen.getByText("/projets/Mon film/Mon film.kito")).toBeInTheDocument();
    expect(screen.getByText("2 projets")).toBeInTheDocument();
  });

  it("ouvre un projet existant au clic", () => {
    const h = setup();
    fireEvent.click(screen.getByText("Mon film"));
    expect(h.onOpen).toHaveBeenCalledWith("/projets/Mon film/Mon film.kito");
  });

  it("marque un projet introuvable, non ouvrable, mais retirable", () => {
    const h = setup();
    expect(screen.getByText("Introuvable")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Perdu"));
    expect(h.onOpen).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Retirer Perdu des projets récents" }));
    expect(h.onRemove).toHaveBeenCalledWith("/p/Perdu/Perdu.kito");
  });

  it("permet de vider la liste", () => {
    const h = setup();
    fireEvent.click(screen.getByRole("button", { name: "Vider la liste" }));
    expect(h.onClear).toHaveBeenCalled();
  });

  it("affiche un état vide honnête, sans projets inventés", () => {
    setup([]);
    expect(screen.getByText("Aucun projet récent")).toBeInTheDocument();
    expect(screen.queryByRole("list")).not.toBeInTheDocument();
  });

  it("indique le chargement avant la première réponse", () => {
    setup([], false);
    expect(screen.getByText("Chargement…")).toBeInTheDocument();
  });
});
