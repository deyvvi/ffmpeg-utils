import { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "@heroui/react";
import { pickVideo } from "../lib/files";
import { errorText } from "../lib/errors";
import { useAppStore } from "../store";

export function DropZone() {
  const { t } = useTranslation();
  const loadFile = useAppStore((state) => state.loadFile);
  const [dragging, setDragging] = useState(false);

  const choose = async () => {
    try {
      const path = await pickVideo();
      if (path) await loadFile(path);
    } catch (error) {
      toast.danger(errorText(error, t));
    }
  };

  return (
    <section className="pane source-pane drop-pane">
      <div className="pane-head">
        <span>{t("shell.source")}</span>
        <span>01</span>
      </div>
      <button
        className="drop-slab"
        data-dragging={dragging}
        onClick={() => void choose()}
        onDragEnter={() => setDragging(true)}
        onDragLeave={() => setDragging(false)}
        onDrop={() => setDragging(false)}
        type="button"
      >
        <span className="drop-number">01</span>
        <span className="drop-corners" aria-hidden="true">
          <span />
          <span />
          <span />
          <span />
        </span>
        <span className="drop-copy">
          <span className="eyebrow">{t("dropsession.label")}</span>
          <strong>{dragging ? t("drop.dragging") : t("drop.title")}</strong>
          <span>{t("drop.hint")}</span>
        </span>
        <span className="drop-footer">
          <span className="drop-cta">{t("drop.button")}</span>
          <span className="drop-meta">{t("empty.formats")}</span>
        </span>
      </button>
    </section>
  );
}
