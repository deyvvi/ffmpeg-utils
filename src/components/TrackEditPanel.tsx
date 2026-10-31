import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button, Checkbox, Spinner, toast } from "@heroui/react";
import { AddTrackDialog } from "./AddTrackDialog";
import { useJobLauncher } from "../hooks/useJobLauncher";
import { useOutputPreview } from "../hooks/useOutputPreview";
import { api } from "../lib/api";
import { errorText } from "../lib/errors";
import { baseName, channelText } from "../lib/format";
import { pickTrackFile } from "../lib/files";
import type {
  AddedTrackRequest,
  JobRequest,
  MediaInfo,
  SourceTrackChoice,
} from "../lib/types";
import { useAppStore } from "../store";

interface AddedTrack extends AddedTrackRequest {
  id: string;
}

const sourceKey = (index: number) => `src:${index}`;
const addedKey = (id: string) => `add:${id}`;

function makeId(): string {
  return `track-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
}

export function TrackEditPanel() {
  const { t } = useTranslation();
  const media = useAppStore((state) => state.media);
  const tools = useAppStore((state) => state.tools);
  const busy = useAppStore((state) => state.busy);
  const setPreview = useAppStore((state) => state.setPreview);
  const { launchWithRequest, launching, collisionDialog } = useJobLauncher();

  const [audioKeep, setAudioKeep] = useState<Set<number>>(new Set());
  const [subtitleKeep, setSubtitleKeep] = useState<Set<number>>(new Set());
  const [defaultAudio, setDefaultAudio] = useState<string | null>(null);
  const [defaultSubtitle, setDefaultSubtitle] = useState<string | null>(null);
  const [added, setAdded] = useState<AddedTrack[]>([]);
  const [pendingFile, setPendingFile] = useState<MediaInfo | null>(null);
  const [pendingKind, setPendingKind] = useState<"audio" | "subtitle">("audio");

  const audios = useMemo(
    () => media?.streams.filter((stream) => stream.type === "audio") ?? [],
    [media],
  );
  const subtitles = useMemo(
    () => media?.streams.filter((stream) => stream.type === "subtitle") ?? [],
    [media],
  );
  const addedAudio = useMemo(
    () => added.filter((track) => track.kind === "audio"),
    [added],
  );
  const addedSubtitles = useMemo(
    () => added.filter((track) => track.kind === "subtitle"),
    [added],
  );
  const keptSourceAudio = useMemo(
    () => audios.filter((stream) => audioKeep.has(stream.index)),
    [audios, audioKeep],
  );
  const keptSourceSubtitles = useMemo(
    () => subtitles.filter((stream) => subtitleKeep.has(stream.index)),
    [subtitles, subtitleKeep],
  );

  useEffect(() => {
    if (!media) return;
    setAudioKeep(new Set(audios.map((stream) => stream.index)));
    setSubtitleKeep(new Set(subtitles.map((stream) => stream.index)));
    const firstAudio = audios.find((stream) => stream.disposition.default) ?? audios[0];
    const firstSubtitle =
      subtitles.find((stream) => stream.disposition.default) ?? subtitles[0];
    setDefaultAudio(firstAudio ? sourceKey(firstAudio.index) : null);
    setDefaultSubtitle(firstSubtitle ? sourceKey(firstSubtitle.index) : null);
    setAdded([]);
    setPendingFile(null);
    // The media path is the identity of a loaded source.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [media?.path]);

  const audioKeys = [
    ...keptSourceAudio.map((stream) => sourceKey(stream.index)),
    ...addedAudio.map((track) => addedKey(track.id)),
  ];
  const subtitleKeys = [
    ...keptSourceSubtitles.map((stream) => sourceKey(stream.index)),
    ...addedSubtitles.map((track) => addedKey(track.id)),
  ];
  const audioKeysSignature = audioKeys.join("|");
  const subtitleKeysSignature = subtitleKeys.join("|");

  useEffect(() => {
    if (!audioKeys.length) {
      if (defaultAudio !== null) setDefaultAudio(null);
      return;
    }
    if (!defaultAudio || !audioKeys.includes(defaultAudio)) {
      setDefaultAudio(audioKeys[0]);
    }
  }, [audioKeysSignature, defaultAudio]);

  useEffect(() => {
    if (!subtitleKeys.length) {
      if (defaultSubtitle !== null) setDefaultSubtitle(null);
      return;
    }
    if (!defaultSubtitle || !subtitleKeys.includes(defaultSubtitle)) {
      setDefaultSubtitle(subtitleKeys[0]);
    }
  }, [subtitleKeysSignature, defaultSubtitle]);

  const suggestions = useOutputPreview(
    media ? { kind: "editTracks", path: media.path } : null,
  );
  const output = suggestions[0];

  if (!media) return null;

  const video = media.streams.find((stream) => stream.type === "video");
  const disabled = busy || launching || !tools?.found;

  const toggleAudio = (index: number, keep: boolean) => {
    setAudioKeep((current) => {
      const next = new Set(current);
      if (keep) next.add(index);
      else next.delete(index);
      return next;
    });
  };

  const toggleSubtitle = (index: number, keep: boolean) => {
    setSubtitleKeep((current) => {
      const next = new Set(current);
      if (keep) next.add(index);
      else next.delete(index);
      return next;
    });
  };

  const chooseTrack = async (kind: "audio" | "subtitle") => {
    try {
      const path = await pickTrackFile(kind);
      if (!path) return;
      const info = await api.probeTrackFile(path);
      if (!info.streams.some((stream) => stream.type === kind)) {
        toast.info(t("tracks.noCompatible"));
        return;
      }
      setPendingKind(kind);
      setPendingFile(info);
    } catch (error) {
      toast.danger(errorText(error, t));
    }
  };

  const addTrack = (track: AddedTrackRequest) => {
    const id = makeId();
    setAdded((current) => [...current, { ...track, id }]);
    if (track.default) {
      if (track.kind === "audio") setDefaultAudio(addedKey(id));
      else setDefaultSubtitle(addedKey(id));
    }
  };

  const removeAdded = (id: string) => {
    setAdded((current) => current.filter((track) => track.id !== id));
  };

  const buildRequest = (
    allSuggestions: typeof suggestions,
    useAlternate: boolean,
    replace: boolean,
  ): JobRequest => {
    const audio: SourceTrackChoice[] = keptSourceAudio.map((stream) => ({
      streamIndex: stream.index,
      default: defaultAudio === sourceKey(stream.index),
    }));
    const subtitleChoices: SourceTrackChoice[] = keptSourceSubtitles.map((stream) => ({
      streamIndex: stream.index,
      default: defaultSubtitle === sourceKey(stream.index),
    }));
    return {
      kind: "editTracks",
      path: media.path,
      output: useAlternate
        ? allSuggestions[0]?.alternate ?? output?.alternate ?? ""
        : allSuggestions[0]?.path ?? output?.path ?? "",
      audio,
      subtitles: subtitleChoices,
      added: added.map(({ id: _id, ...track }) => track),
      replace,
    };
  };

  const start = () => {
    void launchWithRequest({ kind: "editTracks", path: media.path }, buildRequest);
  };

  const preview = async () => {
    try {
      const nextSuggestions = await api.suggestOutputs({
        kind: "editTracks",
        path: media.path,
      });
      const plan = await api.previewPlan(buildRequest(nextSuggestions, false, false));
      setPreview(plan);
    } catch (error) {
      toast.danger(errorText(error, t));
    }
  };

  const languageName = (code: string) =>
    t(`languages.${code}`, { defaultValue: code.toUpperCase() });

  return (
    <>
      <section className="pane operation-pane operation-pane--fixed">
        <div className="operation-scroll">
          <div className="pane-head">
            <span>{t("shell.operation")}</span>
            <span>02</span>
          </div>

          <header className="op-head">
            <span className="eyebrow">{t("shell.sessionTracks")}</span>
            <h1 className="op-title">{t("header.tracksTitle")}</h1>
            <p className="op-sub">{t("header.tracksSubtitle")}</p>
          </header>

          <section className="op-section">
            <div className="op-section-head">
              <span>
                <b>01</b> / {t("tracks.video")}
              </span>
              <span>
                {video?.width && video.height ? `${video.width} × ${video.height}` : "—"}
              </span>
            </div>
            <p className="notice">{t("tracks.videoCopy")}</p>
          </section>

          <section className="op-section">
            <div className="op-section-head">
              <span>
                <b>02</b> / {t("tracks.audio")}
              </span>
              <span>
                {keptSourceAudio.length + addedAudio.length} / {audios.length}
              </span>
            </div>
            <div className="edit-track-list">
              {audios.map((stream) => {
                const keep = audioKeep.has(stream.index);
                const isDefault = defaultAudio === sourceKey(stream.index);
                return (
                  <div className="edit-track" data-kept={keep} key={stream.index}>
                    <Checkbox
                      aria-label={`#${stream.index}`}
                      isSelected={keep}
                      onChange={(selected) => toggleAudio(stream.index, selected)}
                      variant="secondary"
                    >
                      <Checkbox.Content>
                        <Checkbox.Control>
                          <Checkbox.Indicator />
                        </Checkbox.Control>
                      </Checkbox.Content>
                    </Checkbox>
                    <div className="edit-track-main">
                      <div className="edit-track-name">
                        {stream.title || languageName(stream.language)}
                      </div>
                      <div className="edit-track-meta">
                        {`#${stream.index} / ${stream.codec.toUpperCase()} / ${channelText(stream)} / ${languageName(stream.language)}`}
                      </div>
                    </div>
                    {keep ? (
                      <button
                        aria-pressed={isDefault}
                        className="default-toggle"
                        onClick={() => setDefaultAudio(sourceKey(stream.index))}
                        type="button"
                      >
                        {isDefault ? t("tracks.defaultBadge") : t("tracks.makeDefault")}
                      </button>
                    ) : null}
                  </div>
                );
              })}
              {addedAudio.map((track) => {
                const isDefault = defaultAudio === addedKey(track.id);
                return (
                  <div className="edit-track edit-track--added" data-kept="true" key={track.id}>
                    <span className="added-mark">{t("tracks.added")}</span>
                    <div className="edit-track-main">
                      <div className="edit-track-name">
                        {track.title || baseName(track.path)}
                      </div>
                      <div className="edit-track-meta">
                        {`${languageName(track.language || "und")} / ${baseName(track.path)}`}
                      </div>
                    </div>
                    <button
                      aria-pressed={isDefault}
                      className="default-toggle"
                      onClick={() => setDefaultAudio(addedKey(track.id))}
                      type="button"
                    >
                      {isDefault ? t("tracks.defaultBadge") : t("tracks.makeDefault")}
                    </button>
                    <button
                      className="track-remove"
                      onClick={() => removeAdded(track.id)}
                      type="button"
                    >
                      {t("tracks.remove")}
                    </button>
                  </div>
                );
              })}
            </div>
            <div className="track-section-actions">
              <Button onPress={() => void chooseTrack("audio")} size="sm" variant="outline">
                {t("tracks.addAudio")}
              </Button>
            </div>
            {keptSourceAudio.length + addedAudio.length === 0 ? (
              <p className="notice">{t("tracks.noAudio")}</p>
            ) : null}
          </section>

          <section className="op-section">
            <div className="op-section-head">
              <span>
                <b>03</b> / {t("tracks.subtitles")}
              </span>
              <span>
                {keptSourceSubtitles.length + addedSubtitles.length} / {subtitles.length}
              </span>
            </div>
            <div className="edit-track-list">
              {subtitles.map((stream) => {
                const keep = subtitleKeep.has(stream.index);
                const isDefault = defaultSubtitle === sourceKey(stream.index);
                return (
                  <div className="edit-track" data-kept={keep} key={stream.index}>
                    <Checkbox
                      aria-label={`#${stream.index}`}
                      isSelected={keep}
                      onChange={(selected) => toggleSubtitle(stream.index, selected)}
                      variant="secondary"
                    >
                      <Checkbox.Content>
                        <Checkbox.Control>
                          <Checkbox.Indicator />
                        </Checkbox.Control>
                      </Checkbox.Content>
                    </Checkbox>
                    <div className="edit-track-main">
                      <div className="edit-track-name">
                        {stream.title || languageName(stream.language)}
                      </div>
                      <div className="edit-track-meta">
                        {`#${stream.index} / ${stream.codec.toUpperCase()} / ${languageName(stream.language)}`}
                        {stream.disposition.forced ? ` / ${t("tracks.forced")}` : ""}
                      </div>
                    </div>
                    {keep ? (
                      <button
                        aria-pressed={isDefault}
                        className="default-toggle"
                        onClick={() => setDefaultSubtitle(sourceKey(stream.index))}
                        type="button"
                      >
                        {isDefault ? t("tracks.defaultBadge") : t("tracks.makeDefault")}
                      </button>
                    ) : null}
                  </div>
                );
              })}
              {addedSubtitles.map((track) => {
                const isDefault = defaultSubtitle === addedKey(track.id);
                return (
                  <div className="edit-track edit-track--added" data-kept="true" key={track.id}>
                    <span className="added-mark">{t("tracks.added")}</span>
                    <div className="edit-track-main">
                      <div className="edit-track-name">
                        {track.title || baseName(track.path)}
                      </div>
                      <div className="edit-track-meta">
                        {`${languageName(track.language || "und")} / ${baseName(track.path)}`}
                        {track.forced ? ` / ${t("tracks.forced")}` : ""}
                      </div>
                    </div>
                    <button
                      aria-pressed={isDefault}
                      className="default-toggle"
                      onClick={() => setDefaultSubtitle(addedKey(track.id))}
                      type="button"
                    >
                      {isDefault ? t("tracks.defaultBadge") : t("tracks.makeDefault")}
                    </button>
                    <button
                      className="track-remove"
                      onClick={() => removeAdded(track.id)}
                      type="button"
                    >
                      {t("tracks.remove")}
                    </button>
                  </div>
                );
              })}
            </div>
            <div className="track-section-actions">
              <Button onPress={() => void chooseTrack("subtitle")} size="sm" variant="outline">
                {t("tracks.addSubtitle")}
              </Button>
            </div>
            {keptSourceSubtitles.length + addedSubtitles.length === 0 ? (
              <p className="notice">{t("tracks.noSubtitles")}</p>
            ) : null}
          </section>

          <section className="op-section">
            <div className="op-section-head">
              <span>
                <b>04</b> / {t("shell.destination")}
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
            {t("tracks.startAction")}
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

      <AddTrackDialog
        file={pendingFile}
        kind={pendingKind}
        onAdd={addTrack}
        onClose={() => setPendingFile(null)}
      />
    </>
  );
}
