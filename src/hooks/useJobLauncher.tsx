import { useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Button, toast } from "@heroui/react";
import { Modal } from "../components/Modal";
import { api } from "../lib/api";
import { errorDetail, errorText } from "../lib/errors";
import { baseName } from "../lib/format";
import type { JobRequest, OutputSuggestion } from "../lib/types";
import { useAppStore } from "../store";

type BuildRequest = (
  suggestions: OutputSuggestion[],
  useAlternate: boolean,
  replace: boolean,
) => JobRequest;

interface PendingCollision {
  suggestions: OutputSuggestion[];
  build: BuildRequest;
}

export function useJobLauncher() {
  const { t } = useTranslation();
  const startJob = useAppStore((state) => state.startJob);
  const appendLog = useAppStore((state) => state.appendLog);
  const [pending, setPending] = useState<PendingCollision | null>(null);
  const [launching, setLaunching] = useState(false);

  const runRequest = async (request: JobRequest) => {
    setLaunching(true);
    try {
      await startJob(request);
      setPending(null);
      return true;
    } catch (error) {
      toast.danger(errorText(error, t));
      const detail = errorDetail(error);
      if (detail) appendLog(detail);
      return false;
    } finally {
      setLaunching(false);
    }
  };

  const launch = async (suggestions: OutputSuggestion[], build: BuildRequest) => {
    if (suggestions.some((suggestion) => suggestion.exists)) {
      setPending({ suggestions, build });
      return;
    }
    await runRequest(build(suggestions, false, false));
  };

  const launchWithRequest = async (
    request: Parameters<typeof api.suggestOutputs>[0],
    build: BuildRequest,
  ) => {
    setLaunching(true);
    try {
      const suggestions = await api.suggestOutputs(request);
      setLaunching(false);
      await launch(suggestions, build);
    } catch (error) {
      setLaunching(false);
      toast.danger(errorText(error, t));
    }
  };

  const collisionDialog: ReactNode = (
    <Modal
      className="collision-dialog"
      description={t("collision.message")}
      footer={
        <>
          <Button onPress={() => setPending(null)} variant="tertiary">
            {t("collision.cancel")}
          </Button>
          <Button
            isDisabled={launching}
            onPress={() =>
              pending && void runRequest(pending.build(pending.suggestions, true, false))
            }
            variant="outline"
          >
            {t("collision.alternate")}
          </Button>
          <Button
            isDisabled={launching}
            onPress={() =>
              pending && void runRequest(pending.build(pending.suggestions, false, true))
            }
            variant="danger"
          >
            {t("collision.replace")}
          </Button>
        </>
      }
      onOpenChange={(open) => {
        if (!open) setPending(null);
      }}
      open={Boolean(pending)}
      size="sm"
      title={t("collision.title")}
    >
      <div className="collision-list">
        {pending?.suggestions
          .filter((suggestion) => suggestion.exists)
          .map((suggestion) => (
            <div key={suggestion.path}>
              <strong>{baseName(suggestion.path)}</strong>
              <span>{suggestion.path}</span>
            </div>
          ))}
      </div>
    </Modal>
  );

  return { launch, launchWithRequest, runRequest, launching, collisionDialog };
}
