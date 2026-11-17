export type VideoProfile = {
  id: string;
  label: string;
  width: number;
  height: number;
};

export const VIDEO_PROFILES: readonly VideoProfile[] = [
  { id: "720p", label: "720p HD", width: 1280, height: 720 },
  { id: "1080p", label: "1080p Full HD", width: 1920, height: 1080 },
  { id: "1440p", label: "1440p QHD", width: 2560, height: 1440 },
  { id: "2160p", label: "4K UHD", width: 3840, height: 2160 },
  { id: "vertical", label: "Vertical 1080p", width: 1080, height: 1920 }
];

export const FRAME_RATES: readonly number[] = [24, 25, 30, 50, 60];
export const SAMPLE_RATES: readonly number[] = [44_100, 48_000, 96_000];

export const DEFAULT_PROFILE_ID = "1080p";
export const DEFAULT_FRAME_RATE = 30;
export const DEFAULT_SAMPLE_RATE = 48_000;

export function findProfile(width: number, height: number): VideoProfile | undefined {
  return VIDEO_PROFILES.find((p) => p.width === width && p.height === height);
}
