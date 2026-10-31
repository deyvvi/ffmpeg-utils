import { useAppStore } from "../store";
import { ConvertPanel } from "./ConvertPanel";
import { TrimPanel } from "./TrimPanel";
import { TrackEditPanel } from "./TrackEditPanel";
import { ExtractPanel } from "./ExtractPanel";

export function OperationPanel() {
  const tab = useAppStore((state) => state.tab);

  if (tab === "convert") return <ConvertPanel />;
  if (tab === "trim") return <TrimPanel />;
  if (tab === "tracks") return <TrackEditPanel />;
  return <ExtractPanel />;
}