import type { ProjectProfile } from "@/types/project";

export const PROJECT_SCHEMA_VERSION = 1;
export const DEFAULT_AUDIO_SAMPLE_RATE = 48_000;

export type NewProjectInput = {
  name: string;
  width: number;
  height: number;
  frameRate: number;
};

/**
 * Construit un profil de projet en mémoire.
 * Transitoire : la création et la persistance `.kito` passeront par le backend Rust.
 */
export function createProjectProfile(input: NewProjectInput): ProjectProfile {
  return {
    id: crypto.randomUUID(),
    name: input.name.trim() || "Sans titre",
    width: input.width,
    height: input.height,
    fpsNumerator: input.frameRate,
    fpsDenominator: 1,
    audioSampleRate: DEFAULT_AUDIO_SAMPLE_RATE,
    createdAt: new Date().toISOString(),
    schemaVersion: PROJECT_SCHEMA_VERSION
  };
}
