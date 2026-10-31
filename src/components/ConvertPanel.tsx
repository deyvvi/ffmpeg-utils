import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button, Input, Label, Spinner, TextField, toast } from "@heroui/react";
import { useOutputPreview } from "../hooks/useOutputPreview";
import { useJobLauncher } from "../hooks/useJobLauncher";
import { api } from "../lib/api";
import { errorText } from "../lib/errors";
import { baseName } from "../lib/format";
import { RESOLUTION_PRESETS, validateResolution } from "../lib/resolutions";
import type { JobRequest } from "../lib/types";
import { useAppStore } from "../store";

export function ConvertPanel() {
  const { t } = useTranslation();
  const media = useAppStore((state) => state.media);
  const encoders = useAppStore((state) => state.encoders);
  const tools = useAppStore((state) => state.tools);
  const busy = useAppStore((state) => state.busy);
  const targetWidth = useAppStore((state) => state.targetWidth);
  const targetHeight = useAppStore((state) => state.targetHeight);
  const setTargetSize = useAppStore((state) => state.setTargetSize);
  const setPreview = useAppStore((state) => state.setPreview);
  const { launchWithRequest, launching, collisionDialog } = useJobLauncher();

  const [videoMode, setVideoMode] = useState("");
  const [customMode, setCustomMode] = useState(false);
  const [customWidth, setCustomWidth] = useState(String(targetWidth));
  const [customHeight, setCustomHeight] = useState(String(targetHeight));

  const video = media?.streams.find((stream) => stream.type === "video");
  const sourceWidth = video?.width ?? 0;
  const sourceHeight = video?.height ?? 0;
  const sourceRatio = sourceWidth && sourceHeight ? sourceWidth / sourceHeight : 16 / 9;
  const targetRatio = targetWidth / targetHeight;
  const outputWidth =
    sourceRatio >= targetRatio
      ? targetWidth
      : Math.round((targetHeight * sourceRatio) / 2) * 2;
  const outputHeight =
    sourceRatio >= targetRatio
      ? Math.round((targetWidth / sourceRatio) / 2) * 2
      : targetHeight;
  const padX = Math.round((targetWidth - outputWidth) / 2);
  const padY = Math.round((targetHeight - outputHeight) / 2);
  const preset = RESOLUTION_PRESETS.find(
    (item) => item.width === targetWidth && item.height === targetHeight,
  );
  const customError = validateResolution(
    Number(customWidth),
    Number(customHeight),
  );
  const copyAllowed = Boolean(
    video &&
      video.width === targetWidth &&
      video.height === targetHeight &&
      video.rotation === 0 &&
      (!video.sampleAspectRatio || video.sampleAspectRatio === "1:1"),
  );

  const availableEncoders = useMemo(
    () => encoders.filter((encoder) => encoder.available),
    [encoders],
  );

  useEffect(() => {
    setCustomWidth(String(targetWidth));
    setCustomHeight(String(targetHeight));
  }, [targetWidth, targetHeight]);

  useEffect(() => {
    if (!encoders.length) return;
    const ids = availableEncoders.map((encoder) => encoder.id);
    if (videoMode === "copy" && copyAllowed) return;
    if (ids.includes(videoMode)) return;
    const preferred =
      availableEncoders.find((encoder) => encoder.hardware)?.id ?? "libx264";
    setVideoMode(preferred);
  }, [availableEncoders, copyAllowed, encoders.length, videoMode]);

  const suggestions = useOutputPreview(
    media
      ? { kind: "normalize", path: media.path, width: targetWidth, height: targetHeight }
      : null,
  );
  const output = suggestions[0];

  if (!media) return null;

  const disabled =
    busy ||
    launching ||
    !tools?.found ||
    !videoMode ||
    (customMode && customError !== null);

  const currentEncoder = availableEncoders.find(
    (encoder) => encoder.id === videoMode,
  );

  const setCustomSize = (value: string, part: "width" | "height") => {
    if (part === "width") setCustomWidth(value);
    else setCustomHeight(value);
    const nextWidth = Number(part === "width" ? value : customWidth);
    const nextHeight = Number(part === "height" ? value : customHeight);
    if (validateResolution(nextWidth, nextHeight) === null) {
      setTargetSize(nextWidth, nextHeight);
    }
  };

  const buildRequest = (
    allSuggestions: typeof suggestions,
    useAlternate: boolean,
    replace: boolean,
  ): JobRequest => ({
    kind: "normalize",
    path: media.path,
    output: useAlternate
      ? allSuggestions[0]?.alternate ?? output?.alternate ?? ""
      : allSuggestions[0]?.path ?? output?.path ?? "",
    videoMode,
    width: targetWidth,
    height: targetHeight,
    replace,
  });

  const start = () => {
    void launchWithRequest(
      {
        kind: "normalize",
        path: media.path,
        width: targetWidth,
        height: targetHeight,
      },
      buildRequest,
    );
  };

  const preview = async () => {
    try {
      const nextSuggestions = await api.suggestOutputs({
        kind: "normalize",
        path: media.path,
        width: targetWidth,
        height: targetHeight,
      });
      const plan = await api.previewPlan(buildRequest(nextSuggestions, false, false));
      setPreview(plan);
    } catch (error) {
      toast.danger(errorText(error, t));
    }
  };

  return (
    <section className="pane operation-pane operation-pane--fixed">
      <div className="operation-scroll">
        <div className="pane-head">
          <span>{t("shell.operation")}</span>
          <span>02</span>
        </div>

        <header className="op-head">
          <span className="eyebrow">{t("shell.sessionConvert")}</span>
          <h1 className="op-title">{t("header.convertTitle")}</h1>
          <p className="op-sub">{t("header.convertSubtitle")}</p>
        </header>

        <section className="op-section">
          <div className="op-section-head">
            <span>
              <b>01</b> / {t("convert.resolution")}
            </span>
            <span>
              {targetWidth} × {targetHeight}
            </span>
          </div>

          <div className="resolution-presets">
            {RESOLUTION_PRESETS.map((item) => (
              <button
                aria-pressed={!customMode && preset?.id === item.id}
                className="resolution-preset"
                key={item.id}
                onClick={() => {
                  setCustomMode(false);
                  setTargetSize(item.width, item.height);
                }}
                type="button"
              >
                <span>{item.label}</span>
                <small>
                  {item.width} × {item.height}
                </small>
              </button>
            ))}
            <button
              aria-pressed={customMode}
              className="resolution-preset"
              onClick={() => setCustomMode(true)}
              type="button"
            >
              <span>{t("convert.custom")}</span>
              <small>{t("convert.customHint")}</small>
            </button>
          </div>

          {customMode ? (
            <div className="resolution-custom">
              <TextField
                name="target-width"
                type="number"
                value={customWidth}
                onChange={(value) => setCustomSize(value, "width")}
              >
                <Label>{t("convert.width")}</Label>
                <Input min={16} max={7680} type="number" />
              </TextField>
              <span className="resolution-times">×</span>
              <TextField
                name="target-height"
                type="number"
                value={customHeight}
                onChange={(value) => setCustomSize(value, "height")}
              >
                <Label>{t("convert.height")}</Label>
                <Input min={16} max={7680} type="number" />
              </TextField>
            </div>
          ) : null}
          {customMode && customError ? (
            <p className="notice">{t("convert.invalidResolution")}</p>
          ) : null}

          <div className="canvas-facts">
            <div className="fact">
              <span>{t("convert.sourceSize")}</span>
              <b>{sourceWidth && sourceHeight ? `${sourceWidth} × ${sourceHeight}` : "—"}</b>
            </div>
            <div className="fact">
              <span>{t("convert.padding")}</span>
              <b>{`X ${padX} / Y ${padY}`}</b>
            </div>
            <div className="fact">
              <span>{t("convert.content")}</span>
              <b>{`${outputWidth} × ${outputHeight}`}</b>
            </div>
            <div className="fact">
              <span>{t("convert.outputCanvas")}</span>
              <b>{`${targetWidth} × ${targetHeight}`}</b>
            </div>
          </div>
        </section>

        <section className="op-section">
          <div className="op-section-head">
            <span>
              <b>02</b> / {t("convert.encoderMode")}
            </span>
            <span>
              {videoMode === "copy"
                ? "STREAM COPY"
                : currentEncoder?.label.toUpperCase() ?? videoMode}
            </span>
          </div>
          <div className="choice-list">
            {copyAllowed ? (
              <button
                aria-pressed={videoMode === "copy"}
                className="choice"
                onClick={() => setVideoMode("copy")}
                type="button"
              >
                <span className="choice-mark" aria-hidden="true" />
                <span>
                  <span className="choice-title">{t("convert.keepVideo")}</span>
                  <span className="choice-desc">{t("convert.keepVideoHint")}</span>
                </span>
              </button>
            ) : null}
            {availableEncoders.map((encoder) => (
              <button
                aria-pressed={videoMode === encoder.id}
                className="choice"
                key={encoder.id}
                onClick={() => setVideoMode(encoder.id)}
                type="button"
              >
                <span className="choice-mark" aria-hidden="true" />
                <span>
                  <span className="choice-title">{encoder.label}</span>
                  <span className="choice-desc">
                    {encoder.hardware
                      ? t("convert.hardwareHint", { vendor: encoder.vendor })
                      : t("convert.softwareHint")}
                  </span>
                </span>
              </button>
            ))}
          </div>
          {!availableEncoders.some((encoder) => encoder.hardware) ? (
            <p className="notice">{t("convert.noHardware")}</p>
          ) : null}
        </section>

        <section className="op-section">
          <div className="op-section-head">
            <span>
              <b>03</b> / {t("shell.destination")}
            </span>
            <span>{output?.exists ? t("errors.exists") : t("shell.ready")}</span>
          </div>
          <div className="dest-line">
            <span className="dest-name" title={output?.path}>
              {output ? baseName(output.path) : "—"}
            </span>
            <span className="dest-path" title={output?.path}>
              {output?.path ?? ""}
            </span>
          </div>
        </section>
      </div>

      <div className="op-actions">
        <Button
          className="primary-action"
          isDisabled={disabled || !output}
          onPress={start}
          variant="primary"
        >
          {busy || launching ? <Spinner color="current" size="sm" /> : null}
          {t("convert.start")}
        </Button>
        <Button
          isDisabled={disabled || !output}
          onPress={() => void preview()}
          variant="outline"
        >
          {t("actions.preview")}
        </Button>
      </div>
      {collisionDialog}
    </section>
  );
}
