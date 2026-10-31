import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Button,
  Checkbox,
  Input,
  Label,
  ListBox,
  Select,
  TextField,
  toast,
} from "@heroui/react";
import { Modal } from "./Modal";
import { api } from "../lib/api";
import { baseName, channelText, parentName } from "../lib/format";
import { pickFolder } from "../lib/files";
import type {
  ExtractChoice,
  ExtractFormat,
  OutputSuggestion,
} from "../lib/types";
import { useAppStore } from "../store";

export type ExtractPolicy = "ask" | "overwrite" | "skip" | "alternate";

interface ExtractDialogProps {
  onSubmit: (
    choices: ExtractChoice[],
    outputDir: string,
    policy: ExtractPolicy,
  ) => void;
}

const AUDIO_FORMATS: ExtractFormat[] = [
  "copy",
  "mp3",
  "m4a",
  "flac",
  "wav",
  "ac3",
  "opus",
];

const SUBTITLE_FORMATS: ExtractFormat[] = ["copy", "srt", "vtt"];

function isBitmapSubtitle(codec: string): boolean {
  return [
    "hdmv_pgs_subtitle",
    "dvd_subtitle",
    "dvb_subtitle",
    "xsub",
    "dvb_teletext",
  ].includes(codec.toLowerCase());
}

