# Redesign mapping note (Stage 1 deliverable)

Source: `redesign-plan.md` + `design-thoughts.md` + `reference/` concepts.
Code facts traced from `gui/`, `lib/` — no behavior changed.

## App shell (`gui/src/app.rs`, `main.rs`, `config.rs`)

- Screens (`app.rs:129-137`): `DeviceSelection` (start) → `AddDevice`
  (`ModelSelection`/`SelectDevice`/`Error`) → `Connecting` (spinner +
  cancel) → `DeviceSettings` (only screen with `nav_model`).
- `DeviceSettings` nav is dynamic: one entry per `device.categories()`,
  plus `quick-presets`, plus conditional `legacy-equalizer-profile-migration`
  (`device_settings.rs:347-355,970-976`).
- Theme already `System/Light/Dark` (`config.rs:25-32`, `AppTheme`, default
  `System`); applied via `apply_theme()` (`app.rs:652-658`); dropdown in
  settings drawer (`app.rs:370-378`, `Message::SetTheme`).
- Config: `%AppData%/openscq30/openscq30-gui-config.toml`
  (`main.rs:43-47`, `config.rs:47-61`). DB (devices + presets + custom EQ):
  `<config>/openscq30/database.sqlite` (`app.rs:177`,
  `lib/src/api/session.rs:22-26`). No cache dir in use — Stage 2 must add
  `dirs::cache_dir()` usage.
- i18n: `fl!` macro (`gui/src/i18n.rs`), `openscq30-gui.ftl` (16 locales);
  lib strings via `Translate` derive + `openscq30-lib.ftl`.

## Setting metadata → controls (keep dynamic, per plan)

`SettingUiState`/`SettingVariantUiState` (`device_settings.rs:96-209`);
`view_setting` (`597-744`) maps each `Setting` variant to a control in
`device_settings/controls*.rs`. Redesigned controls must reuse:

| Control | Existing message |
|---|---|
| Toggle, Select, MultiSelect, Action, Hue, TimeOfDay, ImportString | `device_settings::Message::SetSetting(SettingId, Value)` (`device_settings.rs:38`), throttled for ranges (`throttle.rs`) |
| Writable equalizer | `Message::SetEqualizerBand(SettingId, u8, i16)` (`device_settings.rs:39`) |
| Read-only equalizer | `canvas(EqualizerLine)` (`equalizer_line.rs`) — reuse for mini-curve previews |
| Preset profile select | `PresetEqualizerProfileSelect` via `optional_select` |
| Quick presets | `quick_presets::Message::{RefreshQuickPresets, ShowCreateQuickPresetDialog, CreateQuickPreset, ActivateQuickPreset, EditQuickPreset, EditQuickPresetToggleField, SnapshotQuickPresetSettings, ShowDeleteQuickPresetDialog, DeleteQuickPreset}` (`quick_presets.rs:12-29`) |
| Copyable info fields | `Message::CopyToClipboard(String)` |

## Equalizer model (metadata-driven, per plan)

- `lib/src/api/settings/equalizer.rs:7-12`: `Equalizer{band_hz, fraction_digits,
  min, max}`; `Setting::Equalizer{setting, read_only, value}`.
- Common 8-visible/10-preset, -120..134, 1 digit, `band_hz =
  [100,200,400,800,1600,3200,6400,12800]` (`common/modules/equalizer.rs`).
- GUI `controls/equalizer.rs:6-59`: per-band spin_button (dB) + slider.
  Redesign: same metadata, new layout; never hard-code 10 handles.

## DeviceModel checklist (42 variants, `lib/src/devices/device_model.rs`)

Alias groups sharing one implementation (one canonical image identity each):

- `A3027`/`A3030` → `a3027`; `A3028`/`A3029` → `a3028`;
  `A3931`/`A3935` → `a3931`; `A3933`/`A3939` → `a3933`;
  `D1202`/`D1202C` → `d1202`. All others (incl. `A3004` Q20i, `A3953`,
  `A3954`, `A6611`, `D1301`, `Development`) unique.
- Q20i/A3004 exposes: ambient sound modes (Normal/Transparency/NoiseCanceling),
  10-band EQ (+import/export), battery, serial, firmware. No button remap,
  spatial, or bass-enhancement — omit those destinations.

## Category → concept page mapping (Q20i first, data-driven for all)

- Overview cards: `SoundModes` category (3-state mode choice), `Equalizer`
  summary (read-only mini curve + profile name → jumps to Equalizer),
  `DeviceInformation` subset (battery/connection) + shortcuts. Cards only for
  categories present in `device.categories()`.
- Equalizer page: full editor bound to `Equalizer` setting metadata +
  `PresetEqualizerProfileSelect`/`CustomEqualizerProfile` where present.
- Quick Presets page: restyle existing view/actions; curve preview only when
  saved data yields a real curve (`EqualizerLine`).
- Device Information page: `Information` settings as-is (copyable); supported
  features derived from actual categories — no Controls page for Q20i.

## Concept-gap decisions (recorded during implementation)

- **01 connection dot:** connection state is text (`bluetooth-connected` /
  `bluetooth-not-connected`), connected rendered with theme accent text, no
  colored dot. Red is never used for connection state.
- **Serif vs sans:** sans throughout, hierarchy via the built-in
  title2/heading/body scale. No serif shipped: no system serif renders
  consistently across Windows/Linux/macOS, and bundling a font would add a
  binary asset against the no-binaries rule. This uses the doc's explicit
  fallback option, not a direction change.
- **02 tabs + numeric EQ input:** implemented as Simple/Advanced mode-choice
  tabs plus the existing per-band spin buttons next to the interactive graph.
  Keyboard users get full precision without touching the canvas.

## Image support verdict (Stage 2 status: implemented)

- No direct `image` decoding, no iced image-widget use, no HTTP client in
  `gui/` deps (only transitive `image 0.25.10`/`resvg` via iced/libcosmic;
  compile-time SVG app icon in `icons.rs`). Stage 2 must add: `image`
  decode + image widget, HTTP client (`reqwest` minimal or `ureq`),
  `dirs::cache_dir` persistence, canonical-code catalog (`A3004` etc.) with
  explicit missing-URL state + neutral fallback. Fetch selected-device only,
  async, validated (HTTPS/MIME/size/timeout), deduped, offline-tolerant.
