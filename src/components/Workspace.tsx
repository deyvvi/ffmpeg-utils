import { useTranslation } from "react-i18next";
import { useAppStore } from "../store";
import { DropZone } from "./DropZone";
import { OperationPanel } from "./OperationPanel";
import { SourcePanel } from "./SourcePanel";

function EmptyOperation() {
  const { t } = useTranslation();

  return (
    <section className="pane operation-pane empty-operation">
      <div className="pane-head">
        <span>{t("shell.operation")}</span>
        <span>02</span>
      </div>
      <div className="empty-operation-body">
        <span className="eyebrow">{t("empty.kicker")}</span>
        <h2>{t("file.notLoaded")}</h2>
        <p>{t("empty.intro")}</p>
        <ol className="step-list">
          <li>
            <span>01</span>
            <p>{t("empty.step1")}</p>
          </li>
          <li>
            <span>02</span>
            <p>{t("empty.step2")}</p>
          </li>
          <li>
            <span>03</span>
            <p>{t("empty.step3")}</p>
          </li>
        </ol>
        <div className="format-line">{t("empty.formats")}</div>
      </div>
    </section>
  );
}

export function Workspace() {
  const media = useAppStore((state) => state.media);

  if (!media) {
    return (
      <main className="workbench-body">
        <DropZone />
        <EmptyOperation />
      </main>
    );
  }

  return (
    <main className="workbench-body">
      <SourcePanel />
      <OperationPanel />
    </main>
  );
}