export function ExtractDialog({ onSubmit }: ExtractDialogProps) {
  const { t } = useTranslation();
  const target = useAppStore((state) => state.extractTarget);
  const setTarget = useAppStore((state) => state.setExtractTarget);
  const config = useAppStore((state) => state.config);
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [formats, setFormats] = useState<Record<number, ExtractFormat>>({});
  const [outputDir, setOutputDir] = useState("");
  const [policy, setPolicy] = useState<ExtractPolicy>("ask");
  const [outputs, setOutputs] = useState<OutputSuggestion[]>([]);

  const streams = useMemo(
    () =>
      target?.media.streams.filter(
        (stream) => stream.type === "audio" || stream.type === "subtitle",
      ) ?? [],
    [target],
  );

  useEffect(() => {
    if (!target) return;
    setSelected(new Set(target.preselect));
    setFormats(
      Object.fromEntries(
        target.media.streams
          .filter((stream) => stream.type === "audio" || stream.type === "subtitle")
          .map((stream) => [stream.index, "copy" as ExtractFormat]),
      ),
    );
    setPolicy("ask");
    setOutputDir(config.outputDir || parentName(target.path));
  }, [target, config.outputDir]);

  const selectedKey = useMemo(
    () => [...selected].sort((a, b) => a - b).join(","),
    [selected],
  );
  const formatsKey = useMemo(
    () =>
      [...selected]
        .sort((a, b) => a - b)
        .map((index) => `${index}:${formats[index] ?? "copy"}`)
        .join(","),
    [selected, formats],
  );

  useEffect(() => {
    if (!target || selected.size === 0) {
      setOutputs([]);
      return;
    }
    let cancelled = false;
    api
      .suggestOutputs({
        kind: "extract",
        path: target.path,
        outputDir,
        items: [...selected]
          .sort((a, b) => a - b)
          .map((streamIndex) => ({
            streamIndex,
            format: formats[streamIndex] ?? "copy",
          })),
      })
      .then((result) => {
        if (!cancelled) setOutputs(result);
      })
      .catch(() => {
        if (!cancelled) setOutputs([]);
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [target?.path, outputDir, selectedKey, formatsKey]);

  const outputByIndex = useMemo(() => {
    const map = new Map<number, string>();
    outputs.forEach((suggestion) => {
      if (suggestion.streamIndex !== null) {
        map.set(suggestion.streamIndex, suggestion.path);
      }
    });
    return map;
  }, [outputs]);

  if (!target) return null;

  const toggle = (index: number, checked: boolean) => {
    setSelected((current) => {
      const next = new Set(current);
      if (checked) next.add(index);
      else next.delete(index);
      return next;
    });
  };

  const selectType = (kind: "audio" | "subtitle") => {
    setSelected(
      new Set(
        streams
          .filter((stream) => stream.type === kind)
          .map((stream) => stream.index),
      ),
    );
  };

  const browse = async () => {
    const folder = await pickFolder(outputDir);
    if (folder) setOutputDir(folder);
  };

  const submit = () => {
    if (selected.size === 0) {
      toast.danger(t("extractDialog.noSelection"));
      return;
    }
    onSubmit(
      [...selected]
        .sort((a, b) => a - b)
        .map((streamIndex) => ({
          streamIndex,
          format: formats[streamIndex] ?? "copy",
        })),
      outputDir,
      policy,
    );
  };

  const formatLabels: Record<ExtractFormat, string> = {
    copy: t("extract.formatOriginal"),
    mp3: t("extract.formatMp3"),
    m4a: t("extract.formatM4a"),
    flac: t("extract.formatFlac"),
    wav: t("extract.formatWav"),
    ac3: t("extract.formatAc3"),
    opus: t("extract.formatOpus"),
    srt: t("extract.formatSrt"),
    vtt: t("extract.formatVtt"),
  };

  const formatsFor = (stream: { type: string; codec: string }): ExtractFormat[] => {
    if (stream.type === "audio") return AUDIO_FORMATS;
    if (isBitmapSubtitle(stream.codec)) return ["copy"];
    return SUBTITLE_FORMATS;
  };

  return (
    <Modal
      className="extract-dialog"
      description={t("extractDialog.info")}
      footer={
        <>
          <span className="dialog-count">
            {t("extractDialog.selected", { count: selected.size })}
          </span>
          <Button onPress={() => setTarget(null)} variant="tertiary">
            {t("actions.cancel")}
          </Button>
          <Button onPress={submit} variant="primary">
            {t("extractDialog.start")}
          </Button>
        </>
      }
      onOpenChange={(open) => {
        if (!open) setTarget(null);
      }}
      open={Boolean(target)}
      size="lg"
      title={t("extractDialog.title")}
    >
      <div className="dialog-tools">
        <Button onPress={() => selectType("audio")} size="sm" variant="tertiary">
          {t("extractDialog.selectAllAudio")}
        </Button>
        <Button onPress={() => selectType("subtitle")} size="sm" variant="tertiary">
          {t("extractDialog.selectAllSubs")}
        </Button>
        <Button onPress={() => setSelected(new Set())} size="sm" variant="tertiary">
          {t("extractDialog.selectNone")}
        </Button>
      </div>

      <div className="table-wrap">
        <table className="data-table">
          <thead>
            <tr>
              <th aria-label="select" />
              <th>{t("tracks.type")}</th>
              <th>{t("tracks.index")}</th>
              <th>{t("tracks.codec")}</th>
              <th>{t("tracks.language")}</th>
              <th>{t("tracks.channels")}</th>
              <th>{t("tracks.titleCol")}</th>
              <th>{t("extract.format")}</th>
              <th>{t("extractDialog.outputDir")}</th>
            </tr>
          </thead>
          <tbody>
            {streams.map((stream) => {
              const output = outputByIndex.get(stream.index);
              return (
                <tr key={stream.index}>
                  <td className="check-cell">
                    <Checkbox
                      aria-label={`#${stream.index}`}
                      isSelected={selected.has(stream.index)}
                      onChange={(checked) => toggle(stream.index, checked)}
                      variant="secondary"
                    >
                      <Checkbox.Content>
                        <Checkbox.Control>
                          <Checkbox.Indicator />
                        </Checkbox.Control>
                      </Checkbox.Content>
                    </Checkbox>
                  </td>
                  <td>
                    {stream.type === "audio" ? t("tracks.audio") : t("tracks.subtitle")}
                  </td>
                  <td className="mono-cell">{stream.index}</td>
                  <td>{stream.codec.toUpperCase()}</td>
                  <td>
                    {t(`languages.${stream.language}`, {
                      defaultValue: stream.language.toUpperCase(),
                    })}
                  </td>
                  <td>{stream.type === "audio" ? channelText(stream) : "—"}</td>
                  <td className="title-cell" title={stream.title || undefined}>
                    {stream.title || "—"}
                  </td>
                  <td className="format-cell">
                    <select
                      aria-label={`${t("extract.format")} #${stream.index}`}
                      className="format-select"
                      value={formats[stream.index] ?? "copy"}
                      onChange={(event) =>
                        setFormats((current) => ({
                          ...current,
                          [stream.index]: event.target.value as ExtractFormat,
                        }))
                      }
                    >
                      {formatsFor(stream).map((format) => (
                        <option key={format} value={format}>
                          {formatLabels[format]}
                        </option>
                      ))}
                    </select>
                  </td>
                  <td className="output-cell">
                    <span title={output}>{output ? baseName(output) : "—"}</span>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <p className="notice">{t("extract.conversionHint")}</p>

      <div className="extract-settings">
        <div className="path-row">
          <TextField
            className="path-field"
            fullWidth
            name="extract-output"
            value={outputDir}
            onChange={setOutputDir}
          >
            <Label>{t("extractDialog.outputDir")}</Label>
            <Input />
          </TextField>
          <Button onPress={() => void browse()} variant="tertiary">
            {t("actions.browse")}
          </Button>
        </div>
        <Select
          value={policy}
          onChange={(key) => {
            if (key) setPolicy(key as ExtractPolicy);
          }}
        >
          <Label>{t("extractDialog.ifExists")}</Label>
          <Select.Trigger>
            <Select.Value />
            <Select.Indicator />
          </Select.Trigger>
          <Select.Popover>
            <ListBox>
              <ListBox.Item id="ask" textValue={t("extractDialog.ask")}>
                {t("extractDialog.ask")}
                <ListBox.ItemIndicator />
              </ListBox.Item>
              <ListBox.Item id="overwrite" textValue={t("extractDialog.overwrite")}>
                {t("extractDialog.overwrite")}
                <ListBox.ItemIndicator />
              </ListBox.Item>
              <ListBox.Item id="skip" textValue={t("extractDialog.skip")}>
                {t("extractDialog.skip")}
                <ListBox.ItemIndicator />
              </ListBox.Item>
              <ListBox.Item id="alternate" textValue={t("extractDialog.alternate")}>
                {t("extractDialog.alternate")}
                <ListBox.ItemIndicator />
              </ListBox.Item>
            </ListBox>
          </Select.Popover>
        </Select>
      </div>
      <p className="notice">{t("extractDialog.warning")}</p>
    </Modal>
  );
}
