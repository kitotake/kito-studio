export type VideoProfile = {
  label: string;
  width: number;
  height: number;
};

export const VIDEO_PROFILES: readonly VideoProfile[] = [
  { label: "Full HD", width: 1920, height: 1080 },
  { label: "4K UHD", width: 3840, height: 2160 },
  { label: "Vertical", width: 1080, height: 1920 },
  { label: "HD", width: 1280, height: 720 }
];

export const FRAME_RATES: readonly number[] = [24, 25, 30, 50, 60];
export const DEFAULT_FRAME_RATE = 30;
