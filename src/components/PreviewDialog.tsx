import { useTranslation } from "react-i18next";
import { Button, toast } from "@heroui/react";
import { Modal } from "./Modal";
import { useAppStore } from "../store";

export function PreviewDialog() {
  const { t } = useTranslation();
  const preview = useAppStore((state) => state.preview);
  const setPreview = useAppStore((state) => state.setPreview);
  const text = preview?.display.join("\n\n") ?? "";

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(t("actions.copy"));
    } catch (error) {
      toast.danger(String(error));
    }
  };

  return (
    <Modal
      className="preview-dialog"
      footer={
        <>
          <Button onPress={() => setPreview(null)} variant="tertiary">
            {t("preview.close")}
          </Button>
          <Button onPress={() => void copy()} variant="primary">
            {t("preview.copy")}
          </Button>
        </>
      }
      onOpenChange={(open) => {
        if (!open) setPreview(null);
      }}
      open={Boolean(preview)}
      title={t("preview.title")}
    >
      {preview?.notes.length ? (
        <div className="note-list">
          <span className="eyebrow">{t("preview.notes")}</span>
          {preview.notes.map((note, index) => (
            <p key={`${note}-${index}`}>{note}</p>
          ))}
        </div>
      ) : null}
      <span className="eyebrow">FFMPEG / COMMAND</span>
      <pre className="command-block">{text}</pre>
    </Modal>
  );
}
