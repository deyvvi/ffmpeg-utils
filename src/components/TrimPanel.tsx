import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Button,
  Input,
  Label,
  ListBox,
  Select,
  Slider,
  Spinner,
  TextField,
  toast,
} from "@heroui/react";
import { useJobLauncher } from "../hooks/useJobLauncher";
import { useOutputPreview } from "../hooks/useOutputPreview";
import { api } from "../lib/api";
import { errorText } from "../lib/errors";
import { baseName, formatTimecode, parseTimecode } from "../lib/format";
import type { JobRequest } from "../lib/types";
import { useAppStore } from "../store";

type TrimMode = "fast" | "precise" | "software";

export function TrimPanel() {
  const { t } = useTranslation();
  const media = useAppStore((state) => state.media);
  const amf = useAppStore((state) => state.amf);
  const tools = useAppStore((state) => state.tools);
  const busy = useAppStore((state) => state.busy);
  const setPreview = useAppStore((state) => state.setPreview);
  const { launchWithRequest, launching, collisionDialog } = useJobLauncher();

  const [startSeconds, setStartSeconds] = useState(0);
  const [endSeconds, setEndSeconds] = useState(0);
  const [startText, setStartText] = useState("00:00:00.000");
  const [endText, setEndText] = useState("00:00:00.000");
  const [mode, setMode] = useState<TrimMode>("fast");

  useEffect(() => {
    if (!media) return;
    setStartSeconds(0);
    setEndSeconds(media.duration);
    setStartText("00:00:00.000");
    setEndText(formatTimecode(media.duration));
    setMode("fast");
  }, [media]);

  const valid = Boolean(
    media &&
      endSeconds > startSeconds &&
      startSeconds >= 0 &&
      endSeconds <= media.duration + 0.001,
  );
  const duration = Math.max(0, endSeconds - startSeconds);

  const suggestions = useOutputPreview(
    media && valid
      ? { kind: "trim", path: media.path, start: startSeconds, end: endSeconds }
      : null,
  );
  const output = suggestions[0];

  const modeOptions = useMemo(() => {
    const options = [{ value: "fast", label: t("trim.fast") }];
    if (amf?.available) options.push({ value: "precise", label: t("trim.precise") });
    options.push({ value: "software", label: t("trim.software") });
    return options;
  }, [amf?.available, t]);

  if (!media) return null;
  const disabled = busy || launching || !tools?.found || !valid;

  const updateText = (kind: "start" | "end") => {
    const parsed = kind === "start" ? parseTimecode(startText) : parseTimecode(endText);
    if (parsed === null) {
      toast.danger(t("trim.invalid"));
      return;
    }
    if (kind === "start") {
      if (parsed >= endSeconds || parsed < 0) {
        toast.danger(t("trim.invalid"));
        return;
      }
      setStartSeconds(parsed);
    } else {
      if (parsed <= startSeconds || parsed > media.duration + 0.001) {
        toast.danger(t("trim.invalid"));
        return;
      }
      setEndSeconds(parsed);
    }
  };

  const buildRequest = (
    allSuggestions: typeof suggestions,
    useAlternate: boolean,
    replace: boolean,
  ): JobRequest => ({
    kind: "trim",
    path: media.path,
    output: useAlternate
      ? allSuggestions[0]?.alternate ?? output?.alternate ?? ""
      : allSuggestions[0]?.path ?? output?.path ?? "",
    start: startSeconds,
    end: endSeconds,
    mode,
    replace,
  });

  const start = () => {
    if (!valid) {
      toast.danger(t("trim.invalid"));
      return;
    }
    void launchWithRequest(
      { kind: "trim", path: media.path, start: startSeconds, end: endSeconds },
      buildRequest,
    );
  };

  const preview = async () => {
    if (!valid) {
      toast.danger(t("trim.invalid"));
      return;
    }
    try {
      const nextSuggestions = await api.suggestOutputs({
        kind: "trim",
        path: media.path,
        start: startSeconds,
        end: endSeconds,
      });
      const plan = await api.previewPlan(buildRequest(nextSuggestions, false, false));
      setPreview(plan);
    } catch (error) {
      toast.danger(errorText(error, t));
    }
  };

  const onSliderChange = (value: number | number[]) => {
    if (!Array.isArray(value)) return;
    const [nextStart, nextEnd] = value;
    const safeStart = Math.max(0, Math.min(nextStart, nextEnd - 0.1));
    const safeEnd = Math.min(media.duration, nextEnd);
    setStartSeconds(safeStart);
    setEndSeconds(safeEnd);
    setStartText(formatTimecode(safeStart));
    setEndText(formatTimecode(safeEnd));
  };

  return (
    <section className="pane operation-pane operation-pane--fixed">
      <div className="operation-scroll">
      <div className="pane-head">
        <span>{t("shell.operation")}</span>
        <span>02</span>
      </div>

      <header className="op-head">
        <span className="eyebrow">{t("shell.sessionTrim")}</span>
        <h1 className="op-title">{t("header.trimTitle")}</h1>
        <p className="op-sub">{t("header.trimSubtitle")}</p>
      </header>

      <section className="op-section">
        <div className="op-section-head">
          <span>
            <b>01</b> / {t("trim.rangeLabel")}
          </span>
          <span>{t("trim.duration")} {formatTimecode(duration)}</span>
        </div>

        <div className="time-grid">
          <TextField
            className="time-field"
            isDisabled={busy}
            name="trim-start"
            value={startText}
            onChange={setStartText}
          >
            <Label>{t("trim.start")}</Label>
            <Input
              className="time-input"
              onBlur={() => updateText("start")}
              spellCheck={false}
            />
          </TextField>
          <TextField
            className="time-field"
            isDisabled={busy}
            name="trim-end"
            value={endText}
            onChange={setEndText}
          >
            <Label>{t("trim.end")}</Label>
            <Input
              className="time-input"
              onBlur={() => updateText("end")}
              spellCheck={false}
            />
          </TextField>
          <Select
            className="mode-select"
            value={mode}
            onChange={(key) => {
              if (key) setMode(key as TrimMode);
            }}
          >
            <Label>{t("trim.mode")}</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {modeOptions.map((option) => (
                  <ListBox.Item
                    id={option.value}
                    key={option.value}
                    textValue={option.label}
                  >
                    {option.label}
                    <ListBox.ItemIndicator />
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>
        </div>

        <Slider
          aria-label={t("trim.rangeLabel")}
          className="trim-slider"
          maxValue={Math.max(1, media.duration)}
          minValue={0}
          onChange={onSliderChange}
          step={0.1}
          value={[startSeconds, endSeconds]}
        >
          <Slider.Track>
            {({ state }) => (
              <>
                <Slider.Fill />
                {state.values.map((_, index) => (
                  <Slider.Thumb index={index} key={index} />
                ))}
              </>
            )}
          </Slider.Track>
        </Slider>
        <div className="range-labels">
          <span>{formatTimecode(startSeconds)}</span>
          <span>{formatTimecode(endSeconds)}</span>
        </div>

        <p className="notice">
          {mode === "fast"
            ? t("trim.fastHint")
            : mode === "precise"
              ? t("trim.preciseHint")
              : t("trim.softwareHint")}
        </p>
      </section>

      <section className="op-section">
        <div className="op-section-head">
          <span>
            <b>02</b> / {t("shell.destination")}
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
          {t("trim.startAction")}
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

