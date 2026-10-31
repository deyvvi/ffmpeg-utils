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
} from "@heroui/react";
import { Modal } from "./Modal";
import { baseName, channelText } from "../lib/format";
import type { AddedTrackRequest, MediaInfo } from "../lib/types";

interface AddTrackDialogProps {
  file: MediaInfo | null;
  kind: "audio" | "subtitle";
  onAdd: (track: AddedTrackRequest) => void;
  onClose: () => void;
}

export function AddTrackDialog({ file, kind, onAdd, onClose }: AddTrackDialogProps) {
  const { t } = useTranslation();
  const streams = useMemo(
    () => file?.streams.filter((stream) => stream.type === kind) ?? [],
    [file, kind],
  );
  const [streamIndex, setStreamIndex] = useState<number | null>(null);
  const [language, setLanguage] = useState("");
  const [title, setTitle] = useState("");
  const [isDefault, setIsDefault] = useState(false);
  const [isForced, setIsForced] = useState(false);

  useEffect(() => {
    if (!file) return;
    const first = streams[0];
    setStreamIndex(first?.index ?? null);
    setLanguage(first?.language && first.language !== "und" ? first.language : "");
    setTitle(first?.title ?? "");
    setIsDefault(false);
    setIsForced(false);
  }, [file, kind, streams]);

  const selected = streams.find((stream) => stream.index === streamIndex) ?? null;

  const chooseStream = (index: number) => {
    const stream = streams.find((candidate) => candidate.index === index);
    setStreamIndex(index);
    if (stream) {
      setLanguage(stream.language && stream.language !== "und" ? stream.language : "");
      setTitle(stream.title ?? "");
    }
  };

  const submit = () => {
    if (!file || !selected) return;
    onAdd({
      path: file.path,
      streamIndex: selected.index,
      kind,
      language: language.trim(),
      title: title.trim(),
      default: isDefault,
      forced: kind === "subtitle" && isForced,
    });
    onClose();
  };

  return (
    <Modal
      className="add-track-dialog"
      description={file ? baseName(file.path) : ""}
      footer={
        <>
          <Button onPress={onClose} variant="tertiary">
            {t("actions.cancel")}
          </Button>
          <Button isDisabled={!selected} onPress={submit} variant="primary">
            {t("tracks.addTrack")}
          </Button>
        </>
      }
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
      open={Boolean(file)}
      size="md"
      title={kind === "audio" ? t("tracks.addAudio") : t("tracks.addSubtitle")}
    >
      {streams.length > 1 ? (
        <Select
          value={streamIndex === null ? null : String(streamIndex)}
          onChange={(key) => {
            if (key !== null) chooseStream(Number(key));
          }}
        >
          <Label>{t("tracks.stream")}</Label>
          <Select.Trigger>
            <Select.Value />
            <Select.Indicator />
          </Select.Trigger>
          <Select.Popover>
            <ListBox>
              {streams.map((stream) => (
                <ListBox.Item
                  id={String(stream.index)}
                  key={stream.index}
                  textValue={`#${stream.index} ${stream.codec}`}
                >
                  {`#${stream.index} / ${stream.codec.toUpperCase()}${
                    stream.type === "audio" ? ` / ${channelText(stream)}` : ""
                  }${stream.language ? ` / ${stream.language}` : ""}${
                    stream.title ? ` / ${stream.title}` : ""
                  }`}
                  <ListBox.ItemIndicator />
                </ListBox.Item>
              ))}
            </ListBox>
          </Select.Popover>
        </Select>
      ) : selected ? (
        <div className="static-field">
          <span className="static-label">{t("tracks.stream")}</span>
          <span className="static-value">
            {`#${selected.index} / ${selected.codec.toUpperCase()}${
              selected.type === "audio" ? ` / ${channelText(selected)}` : ""
            }`}
          </span>
        </div>
      ) : (
        <p className="notice">{t("tracks.noCompatible")}</p>
      )}

      <div className="add-track-fields">
        <TextField name="track-language" value={language} onChange={setLanguage}>
          <Label>{t("tracks.language")}</Label>
          <Input placeholder={t("tracks.languageHint")} spellCheck={false} />
        </TextField>
        <TextField name="track-title" value={title} onChange={setTitle}>
          <Label>{t("tracks.titleCol")}</Label>
          <Input placeholder={t("tracks.titleHint")} />
        </TextField>
      </div>

      <div className="track-flags">
        <Checkbox
          isSelected={isDefault}
          onChange={setIsDefault}
          variant="secondary"
        >
          <Checkbox.Content>
            <Checkbox.Control>
              <Checkbox.Indicator />
            </Checkbox.Control>
            {t("tracks.makeDefault")}
          </Checkbox.Content>
        </Checkbox>
        {kind === "subtitle" ? (
          <Checkbox
            isSelected={isForced}
            onChange={setIsForced}
            variant="secondary"
          >
            <Checkbox.Content>
              <Checkbox.Control>
                <Checkbox.Indicator />
              </Checkbox.Control>
              {t("tracks.forced")}
            </Checkbox.Content>
          </Checkbox>
        ) : null}
      </div>
    </Modal>
  );
}
