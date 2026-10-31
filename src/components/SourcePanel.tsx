import { useTranslation } from "react-i18next";
import { Button, toast } from "@heroui/react";
import { FrameDiagram } from "../lib/icons";
import { errorText } from "../lib/errors";
import { baseName, channelText, formatDuration, formatSize, parentName, trackDetail } from "../lib/format";
import { pickVideo } from "../lib/files";
import type { StreamInfo } from "../lib/types";
import { useAppStore } from "../store";

function kindLabel(kind: string, t: (key: string) => string): string {
  switch (kind) {
    case "video":
      return t("tracks.video");
    case "audio":
      return t("tracks.audio");
    case "subtitle":
      return t("tracks.subtitle");
    case "attachment":
      return t("tracks.attachment");
    default:
      return kind;
  }
}

function canExtract(stream: StreamInfo): boolean {
  return stream.type === "audio" || stream.type === "subtitle";
}

export function SourcePanel() {
  const { t } = useTranslation();
  const media = useAppStore((state) => state.media);
  const loadFile = useAppStore((state) => state.loadFile);
  const probing = useAppStore((state) => state.probing);
  const setExtractTarget = useAppStore((state) => state.setExtractTarget);
  const targetWidth = useAppStore((state) => state.targetWidth);
  const targetHeight = useAppStore((state) => state.targetHeight);

  if (!media) return null;

  const video = media.streams.find((stream) => stream.type === "video");
  const tracks = media.streams.filter((stream) =>
    ["video", "audio", "subtitle", "attachment"].includes(stream.type),
  );
  const audioCount = tracks.filter((stream) => stream.type === "audio").length;
  const subtitleCount = tracks.filter((stream) => stream.type === "subtitle").length;
  const matchesTarget = Boolean(
    video &&
      video.width === targetWidth &&
      video.height === targetHeight &&
      video.rotation === 0 &&
      (!video.sampleAspectRatio || video.sampleAspectRatio === "1:1"),
  );

  const changeFile = async () => {
    try {
      const path = await pickVideo();
      if (path) await loadFile(path);
    } catch (error) {
      toast.danger(errorText(error, t));
    }
  };

  return (
    <section className="pane source-pane">
      <div className="pane-head">
        <span>{t("shell.source")}</span>
        <span>01</span>
      </div>

      <div className="source-file">
        <div className="source-title" title={media.path}>
          {baseName(media.path)
            .split(/([._-])/)
            .map((part, index) => (
              <span key={`${part}-${index}`}>
                {part}
                {/^[._-]$/.test(part) ? <wbr /> : null}
              </span>
            ))}
        </div>
        <div className="source-path" title={media.path}>
          {parentName(media.path)}
        </div>
        <div className="source-actions">
          <Button
            className="shell-button"
            isDisabled={probing}
            onPress={() => void changeFile()}
            size="sm"
            variant="tertiary"
          >
            {t("shell.change")}
          </Button>
          <span className="file-flag" data-ok={matchesTarget}>
            {matchesTarget ? t("file.matchesTarget") : t("file.needsResize")}
          </span>
        </div>
      </div>

      <dl className="meta-ledger">
        <div className="meta-cell">
          <dt>{t("file.resolution")}</dt>
          <dd>
            {video?.width && video.height ? `${video.width} × ${video.height}` : "—"}
          </dd>
        </div>
        <div className="meta-cell">
          <dt>{t("file.duration")}</dt>
          <dd>{formatDuration(media.duration)}</dd>
        </div>
        <div className="meta-cell">
          <dt>{t("file.size")}</dt>
          <dd>{formatSize(media.size)}</dd>
        </div>
        <div className="meta-cell">
          <dt>{t("tracks.codec")}</dt>
          <dd>{video?.codec?.toUpperCase() || t("common.unknown")}</dd>
        </div>
        <div className="meta-cell">
          <dt>{t("file.audio")}</dt>
          <dd>{String(audioCount).padStart(2, "0")}</dd>
        </div>
        <div className="meta-cell">
          <dt>{t("file.subtitles")}</dt>
          <dd>{String(subtitleCount).padStart(2, "0")}</dd>
        </div>
      </dl>

      {video?.width && video.height ? (
        <div className="frame-block">
          <div className="frame-caption">
            <span>{t("convert.canvas")}</span>
            <span>{matchesTarget ? t("convert.allGood") : t("file.needsResize")}</span>
          </div>
          <FrameDiagram
            sourceHeight={video.height}
            sourceWidth={video.width}
            targetHeight={targetHeight}
            targetWidth={targetWidth}
          />
        </div>
      ) : null}

      <div className="ledger">
        <div className="ledger-head">
          <span>{t("tracks.title")}</span>
          <span>{String(tracks.length).padStart(2, "0")}</span>
        </div>
        {tracks.map((stream) => (
          <div className="track-row" key={`${stream.type}-${stream.index}`}>
            <span className="track-index">{String(stream.index).padStart(2, "0")}</span>
            <div className="track-main">
              <div className="track-name">
                {kindLabel(stream.type, t)}
                {stream.language ? (
                  <span>
                    {" "}
                    /{" "}
                    {t(`languages.${stream.language}`, {
                      defaultValue: stream.language.toUpperCase(),
                    })}
                  </span>
                ) : null}
                {stream.title ? <span className="track-title"> — {stream.title}</span> : null}
              </div>
              <div className="track-sub">
                {stream.codec?.toUpperCase() || t("common.unknown")}
                {stream.type === "audio" ? ` · ${channelText(stream)}` : ""}
                {stream.type !== "audio" ? ` · ${trackDetail(stream)}` : ""}
                {stream.disposition.default ? ` · ${t("tracks.default")}` : ""}
                {stream.disposition.forced ? ` · ${t("tracks.forced")}` : ""}
              </div>
            </div>
            {canExtract(stream) ? (
              <Button
                className="track-action"
                onPress={() =>
                  setExtractTarget({
                    media,
                    path: media.path,
                    preselect: [stream.index],
                  })
                }
                size="sm"
                variant="ghost"
              >
                {t("actions.extract")}
              </Button>
            ) : null}
          </div>
        ))}
      </div>
    </section>
  );
}
