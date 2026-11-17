import { describe, expect, it } from "vitest";
import { validateProjectName } from "./validateProjectName";

describe("validateProjectName", () => {
  it("accepte les noms normaux, accents et espaces compris", () => {
    for (const name of ["Mon film", "  Été – vidéo 2026 ", "a.b", "console", "auxiliaire"]) {
      expect(validateProjectName(name), name).toBeNull();
    }
  });

  it("refuse les noms vides, trop longs, avec caractères interdits ou réservés", () => {
    for (const name of ["", "   ", "a/b", "a\\b", "a:b", "a*b", "a?b", 'a"b', "a<b", "a|b", "fin.", "..", "CON", "nul", "com1.txt", "lpt9", "a\u0007b"]) {
      expect(validateProjectName(name), JSON.stringify(name)).not.toBeNull();
    }
    expect(validateProjectName("x".repeat(101))).not.toBeNull();
    expect(validateProjectName("x".repeat(100))).toBeNull();
  });

  it("explique le caractère fautif", () => {
    expect(validateProjectName("a/b")).toContain("/");
  });
});
