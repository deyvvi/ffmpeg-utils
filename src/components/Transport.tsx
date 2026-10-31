import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Button, ProgressBar, toast } from "@heroui/react";
import { useAppStore } from "../store";

export function Transport() {
  const { t } = useTranslation();
  const busy = useAppStore((state) => state.busy);
  const progress = useAppStore((state) => state.progress);
  const detail = useAppStore((state) => state.detail);
  const result = useAppStore((state) => state.result);
  const logs = useAppStore((state) => state.logs);
  const config = useAppStore((state) => state.config);
  const updateConfig = useAppStore((state) => state.updateConfig);
  const clearLogs = useAppStore((state) => state.clearLogs);
  const cancel = useAppStore((state) => state.cancel);
  const logRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight;
  }, [logs, config.logVisible]);

  const value = busy ? progress : result?.ok ? 100 : 0;
  const tone = busy ? "busy" : result ? (result.ok ? "ok" : "bad") : "idle";
  const stateLabel = busy
    ? t("shell.running")
    : result
      ? result.ok
        ? t("shell.complete")
        : t("shell.failed")
      : t("shell.ready");

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(logs.join("\n"));
      toast.success(t("log.copy"));
    } catch (error) {
      toast.danger(String(error));
    }
  };

  return (
    <footer className="transport" data-tone={tone}>
      <div className="transport-top">
        <span className="transport-label">{t("shell.transport")}</span>
        <span className="transport-state">
          <span className="status-dot" />
          {stateLabel}
        </span>
        <span className="transport-detail" title={detail}>
          {detail || t("progress.waiting")}
        </span>
        <span className="transport-percent">{String(Math.round(value)).padStart(3, "0")}%</span>
        {busy ? (
          <Button onPress={() => void cancel()} size="sm" variant="danger-soft">
            {t("progress.cancel")}
          </Button>
        ) : null}
        <Button
          onPress={() => void updateConfig({ logVisible: !config.logVisible })}
          size="sm"
          variant="ghost"
        >
          {config.logVisible ? t("log.hide") : t("log.show")}
          <span className="log-count">{logs.length}</span>
        </Button>
      </div>

      <ProgressBar aria-label={t("progress.title")} className="transport-progress" value={value}>
        <ProgressBar.Track>
          <ProgressBar.Fill />
        </ProgressBar.Track>
      </ProgressBar>

      {config.logVisible ? (
        <div className="log-panel">
          <div className="log-toolbar">
            <span>{t("log.title")}</span>
            <span>{logs.length.toString().padStart(4, "0")}</span>
            <span className="log-spacer" />
            <Button onPress={() => void copy()} size="sm" variant="ghost">
              {t("log.copy")}
            </Button>
            <Button onPress={clearLogs} size="sm" variant="ghost">
              {t("log.clear")}
            </Button>
          </div>
          <div className="log-view" ref={logRef}>
            {logs.length ? logs.join("\n") : t("log.empty")}
          </div>
        </div>
      ) : null}
    </footer>
  );
}
