import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useUnsavedGuard } from "./useUnsavedGuard";

function setup(isDirty: boolean, overrides: { save?: () => Promise<boolean>; discard?: () => Promise<boolean> } = {}) {
  const save = vi.fn(overrides.save ?? (async () => true));
  const discard = vi.fn(overrides.discard ?? (async () => true));
  const action = vi.fn();
  const hook = renderHook(() => useUnsavedGuard({ isDirty, save, discard }));
  return { ...hook, save, discard, action };
}

describe("useUnsavedGuard", () => {
  it("exécute l'action immédiatement quand le projet est propre", () => {
    const { result, action } = setup(false);
    act(() => result.current.run(action));
    expect(action).toHaveBeenCalledTimes(1);
    expect(result.current.isPending).toBe(false);
  });

  it("met l'action en attente quand le projet a des modifications", () => {
    const { result, action } = setup(true);
    act(() => result.current.run(action));
    expect(action).not.toHaveBeenCalled();
    expect(result.current.isPending).toBe(true);
  });

  it("« Enregistrer » sauvegarde puis exécute l'action", async () => {
    const { result, action, save, discard } = setup(true);
    act(() => result.current.run(action));
    await act(() => result.current.resolve("save"));
    expect(save).toHaveBeenCalledTimes(1);
    expect(discard).not.toHaveBeenCalled();
    expect(action).toHaveBeenCalledTimes(1);
    expect(result.current.isPending).toBe(false);
  });

  it("« Ne pas enregistrer » abandonne les modifications puis exécute l'action", async () => {
    const { result, action, save, discard } = setup(true);
    act(() => result.current.run(action));
    await act(() => result.current.resolve("discard"));
    expect(discard).toHaveBeenCalledTimes(1);
    expect(save).not.toHaveBeenCalled();
    expect(action).toHaveBeenCalledTimes(1);
  });

  it("« Annuler » n'exécute rien et referme la demande", async () => {
    const { result, action, save, discard } = setup(true);
    act(() => result.current.run(action));
    await act(() => result.current.resolve("cancel"));
    expect([save, discard, action].every((f) => f.mock.calls.length === 0)).toBe(true);
    expect(result.current.isPending).toBe(false);
  });

  it("si l'enregistrement échoue, l'action n'est pas exécutée et reste en attente", async () => {
    const { result, action } = setup(true, { save: async () => false });
    act(() => result.current.run(action));
    await act(() => result.current.resolve("save"));
    expect(action).not.toHaveBeenCalled();
    expect(result.current.isPending).toBe(true);
    expect(result.current.working).toBe(false);
  });
});
