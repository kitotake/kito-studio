import { describe, expect, it } from "vitest";
import { KitoError, toKitoError } from "./errors";

describe("toKitoError", () => {
  it("convertit une erreur sérialisée par Rust en conservant le code", () => {
    for (const code of [
      "IO", "INVALID_INPUT", "NOT_FOUND", "ALREADY_EXISTS", "PERMISSION_DENIED",
      "DISK_FULL", "INVALID_FORMAT", "INCOMPATIBLE_VERSION", "INTERNAL"
    ]) {
      const error = toKitoError({ code, message: "msg" });
      expect(error).toBeInstanceOf(KitoError);
      expect(error.code).toBe(code);
      expect(error.message).toBe("msg");
    }
  });

  it("ne fait pas confiance à un code inconnu", () => {
    expect(toKitoError({ code: "ROOT_ACCESS", message: "x" }).code).toBe("UNKNOWN");
  });

  it("gère Error, chaînes et valeurs quelconques", () => {
    const cause = new Error("boum");
    expect(toKitoError(cause)).toMatchObject({ code: "UNKNOWN", message: "boum", cause });
    expect(toKitoError("texte").message).toBe("texte");
    expect(toKitoError(42).code).toBe("UNKNOWN");
    expect(toKitoError(null).message).toBe("Erreur inconnue.");
  });

  it("laisse passer une KitoError telle quelle", () => {
    const error = new KitoError("DISK_FULL", "plein");
    expect(toKitoError(error)).toBe(error);
  });
});
