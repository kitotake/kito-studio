import type { ProjectSnapshot } from "@/types/project";

const numberFormat = new Intl.NumberFormat("fr-FR");

export function formatResolution(project: Pick<ProjectSnapshot, "width" | "height">): string {
  return `${project.width} × ${project.height}`;
}

export function formatFrameRateValue(num: number, den: number): string {
  const fps = num / den;
  return `${numberFormat.format(Number.isInteger(fps) ? fps : Number(fps.toFixed(3)))} FPS`;
}

export function formatFrameRate(
  project: Pick<ProjectSnapshot, "frameRateNum" | "frameRateDen">
): string {
  return formatFrameRateValue(project.frameRateNum, project.frameRateDen);
}

export function formatSampleRateKhz(project: Pick<ProjectSnapshot, "sampleRate">): string {
  return `${numberFormat.format(project.sampleRate / 1000)} kHz`;
}

export function formatSampleRateHz(project: Pick<ProjectSnapshot, "sampleRate">): string {
  return `${numberFormat.format(project.sampleRate)} Hz`;
}

export function formatDateTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleString("fr-FR", { dateStyle: "medium", timeStyle: "short" });
}

export function formatTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
}
