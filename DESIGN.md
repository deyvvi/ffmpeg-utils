# Design — Sala de montaje

FFmpeg Utils uses a technical editorial workbench instead of a dashboard of
cards. The interface should read like a media station: source on the left,
operation on the right, transport at the bottom.

## Principles

1. **The file is the subject.** Typography and layout point at the loaded source,
   its canvas and the output path before decoration.
2. **One accent.** Graphite/bone is the only accent family. Status colours are
   desaturated and appear only as small marks or text.
3. **No generic SaaS chrome.** No sidebar, gradient hero, glow, glass, floating
   card stack, pills or decorative icon cloud.
4. **One family.** Timecodes, resolutions, codecs, paths and progress inherit the
   same HeroUI system sans, with tabular figures.
5. **Actions are deliberate.** One primary action per operation; secondary
   actions are quiet and rectangular.
6. **Motion is functional.** Only progress, spinners and short overlay fades.
   Reduced-motion preferences are respected.

## Typography

One family only: the HeroUI/Tailwind system stack
(`ui-sans-serif, system-ui, "Segoe UI", sans-serif`). Titles, controls,
timecodes, paths and logs all use it. Base text is 14 px, labels 10–11 px and
data 12 px; controls are 36–38 px tall. Weights carry hierarchy; tabular figures
keep data aligned. No serif display font and no separate mono family.

## Colour

Neutral grey only. Base tokens use zero chroma; status colours stay muted and
small.

Dark (default):

- Background: `oklch(0.16 0 0)`
- Surface: `oklch(0.19 0 0)`
- Foreground: `oklch(0.93 0 0)`
- Muted: `oklch(0.66 0 0)`
- Border: `oklch(0.3 0 0)`
- Accent: light grey `oklch(0.88 0 0)` for the primary button.

Light:

- Background: `oklch(0.962 0 0)`
- Surface: `oklch(0.985 0 0)`
- Foreground: `oklch(0.21 0 0)`
- Border: `oklch(0.815 0 0)`
- Accent: charcoal `oklch(0.4 0 0)` for the primary button.

Success, warning and danger keep low chroma and are used as 5 px dots, 2 px
rules or short text labels.

## Layout

- Masthead: brand mark, FFmpeg/AMF status, output folder and settings.
- Numbered mode bar: `01 Convert`, `02 Trim`, `03 Tracks`, `04 Extract`, active via underline.
- Two-pane workbench: source ledger + frame diagram / operation controls.
- Convert exposes resolution presets, custom even dimensions and only the
  hardware encoders that passed a startup test.
- Bottom transport: state, detail, percent, cancel, log toggle and collapsible log.
- Operation content scrolls; the primary action row stays visible inside the
  operation pane.
- Responsive: two columns at 1040 px and above; single scrollable column below.

## Shape and spacing

- Radius: 4 px fields and buttons, 6 px overlays and frames.
- Borders: 1 px hairlines; no drop shadows except modal and drawer overlays.
- Grid: 2 px rules separate ledger rows; sections are separated by rules, not
  floating cards.
- Padding: 28 px desktop pane gutters, 20 px compact, 14 px narrow.

## Icons

Only two product-owned SVGs are allowed:

- The app mark in `src/lib/icons.tsx`.
- The canvas/source frame diagram in `src/lib/icons.tsx`.

Everything else is text, a rule or a native HeroUI control indicator. No Lucide,
Radix icons, Gravity UI or icon clouds.

## Component mapping

- HeroUI v3: Button, TextField/Input, Select/ListBox, Slider, Checkbox,
  ProgressBar, Spinner, Modal, Drawer and Toast.
- Collision, extract, track import, preview and session report use a shared HeroUI modal wrapper.
- Settings use a right-side HeroUI drawer.
- Logs stay inline above the transport, collapsed by default in the recommended
  configuration.

## Anti-slop checklist

- [x] No purple gradient or glow.
- [x] No glassmorphism, blur panels or floating blobs.
- [x] No uniform card grid or pill navigation.
- [x] No icon for every label.
- [x] No emoji, fake data density or decorative stripes.
- [x] Typography has a real hierarchy with one consistent sans family.
- [x] Every accent has a job: action, focus or status.
- [x] Controls are rectangular and compact; overlays feel like reports, not toys.
- [x] Empty, loading, running, failed and completed states all exist.
- [x] Light and dark themes use the same structure, not two different skins.

