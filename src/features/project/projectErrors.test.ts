import { describe, expect, it } from "vitest";
import { KitoError } from "@/lib/errors";
import type { ErrorCode } from "@/types/errors";
import { describeProjectError } from "./projectErrors";

const ALL: ErrorCode[] = [
  "IO", "INVALID_INPUT", "NOT_FOUND", "ALREADY_EXISTS", "PERMISSION_DENIED",
  "DISK_FULL", "INVALID_FORMAT", "INCOMPATIBLE_VERSION", "INTERNAL", "UNAVAILABLE", "UNKNOWN"
];

describe("describeProjectError", () => {
  it("garde le message du backend et ajoute un titre selon le contexte", () => {
    const d = describeProjectError(new KitoError("DISK_FULL", "Écriture de « a.kito » : espace disque insuffisant"), "save");
    expect(d.title).toBe("L'enregistrement a échoué");
    expect(d.message).toContain("espace disque insuffisant");
    expect(d.context).toBe("save");
    expect(d.code).toBe("DISK_FULL");
  });

  it("rassure sur les modifications en mémoire lors d'un échec d'enregistrement", () => {
    for (const code of ["DISK_FULL", "PERMISSION_DENIED", "IO"] as const) {
      const d = describeProjectError(new KitoError(code, "x"), "save");
      expect(d.suggestion).toMatch(/modifications sont toujours en mémoire/);
    }
    expect(describeProjectError(new KitoError("DISK_FULL", "x"), "create").suggestion).not.toMatch(/en mémoire/);
  });

  it("oriente vers la récupération pour un fichier invalide", () => {
    const d = describeProjectError(new KitoError("INVALID_FORMAT", "x"), "open");
    expect(d.suggestion).toMatch(/Autosave|\.bak/);
  });

  it("distingue un projet introuvable à l'ouverture d'un dossier introuvable ailleurs", () => {
    expect(describeProjectError(new KitoError("NOT_FOUND", "x"), "open").suggestion).toMatch(/récents/);
    expect(describeProjectError(new KitoError("NOT_FOUND", "x"), "save").suggestion).toMatch(/disque externe/);
  });

  it("indique de mettre à jour pour une version incompatible", () => {
    expect(describeProjectError(new KitoError("INCOMPATIBLE_VERSION", "x"), "open").suggestion).toMatch(/Mettez à jour/);
  });

  it("produit toujours un titre et un message non vides, quel que soit le code", () => {
    for (const code of ALL) {
      const d = describeProjectError(new KitoError(code, "détail"), "other");
      expect(d.title).not.toBe("");
      expect(d.message).toBe("détail");
    }
  });
});
