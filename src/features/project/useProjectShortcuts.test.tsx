import { fireEvent, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useProjectShortcuts } from "./useProjectShortcuts";

function setup() {
  const handlers = { onNew: vi.fn(), onOpen: vi.fn(), onSave: vi.fn(), onSaveAs: vi.fn() };
  const { unmount } = renderHook(() => useProjectShortcuts(handlers));
  return { handlers, unmount };
}

describe("useProjectShortcuts", () => {
  it("déclenche Ctrl+N, Ctrl+O, Ctrl+S et Ctrl+Maj+S", () => {
    const { handlers } = setup();
    fireEvent.keyDown(window, { key: "n", ctrlKey: true });
    fireEvent.keyDown(window, { key: "o", ctrlKey: true });
    fireEvent.keyDown(window, { key: "s", ctrlKey: true });
    fireEvent.keyDown(window, { key: "S", ctrlKey: true, shiftKey: true });
    expect(handlers.onNew).toHaveBeenCalledTimes(1);
    expect(handlers.onOpen).toHaveBeenCalledTimes(1);
    expect(handlers.onSave).toHaveBeenCalledTimes(1);
    expect(handlers.onSaveAs).toHaveBeenCalledTimes(1);
  });

  it("accepte Cmd sur macOS et empêche le comportement par défaut du navigateur", () => {
    const { handlers } = setup();
    const notPrevented = fireEvent.keyDown(window, { key: "s", metaKey: true });
    expect(handlers.onSave).toHaveBeenCalledTimes(1);
    expect(notPrevented).toBe(false);
  });

  it("ignore les touches sans Ctrl, avec Alt, ou répétées", () => {
    const { handlers } = setup();
    fireEvent.keyDown(window, { key: "s" });
    fireEvent.keyDown(window, { key: "s", ctrlKey: true, altKey: true });
    fireEvent.keyDown(window, { key: "s", ctrlKey: true, repeat: true });
    fireEvent.keyDown(window, { key: "x", ctrlKey: true });
    expect(handlers.onSave).not.toHaveBeenCalled();
  });

  it("se désabonne au démontage", () => {
    const { handlers, unmount } = setup();
    unmount();
    fireEvent.keyDown(window, { key: "s", ctrlKey: true });
    expect(handlers.onSave).not.toHaveBeenCalled();
  });
});
