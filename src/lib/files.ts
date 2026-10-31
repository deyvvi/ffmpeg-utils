import { open } from "@tauri-apps/plugin-dialog";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";

const videoExtensions = [
  "mkv",
  "mp4",
  "m4v",
  "mov",
  "avi",
  "ts",
  "m2ts",
  "mts",
  "webm",
  "wmv",
  "mpg",
  "mpeg",
  "flv",
];

export async function pickVideo(): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Video", extensions: videoExtensions }],
  });
  return typeof result === "string" ? result : null;
}

export async function pickTrackFile(kind: "audio" | "subtitle"): Promise<string | null> {
  const audioExtensions = ["aac", "m4a", "ac3", "eac3", "dts", "flac", "opus", "ogg", "mp3", "wav", "mka"];
  const subtitleExtensions = ["srt", "ass", "ssa", "vtt", "sup", "mks"];
  const extensions = kind === "audio" ? audioExtensions : subtitleExtensions;
  const result = await open({
    multiple: false,
    directory: false,
    filters: [
      { name: kind === "audio" ? "Audio" : "Subtitles", extensions },
      { name: "Media", extensions: [...videoExtensions, ...extensions] },
    ],
  });
  return typeof result === "string" ? result : null;
}

export async function pickFolder(defaultPath?: string): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: true,
    defaultPath,
  });
  return typeof result === "string" ? result : null;
}

export async function reveal(path: string): Promise<void> {
  await revealItemInDir(path);
}

export async function openFolder(path: string): Promise<void> {
  await openPath(path);
}
