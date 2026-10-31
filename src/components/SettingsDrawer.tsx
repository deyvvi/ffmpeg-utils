import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Button,
  Drawer,
  Input,
  Label,
  ListBox,
  Select,
  TextField,
  toast,
} from "@heroui/react";
import { api } from "../lib/api";
import { errorText } from "../lib/errors";
import { pickFolder } from "../lib/files";
import { useAppStore } from "../store";

export function SettingsDrawer() {
  const { t } = useTranslation();
  const open = useAppStore((state) => state.settingsOpen);
  const setOpen = useAppStore((state) => state.setSettingsOpen);
  const config = useAppStore((state) => state.config);
  const tools = useAppStore((state) => state.tools);
  const setTools = useAppStore((state) => state.setTools);
  const updateConfig = useAppStore((state) => state.updateConfig);
  const [draft, setDraft] = useState(config);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (open) setDraft(config);
  }, [open, config]);

  const set = <K extends keyof typeof draft>(key: K, value: (typeof draft)[K]) => {
    setDraft((current) => ({ ...current, [key]: value }));
  };

  const chooseFfmpeg = async () => {
    const folder = await pickFolder(draft.ffmpegDir || undefined);
    if (folder) set("ffmpegDir", folder);
  };

  const chooseOutput = async () => {
    const folder = await pickFolder(draft.outputDir || undefined);
    if (folder) set("outputDir", folder);
  };

  const detect = async () => {
    const status = await api.detectTools();
    setTools(status);
    if (status.found) toast.success(t("settings.detected"));
    else toast.danger(status.error || t("settings.notDetected"));
  };

  const resetWindow = async () => {
    try {
      await api.setWindowSize(1180, 780);
      set("windowWidth", 1180);
      set("windowHeight", 780);
    } catch (error) {
      toast.danger(String(error));
    }
  };

  const save = async () => {
    setSaving(true);
    try {
      await updateConfig(draft);
      const status = await api.detectTools();
      setTools(status);
      if (status.found) toast.success(t("settings.saved"));
      else toast.danger(status.error || t("settings.notDetected"));
      setOpen(false);
    } catch (error) {
      toast.danger(errorText(error, t));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Drawer.Backdrop isOpen={open} onOpenChange={setOpen}>
      <Drawer.Content placement="right">
        <Drawer.Dialog className="settings-drawer">
          <Drawer.CloseTrigger />
          <Drawer.Header>
            <Drawer.Heading className="dialog-title">{t("settings.title")}</Drawer.Heading>
            <p className="dialog-description">{t("settings.saveHint")}</p>
          </Drawer.Header>

          <Drawer.Body className="settings-body">
            <section className="settings-section">
              <div className="settings-heading">
                <span>01</span>
                {t("settings.appearance")}
              </div>
              <div className="settings-row">
                <div className="field-block">
                  <span className="field-label">{t("settings.theme")}</span>
                  <div className="segmented" role="group" aria-label={t("settings.theme")}>
                    {(["dark", "light", "system"] as const).map((theme) => (
                      <button
                        aria-pressed={draft.theme === theme}
                        key={theme}
                        onClick={() => set("theme", theme)}
                        type="button"
                      >
                        {t(`settings.${theme}`)}
                      </button>
                    ))}
                  </div>
                </div>
                <div className="field-block">
                  <span className="field-label">{t("settings.density")}</span>
                  <div className="segmented" role="group" aria-label={t("settings.density")}>
                    {(["comfortable", "compact"] as const).map((density) => (
                      <button
                        aria-pressed={draft.density === density}
                        key={density}
                        onClick={() => set("density", density)}
                        type="button"
                      >
                        {t(`settings.${density}`)}
                      </button>
                    ))}
                  </div>
                </div>
              </div>

              <Select
                value={draft.language}
                onChange={(key) => {
                  if (key) set("language", String(key));
                }}
              >
                <Label>{t("settings.language")}</Label>
                <Select.Trigger>
                  <Select.Value />
                  <Select.Indicator />
                </Select.Trigger>
                <Select.Popover>
                  <ListBox>
                    <ListBox.Item id="en" textValue="English">
                      English
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                    <ListBox.Item id="es" textValue="Español">
                      Español
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  </ListBox>
                </Select.Popover>
              </Select>
            </section>

            <section className="settings-section">
              <div className="settings-heading">
                <span>02</span>
                {t("settings.ffmpeg")}
              </div>
              <div className="path-row">
                <TextField
                  className="path-field"
                  fullWidth
                  name="ffmpeg-dir"
                  value={draft.ffmpegDir}
                  onChange={(value) => set("ffmpegDir", value)}
                >
                  <Label>{t("settings.ffmpegFolder")}</Label>
                  <Input placeholder={t("settings.automatic")} />
                </TextField>
                <Button onPress={() => void chooseFfmpeg()} variant="tertiary">
                  {t("actions.browse")}
                </Button>
                <Button onPress={() => void detect()} variant="outline">
                  {t("settings.detect")}
                </Button>
              </div>
              <p className="field-help">
                {tools?.found
                  ? `${t("settings.detected")}: ${tools.version}`
                  : t("settings.toolsHint")}
              </p>

              <div className="path-row">
                <TextField
                  className="path-field"
                  fullWidth
                  name="output-dir"
                  value={draft.outputDir}
                  onChange={(value) => set("outputDir", value)}
                >
                  <Label>{t("settings.outputDir")}</Label>
                  <Input placeholder={t("settings.sameAsInput")} />
                </TextField>
                <Button onPress={() => void chooseOutput()} variant="tertiary">
                  {t("actions.browse")}
                </Button>
              </div>
            </section>

            <section className="settings-section">
              <div className="settings-heading">
                <span>03</span>
                {t("settings.encoding")}
              </div>
              <Select
                value={draft.quality}
                onChange={(key) => {
                  if (key) set("quality", String(key));
                }}
              >
                <Label>{t("settings.quality")}</Label>
                <Select.Trigger>
                  <Select.Value />
                  <Select.Indicator />
                </Select.Trigger>
                <Select.Popover>
                  <ListBox>
                    <ListBox.Item id="speed" textValue={t("settings.fast")}>
                      {t("settings.fast")}
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                    <ListBox.Item id="balanced" textValue={t("settings.balanced")}>
                      {t("settings.balanced")}
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                    <ListBox.Item id="quality" textValue={t("settings.qualityHigh")}>
                      {t("settings.qualityHigh")}
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  </ListBox>
                </Select.Popover>
              </Select>

              <div className="number-grid">
                <TextField
                  name="qp-i"
                  type="number"
                  value={String(draft.qpI)}
                  onChange={(value) => set("qpI", Number(value) || 0)}
                >
                  <Label>{t("settings.qpI")}</Label>
                  <Input min={0} max={51} type="number" />
                </TextField>
                <TextField
                  name="qp-p"
                  type="number"
                  value={String(draft.qpP)}
                  onChange={(value) => set("qpP", Number(value) || 0)}
                >
                  <Label>{t("settings.qpP")}</Label>
                  <Input min={0} max={51} type="number" />
                </TextField>
                <TextField
                  name="software-crf"
                  type="number"
                  value={String(draft.softwareCrf)}
                  onChange={(value) => set("softwareCrf", Number(value) || 0)}
                >
                  <Label>{t("settings.crf")}</Label>
                  <Input min={0} max={51} type="number" />
                </TextField>
              </div>

              <div className="reset-row">
                <Button onPress={() => void resetWindow()} variant="tertiary">
                  {t("settings.resetWindow")}
                </Button>
                <span className="field-help">{t("settings.resetHelp")}</span>
              </div>
            </section>
          </Drawer.Body>

          <Drawer.Footer className="dialog-footer">
            <Button onPress={() => setOpen(false)} variant="tertiary">
              {t("actions.cancel")}
            </Button>
            <Button isPending={saving} onPress={() => void save()} variant="primary">
              {t("actions.save")}
            </Button>
          </Drawer.Footer>
        </Drawer.Dialog>
      </Drawer.Content>
    </Drawer.Backdrop>
  );
}

