import { beforeEach, describe, expect, it, vi } from "vitest";

const core = vi.hoisted(() => ({ invoke: vi.fn(), isTauri: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => core);

import { KitoError } from "@/lib/errors";
import { invokeCommand } from "./client";

describe("invokeCommand", () => {
  beforeEach(() => {
    core.invoke.mockReset();
    core.isTauri.mockReset();
  });

  it("refuse proprement hors de l'application native", async () => {
    core.isTauri.mockReturnValue(false);
    await expect(invokeCommand("x")).rejects.toMatchObject({ code: "UNAVAILABLE" });
    expect(core.invoke).not.toHaveBeenCalled();
  });

  it("transmet la commande et ses arguments", async () => {
    core.isTauri.mockReturnValue(true);
    core.invoke.mockResolvedValue({ ok: 1 });
    await expect(invokeCommand("save_project", { a: 1 })).resolves.toEqual({ ok: 1 });
    expect(core.invoke).toHaveBeenCalledWith("save_project", { a: 1 });
  });

  it("convertit une erreur Rust { code, message } en KitoError", async () => {
    core.isTauri.mockReturnValue(true);
    core.invoke.mockRejectedValue({ code: "DISK_FULL", message: "Écriture : espace disque insuffisant" });
    const error = await invokeCommand("save_project").catch((e: unknown) => e);
    expect(error).toBeInstanceOf(KitoError);
    expect(error).toMatchObject({ code: "DISK_FULL", message: "Écriture : espace disque insuffisant" });
  });
});
