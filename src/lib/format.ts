import type { StreamInfo } from "./types";

export function formatDuration(seconds: number): string {
  if (!seconds || seconds <= 0) return "—";
  const total = Math.round(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = total % 60;
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`
    : `${minutes}:${String(secs).padStart(2, "0")}`;
}

export function formatTimecode(seconds: number): string {
  const total = Math.max(0, seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = Math.floor(total % 60);
  const millis = Math.round((total - Math.floor(total)) * 1000);
  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}.${String(millis).padStart(3, "0")}`;
}

export function parseTimecode(value: string): number | null {
  const text = value.trim().replace(",", ".");
  if (!text) return null;
  const parts = text.split(":");
  if (parts.length > 3) return null;
  const numbers = parts.map(Number);
  if (numbers.some((number) => !Number.isFinite(number) || number < 0)) return null;
  if (numbers.length >= 2 && numbers[numbers.length - 1] >= 60) return null;
  if (numbers.length === 3 && numbers[1] >= 60) return null;
  return numbers.reduce((total, number) => total * 60 + number, 0);
}

export function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${index === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[index]}`;
}

export function baseName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

export function parentName(path: string): string {
  const parts = path.split(/[\\/]/);
  parts.pop();
  return parts.join("\\") || ".";
}

export function channelText(stream: StreamInfo): string {
  if (!stream.channels) return "—";
  const layout = stream.channelLayout.trim();
  return layout ? `${stream.channels} (${layout})` : String(stream.channels);
}

export function bitrateText(bits: number | null): string {
  if (!bits) return "—";
  return `${Math.round(bits / 1000)} kb/s`;
}

export function isDefault(stream: StreamInfo): boolean {
  return Boolean(stream.disposition.default);
}

export function isForced(stream: StreamInfo): boolean {
  return Boolean(stream.disposition.forced);
}

export function isEnglish(code: string): boolean {
  return ["eng", "en", "english", "en-us", "en-gb"].includes(code.toLowerCase());
}

export function isSpanish(code: string): boolean {
  return [
    "spa",
    "es",
    "spanish",
    "es-es",
    "es-mx",
    "es-419",
    "latino",
    "latin",
  ].includes(code.toLowerCase());
}

export function trackDetail(stream: StreamInfo): string {
  if (stream.type === "video") {
    const parts = [];
    if (stream.width && stream.height) parts.push(`${stream.width}×${stream.height}`);
    if (stream.fps) parts.push(`${stream.fps.toFixed(3).replace(".000", "")} fps`);
    if (stream.rotation) parts.push(`${stream.rotation}°`);
    return parts.join(" · ") || "—";
  }
  if (stream.type === "audio") return bitrateText(stream.bitRate);
  if (stream.type === "attachment") return stream.title || stream.codec;
  return "—";
}
