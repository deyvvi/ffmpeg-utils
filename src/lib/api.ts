import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AmfStatus,
  AppInfo,
  EncoderReport,
  CommandPlan,
  Config,
  JobFinished,
  JobRequest,
  LogEvent,
  MediaInfo,
  OutputRequest,
  OutputSuggestion,
  ProgressEvent,
  ToolsStatus,
} from "./types";

export const api = {
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  saveConfig: (config: Config) => invoke<void>("save_config_command", { config }),
  detectTools: () => invoke<ToolsStatus>("detect_tools_command"),
  checkAmf: () => invoke<AmfStatus>("check_amf_command"),
  checkEncoders: () => invoke<EncoderReport>("check_encoders_command"),
  probeFile: (path: string) => invoke<MediaInfo>("probe_file_command", { path }),
  probeTrackFile: (path: string) =>
    invoke<MediaInfo>("probe_track_file_command", { path }),
  previewPlan: (request: JobRequest) =>
    invoke<CommandPlan>("preview_plan_command", { request }),
  startJob: (request: JobRequest) =>
    invoke<string>("start_job_command", { request }),
  cancelJob: (jobId: string) =>
    invoke<void>("cancel_job_command", { jobId }),
  suggestOutputs: (request: OutputRequest) =>
    invoke<OutputSuggestion[]>("suggest_outputs_command", { request }),
  pathExists: (path: string) => invoke<boolean>("path_exists", { path }),
  setWindowTheme: (theme: string) =>
    invoke<void>("set_window_theme", { theme }),
  setWindowSize: (width: number, height: number) =>
    invoke<void>("set_window_size", { width, height }),
};

export function onJobLog(callback: (event: LogEvent) => void) {
  return listen<LogEvent>("job:log", (event) => callback(event.payload));
}

export function onJobProgress(callback: (event: ProgressEvent) => void) {
  return listen<ProgressEvent>("job:progress", (event) => callback(event.payload));
}

export function onJobFinished(callback: (event: JobFinished) => void) {
  return listen<JobFinished>("job:finished", (event) => callback(event.payload));
}
