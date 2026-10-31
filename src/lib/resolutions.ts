export interface ResolutionPreset {
  id: string;
  width: number;
  height: number;
  label: string;
}

export const DEFAULT_TARGET_WIDTH = 1920;
export const DEFAULT_TARGET_HEIGHT = 1080;

export const RESOLUTION_PRESETS: ResolutionPreset[] = [
  { id: "360p", width: 640, height: 360, label: "360p" },
  { id: "480p", width: 854, height: 480, label: "480p" },
  { id: "720p", width: 1280, height: 720, label: "720p" },
  { id: "1080p", width: 1920, height: 1080, label: "1080p" },
  { id: "1440p", width: 2560, height: 1440, label: "1440p" },
  { id: "2160p", width: 3840, height: 2160, label: "2160p / 4K" },
];

export function validateResolution(width: number, height: number): string | null {
  if (!Number.isFinite(width) || !Number.isFinite(height)) {
    return "invalid";
  }
  if (width < 16 || height < 16) {
    return "small";
  }
  if (width > 7680 || height > 7680) {
    return "large";
  }
  if (width % 2 !== 0 || height % 2 !== 0) {
    return "even";
  }
  return null;
}
