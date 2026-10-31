import type { TFunction } from "i18next";
import type { AppErrorPayload } from "./types";

export function normalizeError(error: unknown): AppErrorPayload {
  if (typeof error === "string") return { message: error };
  if (error && typeof error === "object") {
    const payload = error as AppErrorPayload;
    return {
      code: payload.code,
      message: payload.message,
      detail: payload.detail,
    };
  }
  return { message: String(error ?? "") };
}

export function errorText(error: unknown, t: TFunction): string {
  const payload = normalizeError(error);
  if (payload.code) {
    const key = `errors.${payload.code}`;
    const translated = t(key, { defaultValue: "" });
    if (translated) return translated;
  }
  if (payload.message) return payload.message;
  return t("errors.ffmpeg");
}

export function errorDetail(error: unknown): string {
  return normalizeError(error).detail ?? "";
}
