import { useTranslation } from "react-i18next";
import { useAppStore } from "../store";
import type { TabId } from "../lib/types";

const modes: { id: TabId; label: string; index: string }[] = [
  { id: "convert", label: "nav.convert", index: "01" },
  { id: "trim", label: "nav.trim", index: "02" },
  { id: "tracks", label: "nav.tracks", index: "03" },
  { id: "extract", label: "nav.extract", index: "04" },
];

export function ModeBar() {
  const { t } = useTranslation();
  const tab = useAppStore((state) => state.tab);
  const setTab = useAppStore((state) => state.setTab);

  return (
    <nav className="mode-bar" aria-label={t("app.name")}>
      {modes.map((mode) => (
        <button
          key={mode.id}
          aria-current={tab === mode.id ? "page" : undefined}
          className="mode-tab"
          onClick={() => setTab(mode.id)}
          type="button"
        >
          <span className="mode-number">{mode.index}</span>
          <span className="mode-label">{t(mode.label)}</span>
        </button>
      ))}
      <span className="mode-note">{t("app.tagline")}</span>
    </nav>
  );
}
