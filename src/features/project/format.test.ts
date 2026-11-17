import { describe, expect, it } from "vitest";
import { formatFrameRateValue, formatSampleRateKhz, formatResolution } from "./format";

describe("format", () => {
  it("formate les cadences entières et fractionnaires", () => {
    expect(formatFrameRateValue(30, 1)).toBe("30 FPS");
    expect(formatFrameRateValue(60, 1)).toBe("60 FPS");
    expect(formatFrameRateValue(30_000, 1001)).toBe("29,97 FPS");
    expect(formatFrameRateValue(24_000, 1001)).toBe("23,976 FPS");
  });

  it("formate résolution et fréquence audio", () => {
    expect(formatResolution({ width: 1920, height: 1080 })).toBe("1920 × 1080");
    expect(formatSampleRateKhz({ sampleRate: 48_000 })).toBe("48 kHz");
    expect(formatSampleRateKhz({ sampleRate: 44_100 })).toBe("44,1 kHz");
  });
});
