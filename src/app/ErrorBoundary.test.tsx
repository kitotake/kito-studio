import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ErrorBoundary } from "./ErrorBoundary";

function Boom(): never {
  throw new Error("crash de rendu volontaire");
}

describe("ErrorBoundary", () => {
  it("affiche l'écran de secours au lieu d'un écran blanc", () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    render(
      <ErrorBoundary>
        <Boom />
      </ErrorBoundary>
    );
    expect(screen.getByRole("alert")).toHaveTextContent("Kito Studio a rencontré un problème");
    expect(screen.getByText("crash de rendu volontaire")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Recharger l'application" })).toBeInTheDocument();
  });

  it("n'affiche rien de plus quand tout va bien", () => {
    render(
      <ErrorBoundary>
        <p>contenu</p>
      </ErrorBoundary>
    );
    expect(screen.getByText("contenu")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
