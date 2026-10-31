import { create } from "zustand";
import i18n from "./i18n";
import { api } from "./lib/api";
import { applyAppearance } from "./lib/theme";
import type {
  AmfStatus,
  CommandPlan,
  Config,
  EncoderStatus,
  JobFinished,
  JobRequest,
  MediaInfo,
  ProgressEvent,
  TabId,
  ToolsStatus,
} from "./lib/types";

const defaultConfig: Config = {
  language: "en",
  theme: "dark",
  accent: "#77756f",
  density: "comfortable",
  ffmpegDir: "",
  outputDir: "",
  quality: "speed",
  qpI: 20,
  qpP: 22,
  softwareCrf: 20,
  windowWidth: 1180,
  windowHeight: 780,
  logVisible: false,
};

export interface ExtractTarget {
  media: MediaInfo;
  path: string;
  preselect: number[];
}

interface AppStore {
  ready: boolean;
  config: Config;
  tools: ToolsStatus | null;
  amf: AmfStatus | null;
  encoders: EncoderStatus[];
  targetWidth: number;
  targetHeight: number;
  media: MediaInfo | null;
  probing: boolean;
  tab: TabId;
  busy: boolean;
  jobId: string | null;
  progress: number;
  detail: string;
  logs: string[];
  result: JobFinished | null;
  settingsOpen: boolean;
  preview: CommandPlan | null;
  extractTarget: ExtractTarget | null;
  init: () => Promise<void>;
  setTab: (tab: TabId) => void;
  setTargetSize: (width: number, height: number) => void;
  updateConfig: (patch: Partial<Config>) => Promise<void>;
  checkAmf: () => Promise<void>;
  checkEncoders: () => Promise<void>;
  loadFile: (path: string) => Promise<void>;
  clearFile: () => void;
  startJob: (request: JobRequest) => Promise<void>;
  cancel: () => Promise<void>;
  appendLog: (line: string) => void;
  handleProgress: (event: ProgressEvent) => void;
  handleFinished: (event: JobFinished) => void;
  setSettingsOpen: (open: boolean) => void;
  setTools: (tools: ToolsStatus) => void;
  clearLogs: () => void;
  setPreview: (preview: CommandPlan | null) => void;
  setExtractTarget: (target: ExtractTarget | null) => void;
  dismissResult: () => void;
}

function appendBounded(lines: string[], line: string): string[] {
  const next = [...lines, line];
  return next.length > 2500 ? next.slice(next.length - 2500) : next;
}

export const useAppStore = create<AppStore>((set, get) => ({
  ready: false,
  config: defaultConfig,
  tools: null,
  amf: null,
  encoders: [],
  targetWidth: 1920,
  targetHeight: 1080,
  media: null,
  probing: false,
  tab: "convert",
  busy: false,
  jobId: null,
  progress: 0,
  detail: "",
  logs: [],
  result: null,
  settingsOpen: false,
  preview: null,
  extractTarget: null,

  init: async () => {
    try {
      const info = await api.getAppInfo();
      applyAppearance(info.config);
      await i18n.changeLanguage(info.config.language || "en");
      set({ config: info.config, tools: info.tools, ready: true });
      void get().checkEncoders();
      if (info.initialFile) {
        void get().loadFile(info.initialFile);
      }
    } catch {
      set({ ready: true });
    }
  },

  setTab: (tab) => set({ tab }),

  setTargetSize: (targetWidth, targetHeight) => set({ targetWidth, targetHeight }),

  updateConfig: async (patch) => {
    const next = { ...get().config, ...patch };
    set({ config: next });
    applyAppearance(next);
    if (patch.language && patch.language !== i18n.language) {
      await i18n.changeLanguage(patch.language);
    }
    await api.saveConfig(next);
  },

  checkAmf: async () => {
    try {
      const amf = await api.checkAmf();
      set({ amf });
    } catch {
      set({
        amf: {
          available: false,
          message: i18n.t("status.amfMissing"),
          detail: "",
          options: [],
        },
      });
    }
  },

  checkEncoders: async () => {
    try {
      const report = await api.checkEncoders();
      set({ encoders: report.encoders, amf: report.amf });
    } catch {
      set({ encoders: [] });
    }
  },

  loadFile: async (path) => {
    set({ probing: true });
    try {
      const media = await api.probeFile(path);
      set({
        media,
        probing: false,
        result: null,
        progress: 0,
        detail: i18n.t("progress.waiting"),
        logs: appendBounded(
          get().logs,
          `[${new Date().toLocaleTimeString()}] ${i18n.t("file.loaded")}: ${path}`,
        ),
      });
    } catch (error) {
      set({ probing: false });
      throw error;
    }
  },

  clearFile: () =>
    set({
      media: null,
      result: null,
      progress: 0,
      detail: "",
      jobId: null,
      busy: false,
    }),

  startJob: async (request) => {
    set({
      busy: true,
      jobId: null,
      progress: 0,
      detail: i18n.t("progress.preparing"),
      result: null,
    });
    try {
      const jobId = await api.startJob(request);
      set({ jobId });
    } catch (error) {
      set({ busy: false });
      throw error;
    }
  },

  cancel: async () => {
    const jobId = get().jobId;
    if (jobId) {
      await api.cancelJob(jobId);
    }
  },

  appendLog: (line) => {
    if (!line.trim()) return;
    const stamp = new Date().toLocaleTimeString();
    set({ logs: appendBounded(get().logs, `[${stamp}] ${line}`) });
  },

  handleProgress: (event) => {
    set({ progress: event.percent, detail: event.detail });
  },

  handleFinished: (event) => {
    set({
      busy: false,
      jobId: null,
      progress: event.ok ? 100 : 0,
      detail: event.ok ? i18n.t("progress.completed") : i18n.t("progress.failed"),
      result: event,
      logs: appendBounded(
        get().logs,
        `[${new Date().toLocaleTimeString()}] ${event.message}`,
      ),
    });
  },

  setSettingsOpen: (settingsOpen) => set({ settingsOpen }),
  setTools: (tools) => set({ tools }),
  clearLogs: () => set({ logs: [] }),
  setPreview: (preview) => set({ preview }),
  setExtractTarget: (extractTarget) => set({ extractTarget }),
  dismissResult: () => set({ result: null }),
}));

