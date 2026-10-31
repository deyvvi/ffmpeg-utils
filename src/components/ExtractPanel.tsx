import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button, Label, ListBox, Select, Spinner } from "@heroui/react";
import { useJobLauncher } from "../hooks/useJobLauncher";
import { useOutputPreview } from "../hooks/useOutputPreview";
import { baseName, channelText, isEnglish, isSpanish } from "../lib/format";
import type { JobRequest } from "../lib/types";
import { useAppStore } from "../store";

export function ExtractPanel() {
  const { t } = useTranslation();
  const media = useAppStore((state) => state.media);
  const tools = useAppStore((state) => state.tools);
  const busy = useAppStore((state) => state.busy);
  const setExtractTarget = useAppStore((state) => state.setExtractTarget);
  const { launchWithRequest, launching, collisionDialog } = useJobLauncher();

  const [language, setLanguage] = useState<"eng" | "spa">("eng");
  const [audioIndex, setAudioIndex] = useState<number | null>(null);

  const candidates = useMemo(() => {
    if (!media) return [];
    return media.streams.filter(
      (stream) =>
        stream.type === "audio" &&
        (language === "eng" ? isEnglish(stream.language) : isSpanish(stream.language)),
    );
  }, [media, language]);

  useEffect(() => {
    if (audioIndex !== null && candidates.some((stream) => stream.index === audioIndex)) {
      return;
    }
    setAudioIndex(candidates[0]?.index ?? null);
  }, [audioIndex, candidates]);

  const suggestions = useOutputPreview(
    media ? { kind: "singleAudio", path: media.path, language } : null,
  );
  const output = suggestions[0];
  const selected = candidates.find((stream) => stream.index === audioIndex) ?? null;

  if (!media) return null;
  const disabled =
    busy || launching || !tools?.found || candidates.length === 0 || audioIndex === null;

  const buildRequest = (
    allSuggestions: typeof suggestions,
    useAlternate: boolean,
    replace: boolean,
  ): JobRequest => ({
    kind: "singleAudio",
    path: media.path,
    output: useAlternate
      ? allSuggestions[0]?.alternate ?? output?.alternate ?? ""
      : allSuggestions[0]?.path ?? output?.path ?? "",
    language,
    audioIndex,
    replace,
  });

  const startSingle = () => {
    void launchWithRequest(
      { kind: "singleAudio", path: media.path, language },
      buildRequest,
    );
  };

  const openTracks = (preselect: number[]) => {
    setExtractTarget({ media, path: media.path, preselect });
  };

  return (
    <section className="pane operation-pane operation-pane--fixed">
      <div className="operation-scroll">
      <div className="pane-head">
        <span>{t("shell.operation")}</span>
        <span>02</span>
      </div>

      <header className="op-head">
        <span className="eyebrow">{t("shell.sessionExtract")}</span>
        <h1 className="op-title">{t("header.extractTitle")}</h1>
        <p className="op-sub">{t("header.extractSubtitle")}</p>
      </header>

      <section className="op-section">
        <div className="op-section-head">
          <span>
            <b>01</b> / {t("extract.bulk")}
          </span>
          <span>{t("extract.info")}</span>
        </div>
        <div className="bulk-actions">
          <Button
            onPress={() => openTracks([])}
            isDisabled={busy}
            variant="outline"
          >
            {t("extract.extractTracks")}
          </Button>
          <Button
            onPress={() =>
              openTracks(
                media.streams
                  .filter((stream) => stream.type === "audio")
                  .map((stream) => stream.index),
              )
            }
            isDisabled={busy}
            variant="tertiary"
          >
            {t("extract.allAudio")}
          </Button>
          <Button
            onPress={() =>
              openTracks(
                media.streams
                  .filter((stream) => stream.type === "subtitle")
                  .map((stream) => stream.index),
              )
            }
            isDisabled={busy}
            variant="tertiary"
          >
            {t("extract.allSubtitles")}
          </Button>
        </div>
      </section>

      <section className="op-section">
        <div className="op-section-head">
          <span>
            <b>02</b> / {t("extract.singleAudio")}
          </span>
          <span>{selected ? `#${selected.index}` : "—"}</span>
        </div>
        <div className="extract-grid">
          <Select
            value={language}
            onChange={(key) => {
              if (key) setLanguage(key as "eng" | "spa");
            }}
          >
            <Label>{t("extract.language")}</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                <ListBox.Item id="eng" textValue={t("languages.eng")}>
                  {t("languages.eng")}
                  <ListBox.ItemIndicator />
                </ListBox.Item>
                <ListBox.Item id="spa" textValue={t("languages.spa")}>
                  {t("languages.spa")}
                  <ListBox.ItemIndicator />
                </ListBox.Item>
              </ListBox>
            </Select.Popover>
          </Select>

          {candidates.length > 1 ? (
            <Select
              value={audioIndex === null ? null : String(audioIndex)}
              onChange={(key) => {
                if (key !== null) setAudioIndex(Number(key));
              }}
            >
              <Label>{t("extract.track")}</Label>
              <Select.Trigger>
                <Select.Value />
                <Select.Indicator />
              </Select.Trigger>
              <Select.Popover>
                <ListBox>
                  {candidates.map((stream) => (
                    <ListBox.Item
                      id={String(stream.index)}
                      key={stream.index}
                      textValue={`#${stream.index} ${stream.codec}`}
                    >
                      {`#${stream.index} / ${stream.codec.toUpperCase()} / ${channelText(stream)}${
                        stream.title ? ` / ${stream.title}` : ""
                      }`}
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  ))}
                </ListBox>
              </Select.Popover>
            </Select>
          ) : (
            <div className="static-field">
              <span className="static-label">{t("extract.track")}</span>
              <span className="static-value">
                {selected
                  ? `#${selected.index} / ${selected.codec.toUpperCase()} / ${channelText(selected)}`
                  : t("extract.noTracks")}
              </span>
            </div>
          )}
        </div>
        {candidates.length === 0 ? (
          <p className="notice">{t("extract.noTracks")}</p>
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
          onPress={startSingle}
          variant="primary"
        >
          {busy || launching ? <Spinner color="current" size="sm" /> : null}
          {t("extract.createCopy")}
        </Button>
      </div>
      {collisionDialog}
    </section>
  );
}

