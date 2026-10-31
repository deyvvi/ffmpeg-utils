import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Spinner, Toast, toast } from "@heroui/react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Masthead } from "./components/Masthead";
import { ModeBar } from "./components/ModeBar";
import { Workspace } from "./components/Workspace";
import { Transport } from "./components/Transport";
import { SettingsDrawer } from "./components/SettingsDrawer";
import { ExtractDialog, type ExtractPolicy } from "./components/ExtractDialog";
import { PreviewDialog } from "./components/PreviewDialog";
import { ResultDialog } from "./components/ResultDialog";
import { useJobLauncher } from "./hooks/useJobLauncher";
import { api, onJobFinished, onJobLog, onJobProgress } from "./lib/api";
import { errorText } from "./lib/errors";
import { applyAppearance, resolveTheme, watchSystemTheme } from "./lib/theme";
import type { ExtractChoice, ExtractItemRequest } from "./lib/types";
import { useAppStore } from "./store";

function App() {
  const { t } = useTranslation();
  const ready = useAppStore((state) => state.ready);
  const config = useAppStore((state) => state.config);
  const result = useAppStore((state) => state.result);
  const extractTarget = useAppStore((state) => state.extractTarget);
  const init = useAppStore((state) => state.init);
  const loadFile = useAppStore((state) => state.loadFile);
  const setTab = useAppStore((state) => state.setTab);
  const setExtractTarget = useAppStore((state) => state.setExtractTarget);
  const appendLog = useAppStore((state) => state.appendLog);
  const handleProgress = useAppStore((state) => state.handleProgress);
  const handleFinished = useAppStore((state) => state.handleFinished);
  const dismissResult = useAppStore((state) => state.dismissResult);
  const [dragging, setDragging] = useState(false);
  const { launch, launchWithRequest, runRequest, collisionDialog } = useJobLauncher();

  useEffect(() => {
    void init();
  }, [init]);

  useEffect(() => {
    let unsubscribers: Array<() => void> = [];
    void Promise.all([
      onJobLog((event) => appendLog(event.line)),
      onJobProgress((event) => handleProgress(event)),
      onJobFinished((event) => handleFinished(event)),
    ]).then((values) => {
      unsubscribers = values;
    });
    return () => unsubscribers.forEach((unsubscribe) => unsubscribe());
  }, [appendLog, handleProgress, handleFinished]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === "enter" || payload.type === "over") {
          setDragging(true);
          return;
        }
        if (payload.type === "leave") {
          setDragging(false);
          return;
        }
        setDragging(false);
        if (payload.type === "drop") {
          const paths = payload.paths.filter((path) => path.length > 0);
          if (paths.length > 1) {
            toast.info(t("drop.oneOnly"));
          }
          const path = paths[0];
          if (path) {
            void loadFile(path).catch((error) => toast.danger(errorText(error, t)));
          }
        }
      })
      .then((value) => {
        unlisten = value;
      });
    return () => unlisten?.();
  }, [loadFile, t]);

  useEffect(() => {
    if (config.theme !== "system") return;
    return watchSystemTheme(() => applyAppearance(config));
  }, [config]);

  useEffect(() => {
    void api.setWindowTheme(resolveTheme(config.theme)).catch(() => {});
  }, [config.theme]);

  const submitExtract = async (
    choices: ExtractChoice[],
    outputDir: string,
    policy: ExtractPolicy,
  ) => {
    if (!extractTarget) return;
    const target = extractTarget;
    setExtractTarget(null);
    try {
      let suggestions = await api.suggestOutputs({
        kind: "extract",
        path: target.path,
        outputDir,
        items: choices,
      });
      if (!suggestions.length) {
        toast.info(t("common.none"));
        return;
      }
      if (policy === "skip") {
        suggestions = suggestions.filter((suggestion) => !suggestion.exists);
        if (!suggestions.length) {
          toast.info(t("extractDialog.noSelection"));
          return;
        }
      }
      const formatByIndex = new Map(
        choices.map((choice) => [choice.streamIndex, choice.format]),
      );
      if (policy === "ask") {
        await launch(suggestions, (items, useAlternate, replace) => ({
          kind: "extract",
          path: target.path,
          items: items.map((suggestion) => ({
            streamIndex: suggestion.streamIndex ?? 0,
            output: useAlternate ? suggestion.alternate : suggestion.path,
            replace: replace && suggestion.exists,
            format: formatByIndex.get(suggestion.streamIndex ?? 0) ?? "copy",
          })),
        }));
        return;
      }
      const useAlternate = policy === "alternate";
      const replace = policy === "overwrite";
      const items: ExtractItemRequest[] = suggestions.map((suggestion) => ({
        streamIndex: suggestion.streamIndex ?? 0,
        output: useAlternate ? suggestion.alternate : suggestion.path,
        replace: replace && suggestion.exists,
        format: formatByIndex.get(suggestion.streamIndex ?? 0) ?? "copy",
      }));
      await runRequest({ kind: "extract", path: target.path, items });
    } catch (error) {
      toast.danger(errorText(error, t));
    }
  };

  const singleFromResult = (language: "eng" | "spa") => {
    const output = result?.outputs[0];
    if (!output) return;
    dismissResult();
    void launchWithRequest(
      { kind: "singleAudio", path: output, language },
      (suggestions, useAlternate, replace) => ({
        kind: "singleAudio",
        path: output,
        output: useAlternate ? suggestions[0].alternate : suggestions[0].path,
        language,
        audioIndex: null,
        replace,
      }),
    );
  };

  const extractFromResult = async () => {
    const output = result?.outputs[0];
    if (!output) return;
    try {
      const probed = await api.probeFile(output);
      dismissResult();
      setExtractTarget({ media: probed, path: output, preselect: [] });
    } catch (error) {
      toast.danger(errorText(error, t));
    }
  };

  const trimFromResult = async () => {
    const output = result?.outputs[0];
    if (!output) return;
    try {
      await loadFile(output);
      dismissResult();
      setTab("trim");
    } catch (error) {
      toast.danger(errorText(error, t));
    }
  };

  return (
    <div className="workbench">
      <Masthead />
      <ModeBar />

      {!ready ? (
        <main className="boot-screen">
          <Spinner color="current" size="sm" />
          <span>{t("progress.preparing")}</span>
        </main>
      ) : (
        <Workspace />
      )}

      <Transport />

      {dragging ? (
        <div className="drop-overlay" aria-live="polite">
          <div className="drop-overlay-panel">
            <span className="eyebrow">{t("drop.target")}</span>
            <strong>{t("drop.dragging")}</strong>
          </div>
        </div>
      ) : null}

      <SettingsDrawer />
      <ExtractDialog
        onSubmit={(choices, outputDir, policy) =>
          void submitExtract(choices, outputDir, policy)
        }
      />
      <PreviewDialog />
      <ResultDialog
        onExtract={() => void extractFromResult()}
        onSingle={singleFromResult}
        onTrim={() => void trimFromResult()}
      />
      {collisionDialog}
      <Toast.Provider placement="bottom end" />
    </div>
  );
}

export default App;
