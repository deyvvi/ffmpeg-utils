import { useTranslation } from "react-i18next";
import { Button, Spinner, toast } from "@heroui/react";
import { AppMark } from "../lib/icons";
import { openFolder } from "../lib/files";
import { parentName } from "../lib/format";
import { useAppStore } from "../store";

export function Masthead() {
  const { t } = useTranslation();
  const tools = useAppStore((state) => state.tools);
  const encoders = useAppStore((state) => state.encoders);
  const media = useAppStore((state) => state.media);
  const config = useAppStore((state) => state.config);
  const probing = useAppStore((state) => state.probing);
  const setSettingsOpen = useAppStore((state) => state.setSettingsOpen);

  const folder = config.outputDir || (media ? parentName(media.path) : "");
  const hardware = encoders.find((encoder) => encoder.hardware && encoder.available);
  const hardwareReady = encoders.length > 0;

  const openOutput = async () => {
    if (!folder) {
      toast.info(t("actions.openFolder"));
      return;
    }
    try {
      await openFolder(folder);
    } catch (error) {
      toast.danger(String(error));
    }
  };

  return (
    <header className="masthead">
      <div className="brand-block">
        <AppMark className="brand-mark" />
        <div className="brand-copy">
          <div className="brand-name">{t("app.name")}</div>
        </div>
      </div>

      <div className="masthead-tools">
        <div className="status-strip" aria-label={t("status.toolsHint")}>
          {probing ? (
            <span className="status-item">
              <Spinner color="current" size="sm" />
              {t("file.analyzing")}
            </span>
          ) : null}
          <span className="status-item" data-tone={tools?.found ? "ok" : "bad"}>
            <span className="status-dot" />
            {tools?.found ? t("status.ffmpeg") : t("status.ffmpegMissing")}
          </span>
          <span
            className="status-item"
            data-tone={!hardwareReady ? "idle" : hardware ? "ok" : "warn"}
          >
            <span className="status-dot" />
            {!hardwareReady
              ? t("status.checking")
              : hardware
                ? t("status.hardwareReady", { name: hardware.label })
                : t("status.softwareOnly")}
          </span>
        </div>

        <span className="masthead-divider" aria-hidden="true" />

        <Button
          className="shell-button"
          isDisabled={!folder}
          onPress={() => void openOutput()}
          size="sm"
          variant="tertiary"
        >
          {t("actions.openFolder")}
        </Button>
        <Button
          className="shell-button"
          onPress={() => setSettingsOpen(true)}
          size="sm"
          variant="secondary"
        >
          {t("nav.settings")}
        </Button>
      </div>
    </header>
  );
}
