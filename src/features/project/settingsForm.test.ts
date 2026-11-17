import { describe, expect, it } from "vitest";
import { makeSnapshot } from "@/test/fixtures";
import { defaultSettingsForm, settingsFormFromProject, toSettingsInput } from "./settingsForm";

describe("settingsForm", () => {
  it("propose 1080p / 30 FPS / 48 kHz par défaut", () => {
    const { value, extras } = defaultSettingsForm("Test");
    expect(toSettingsInput(value, extras)).toEqual({
      name: "Test", width: 1920, height: 1080, frameRateNum: 30, frameRateDen: 1, sampleRate: 48_000
    });
  });

  it("couvre 720p, 1080p, 1440p et 4K", () => {
    const dims = ["720p", "1080p", "1440p", "2160p"].map((profile) => {
      const { value, extras } = defaultSettingsForm("x");
      const r = toSettingsInput({ ...value, profile }, extras);
      return [r.width, r.height];
    });
    expect(dims).toEqual([[1280, 720], [1920, 1080], [2560, 1440], [3840, 2160]]);
  });

  it("reconstitue le formulaire d'un projet standard sans valeurs hors préréglage", () => {
    const { value, extras } = settingsFormFromProject(makeSnapshot({ width: 2560, height: 1440, frameRateNum: 50 }));
    expect(value).toMatchObject({ profile: "1440p", frameRate: "50/1", sampleRate: 48_000 });
    expect(extras).toEqual({});
  });

  it("conserve les valeurs hors préréglage d'un fichier externe au lieu de les écraser", () => {
    const snap = makeSnapshot({ width: 1000, height: 562, frameRateNum: 30_000, frameRateDen: 1001, sampleRate: 32_000 });
    const { value, extras } = settingsFormFromProject(snap);
    expect(value.profile).toBe("custom");
    expect(toSettingsInput(value, extras)).toMatchObject({
      width: 1000, height: 562, frameRateNum: 30_000, frameRateDen: 1001, sampleRate: 32_000
    });
  });

  it("refuse un profil inconnu ou une cadence invalide", () => {
    const { value, extras } = defaultSettingsForm("x");
    expect(() => toSettingsInput({ ...value, profile: "custom" }, extras)).toThrow();
    expect(() => toSettingsInput({ ...value, frameRate: "abc" }, extras)).toThrow();
    expect(() => toSettingsInput({ ...value, frameRate: "30/0" }, extras)).toThrow();
  });
});
