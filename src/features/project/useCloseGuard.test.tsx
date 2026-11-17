import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const tauri = vi.hoisted(() => ({ onCloseRequested: vi.fn() }));
vi.mock("@/lib/tauri", () => tauri);

import { useCloseGuard } from "./useCloseGuard";

async function register(isDirty: boolean) {
  let handler: ((prevent: () => void) => void) | undefined;
  const unlisten = vi.fn();
  tauri.onCloseRequested.mockImplementation(async (h: (prevent: () => void) => void) => {
    handler = h;
    return unlisten;
  });
  const onDirtyClose = vi.fn();
  const hook = renderHook(({ dirty }) => useCloseGuard(dirty, onDirtyClose), { initialProps: { dirty: isDirty } });
  await waitFor(() => expect(handler).toBeDefined());
  return { hook, handler: handler!, onDirtyClose, unlisten };
}

describe("useCloseGuard", () => {
  it("laisse la fenêtre se fermer quand tout est enregistré", async () => {
    const { handler, onDirtyClose } = await register(false);
    const prevent = vi.fn();
    handler(prevent);
    expect(prevent).not.toHaveBeenCalled();
    expect(onDirtyClose).not.toHaveBeenCalled();
  });

  it("empêche la fermeture et délègue la décision quand il y a des modifications", async () => {
    const { handler, onDirtyClose } = await register(true);
    const prevent = vi.fn();
    handler(prevent);
    expect(prevent).toHaveBeenCalledTimes(1);
    expect(onDirtyClose).toHaveBeenCalledTimes(1);
  });

  it("tient compte de l'état le plus récent sans se réabonner", async () => {
    const { hook, handler, onDirtyClose } = await register(false);
    hook.rerender({ dirty: true });
    const prevent = vi.fn();
    handler(prevent);
    expect(prevent).toHaveBeenCalled();
    expect(onDirtyClose).toHaveBeenCalled();
    expect(tauri.onCloseRequested).toHaveBeenCalledTimes(1);
  });

  it("se désabonne au démontage", async () => {
    const { hook, unlisten } = await register(false);
    hook.unmount();
    expect(unlisten).toHaveBeenCalled();
  });
});
