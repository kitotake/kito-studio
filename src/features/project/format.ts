import type { ProjectProfile } from "@/types/project";

const numberFormat = new Intl.NumberFormat("fr-FR");

export function formatResolution(project: ProjectProfile): string {
  return `${project.width} × ${project.height}`;
}

export function formatFrameRate(project: ProjectProfile): string {
  const fps = project.fpsNumerator / project.fpsDenominator;
  return `${numberFormat.format(Number.isInteger(fps) ? fps : Number(fps.toFixed(3)))} FPS`;
}

export function formatSampleRateKhz(project: ProjectProfile): string {
  return `${numberFormat.format(project.audioSampleRate / 1000)} kHz`;
}

export function formatSampleRateHz(project: ProjectProfile): string {
  return `${numberFormat.format(project.audioSampleRate)} Hz`;
}
