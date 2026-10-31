export type StreamKind = "video" | "audio" | "subtitle" | "attachment" | string;

export interface StreamInfo {
  index: number;
  type: StreamKind;
  codec: string;
  codecLong: string;
  profile: string;
  language: string;
  rawLanguage: string;
  title: string;
  channels: number | null;
  channelLayout: string;
  bitRate: number | null;
  width: number | null;
  height: number | null;
  fps: number | null;
  sampleAspectRatio: string;
  displayAspectRatio: string;
  rotation: number;
  disposition: Record<string, number>;
  duration: number | null;
}

export interface MediaInfo {
  path: string;
  size: number;
  duration: number;
  formatName: string;
  formatLongName: string;
  tags: Record<string, string>;
  chapterCount: number;
  streams: StreamInfo[];
  isFullHdCanvas: boolean;
}

export interface Config {
  language: string;
  theme: "dark" | "light" | "system" | string;
  accent: string;
  density: "comfortable" | "compact" | string;
  ffmpegDir: string;
  outputDir: string;
  quality: string;
  qpI: number;
  qpP: number;
  softwareCrf: number;
  windowWidth: number;
  windowHeight: number;
  logVisible: boolean;
}

export interface ToolsStatus {
  found: boolean;
  ffmpeg: string;
  ffprobe: string;
  version: string;
  error: string;
}

export interface AppInfo {
  config: Config;
  tools: ToolsStatus;
  initialFile: string | null;
}

export interface AmfStatus {
  available: boolean;
  message: string;
  detail: string;
  options: string[];
}

export interface EncoderStatus {
  id: string;
  label: string;
  codec: string;
  vendor: string;
  hardware: boolean;
  available: boolean;
  message: string;
  detail: string;
}

export interface EncoderReport {
  encoders: EncoderStatus[];
  amf: AmfStatus;
}
export interface Check {
  ok: boolean;
  code: string;
  params: Record<string, unknown>;
}

export interface JobFinished {
  jobId: string;
  ok: boolean;
  kind: JobKind;
  message: string;
  code: string;
  detail: string;
  outputs: string[];
  checks: Check[];
  elapsed: number;
}

export interface ProgressEvent {
  jobId: string;
  percent: number;
  detail: string;
  speed: string;
  outTime: number;
  duration: number;
}

export interface LogEvent {
  jobId: string;
  line: string;
}

export type JobKind = "normalize" | "singleAudio" | "trim" | "editTracks" | "extract";

export interface OutputSuggestion {
  path: string;
  exists: boolean;
  alternate: string;
  streamIndex: number | null;
}

export interface CommandPlan {
  commands: string[][];
  display: string[];
  notes: string[];
  outputs: string[];
}

export type ExtractFormat =
  | "copy"
  | "mp3"
  | "m4a"
  | "flac"
  | "wav"
  | "ac3"
  | "opus"
  | "srt"
  | "vtt";

export interface ExtractItemRequest {
  streamIndex: number;
  output: string;
  replace: boolean;
  format: ExtractFormat;
}

export interface ExtractChoice {
  streamIndex: number;
  format: ExtractFormat;
}

export interface SourceTrackChoice {
  streamIndex: number;
  default: boolean;
}

export interface AddedTrackRequest {
  path: string;
  streamIndex: number;
  kind: "audio" | "subtitle";
  language: string;
  title: string;
  default: boolean;
  forced: boolean;
}

export type JobRequest =
  | {
      kind: "normalize";
      path: string;
      output: string;
      videoMode: string;
      width: number;
      height: number;
      replace: boolean;
    }
  | {
      kind: "trim";
      path: string;
      output: string;
      start: number;
      end: number;
      mode: "fast" | "precise" | "software";
      replace: boolean;
    }
  | {
      kind: "singleAudio";
      path: string;
      output: string;
      language: string;
      audioIndex: number | null;
      replace: boolean;
    }
  | {
      kind: "editTracks";
      path: string;
      output: string;
      audio: SourceTrackChoice[];
      subtitles: SourceTrackChoice[];
      added: AddedTrackRequest[];
      replace: boolean;
    }
  | {
      kind: "extract";
      path: string;
      items: ExtractItemRequest[];
    };

export type OutputRequest =
  | { kind: "normalize"; path: string; width: number; height: number }
  | { kind: "trim"; path: string; start: number; end: number }
  | { kind: "singleAudio"; path: string; language: string }
  | { kind: "editTracks"; path: string }
  | {
      kind: "extract";
      path: string;
      outputDir: string;
      items: ExtractChoice[];
    };

export type TabId = "convert" | "trim" | "tracks" | "extract";

export interface AppErrorPayload {
  code?: string;
  message?: string;
  detail?: string;
}
