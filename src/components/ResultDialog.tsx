import { useTranslation } from "react-i18next";
import { Button, toast } from "@heroui/react";
import { Modal } from "./Modal";
import { checkText } from "../lib/checks";
import { errorText } from "../lib/errors";
import { baseName } from "../lib/format";
import { reveal } from "../lib/files";
import { useAppStore } from "../store";

interface ResultDialogProps {
  onSingle: (language: "eng" | "spa") => void;
  onExtract: () => void;
  onTrim: () => void;
}

export function ResultDialog({ onSingle, onExtract, onTrim }: ResultDialogProps) {
  const { t } = useTranslation();
  const result = useAppStore((state) => state.result);
  const dismiss = useAppStore((state) => state.dismissResult);

  if (!result) return null;

  const cancelled = result.code === "cancelled";
  const warnings = result.checks.some((check) => !check.ok);
  const title = result.ok
    ? warnings
      ? t("result.completedWithWarnings")
      : t("result.completed")
    : cancelled
      ? t("result.cancelled")
      : t("result.failed");
  const message = result.ok
    ? t("result.completedHint")
    : cancelled
      ? t("result.cancelled")
      : errorText(result, t);
  const showSingle = result.kind === "normalize" || result.kind === "trim";

  const openOutput = async (path: string) => {
    try {
      await reveal(path);
    } catch (error) {
      toast.danger(String(error));
    }
  };

  return (
    <Modal
      className="report-dialog"
      description={t("result.elapsed", { seconds: result.elapsed.toFixed(1) })}
      footer={
        <>
          <Button onPress={dismiss} variant="tertiary">
            {t("result.close")}
          </Button>
          {result.outputs[0] ? (
            <Button onPress={() => void openOutput(result.outputs[0])} variant="outline">
              {t("result.openFolder")}
            </Button>
          ) : null}
        </>
      }
      onOpenChange={(open) => {
        if (!open) dismiss();
      }}
      open
      title={t("result.session")}
    >
      <div className="report-band" data-tone={result.ok ? "ok" : cancelled ? "idle" : "bad"}>
        <span className="report-mark">{result.ok ? "OK" : cancelled ? "—" : "!"}</span>
        <div>
          <strong>{title}</strong>
          <p>{message}</p>
          {!result.ok && result.detail ? (
            <pre className="report-detail">{result.detail}</pre>
          ) : null}
        </div>
      </div>

      {showSingle ? (
        <div className="follow-actions">
          <Button
            onPress={() => {
              dismiss();
              onSingle("eng");
            }}
            variant="tertiary"
          >
            {t("result.singleEnglish")}
          </Button>
          <Button
            onPress={() => {
              dismiss();
              onSingle("spa");
            }}
            variant="tertiary"
          >
            {t("result.singleSpanish")}
          </Button>
        </div>
      ) : null}

      {result.kind !== "extract" ? (
        <div className="follow-actions">
          <Button
            onPress={() => {
              dismiss();
              onExtract();
            }}
            variant="tertiary"
          >
            {t("result.extractTracks")}
          </Button>
          {result.kind === "normalize" ? (
            <Button
              onPress={() => {
                dismiss();
                onTrim();
              }}
              variant="tertiary"
            >
              {t("result.trimResult")}
            </Button>
          ) : null}
        </div>
      ) : null}

      {result.checks.length ? (
        <section className="report-section">
          <div className="report-heading">
            <span>{t("result.checks")}</span>
            <span>{String(result.checks.length).padStart(2, "0")}</span>
          </div>
          <div className="check-list">
            {result.checks.map((check, index) => (
              <div className="check-row" data-ok={check.ok} key={`${check.code}-${index}`}>
                <span className="check-mark">{check.ok ? "OK" : "X"}</span>
                <span>{checkText(check, t)}</span>
              </div>
            ))}
          </div>
        </section>
      ) : null}

      {result.outputs.length ? (
        <section className="report-section">
          <div className="report-heading">
            <span>{t("result.outputs")}</span>
            <span>{String(result.outputs.length).padStart(2, "0")}</span>
          </div>
          <div className="output-list">
            {result.outputs.map((output) => (
              <button
                className="output-row"
                key={output}
                onClick={() => void openOutput(output)}
                title={output}
                type="button"
              >
                <span>{baseName(output)}</span>
                <small>{output}</small>
              </button>
            ))}
          </div>
        </section>
      ) : null}
    </Modal>
  );
}
