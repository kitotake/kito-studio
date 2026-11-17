import type { ProjectSnapshot, SettingsInput } from "@/types/project";
import {
  DEFAULT_FRAME_RATE,
  DEFAULT_PROFILE_ID,
  DEFAULT_SAMPLE_RATE,
  FRAME_RATES,
  SAMPLE_RATES,
  VIDEO_PROFILES,
  findProfile
} from "./projectProfiles";

export type SettingsFormValue = {
  name: string;
  /** Identifiant d'un profil prédéfini, ou "custom" pour une résolution héritée d'un fichier. */
  profile: string;
  /** « num/den », par exemple « 30/1 » ou « 30000/1001 ». */
  frameRate: string;
  sampleRate: number;
};

/** Valeurs hors des préréglages, conservées telles quelles quand un fichier les contient. */
export type SettingsFormExtras = {
  customResolution?: { width: number; height: number };
  customFrameRate?: { num: number; den: number };
  customSampleRate?: number;
};

export const CUSTOM_PROFILE = "custom";

export function defaultSettingsForm(name: string): { value: SettingsFormValue; extras: SettingsFormExtras } {
  return {
    value: {
      name,
      profile: DEFAULT_PROFILE_ID,
      frameRate: `${DEFAULT_FRAME_RATE}/1`,
      sampleRate: DEFAULT_SAMPLE_RATE
    },
    extras: {}
  };
}

export function settingsFormFromProject(project: ProjectSnapshot): {
  value: SettingsFormValue;
  extras: SettingsFormExtras;
} {
  const extras: SettingsFormExtras = {};
  const preset = findProfile(project.width, project.height);
  if (!preset) extras.customResolution = { width: project.width, height: project.height };

  const standardRate = project.frameRateDen === 1 && FRAME_RATES.includes(project.frameRateNum);
  if (!standardRate) extras.customFrameRate = { num: project.frameRateNum, den: project.frameRateDen };

  if (!SAMPLE_RATES.includes(project.sampleRate)) extras.customSampleRate = project.sampleRate;

  return {
    value: {
      name: project.name,
      profile: preset?.id ?? CUSTOM_PROFILE,
      frameRate: `${project.frameRateNum}/${project.frameRateDen}`,
      sampleRate: project.sampleRate
    },
    extras
  };
}

export function toSettingsInput(value: SettingsFormValue, extras: SettingsFormExtras): SettingsInput {
  const preset = VIDEO_PROFILES.find((p) => p.id === value.profile);
  const resolution = preset ?? extras.customResolution;
  if (!resolution) throw new Error("Profil vidéo inconnu.");

  const [num, den] = value.frameRate.split("/").map(Number);
  if (!Number.isInteger(num) || !Number.isInteger(den) || !num || !den) {
    throw new Error("Fréquence d'images invalide.");
  }
  return {
    name: value.name.trim(),
    width: resolution.width,
    height: resolution.height,
    frameRateNum: num,
    frameRateDen: den,
    sampleRate: value.sampleRate
  };
}
