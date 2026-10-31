import type { TFunction } from "i18next";
import type { Check } from "./types";
import { formatDuration, formatSize } from "./format";

export function checkText(check: Check, t: TFunction): string {
  const params = (check.params ?? {}) as Record<string, unknown>;
  const language = params.language ? String(params.language) : "und";
  switch (check.code) {
    case "file_exists":
      return t("checks.file_exists", { size: formatSize(Number(params.size ?? 0)) });
    case "resolution":
      return t("checks.resolution", {
        width: params.width ?? "?",
        height: params.height ?? "?",
      });
    case "audio_language":
      return t("checks.audio_language", {
        language: t(`languages.${language}`, { defaultValue: language }),
      });
    case "duration":
      return t("checks.duration", {
        duration: formatDuration(Number(params.duration ?? 0)),
        original: formatDuration(Number(params.original ?? 0)),
      });
    case "trim_duration":
      return t("checks.trim_duration", {
        duration: formatDuration(Number(params.duration ?? 0)),
        requested: formatDuration(Number(params.requested ?? 0)),
      });
    case "extract_stream":
      return t("checks.extract_stream", {
        kind:
          params.kind === "audio"
            ? t("tracks.audio")
            : t("tracks.subtitle"),
        count: params.count ?? 0,
      });
    case "probe_failed":
      return `${t("checks.probe_failed")} ${String(params.message ?? "")}`.trim();
    default:
      return t(`checks.${check.code}`, {
        ...params,
        defaultValue: check.code,
      });
  }
}
