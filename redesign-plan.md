# OpenSCQ30 desktop redesign plan

> Implementation brief for a coding LLM. Use this plan with [`design-thoughts.md`](design-thoughts.md) and the screen concepts in [`reference/`](reference/README.md).

## Objective

Redesign the existing OpenSCQ30 desktop interface around the visual direction in the Q20i concepts while preserving the current device communication, setting behavior, localization, and support for every model in `DeviceModel`. Load a model-matching device image from a URL at runtime; do not add a local product-image file for every supported headset.

Implement the first visual pass against the Soundcore Q20i (A3004), while keeping all redesigned pages and image lookup data-driven so they work for the rest of the supported catalog.

## Repository facts to preserve

- The desktop GUI is Rust using COSMIC/Iced (`gui/Cargo.toml`).
- `DeviceSettingsModel` builds navigation from `device.categories()` and renders device settings from the setting metadata. Keep that dynamic behavior; do not replace it with a Q20i-only hard-coded settings list. See `gui/src/device_settings.rs`.
- Setting updates are already represented by GUI messages such as `SetSetting` and `SetEqualizerBand`; route redesigned controls through the existing update path.
- Quick preset create, apply, edit, delete, refresh, and persistence behavior already exists in `gui/src/device_settings/quick_presets.rs`. Restyle and reorganize its view without dropping those actions.
- The supported-model enum is `lib/src/devices/device_model.rs`. Some marketed models share a device implementation, so image identity should be based on a canonical product/model code, not blindly one URL per enum variant.
- Q20i/A3004 currently exposes ambient sound modes, a 10-band equalizer, battery, serial number, and firmware information. It does not expose custom button mapping, spatial audio, or a bass-enhancement toggle. Derive available UI from actual device categories/settings for every model.
- The current desktop shell already has navigation. The concepts are visual targets, not a request to replace the device protocol or invent unsupported settings. Values in the concepts are illustrative only.

## Design direction

- Use the four Q20i concept screens as the visual reference: Overview, Equalizer, Quick Presets, and Device Information.
- Keep OpenSCQ30 branding and the current desktop interaction model. Use a persistent navigation area, clear page titles, spacious work areas, tonal charcoal cards, restrained red for meaningful active/custom states, and serif/sans typography contrast where platform fonts support it.
- Preserve an existing light theme if enabled by the app. Use semantic theme tokens rather than hard-coding every surface to dark colors.
- Keep device imagery secondary to the settings. Show the exact selected model image only when a matching image is available. Do not reuse the generic inspiration headset or the Q20i render for another model.
- Hide unsupported settings and destinations. Do not add placeholder cards for features a device does not expose.

## Implementation stages

### 1. Inspect and map current behavior

- Trace the current app shell, device navigation, settings categories, equalizer state, quick preset view, theme configuration, cache/config directories, and localization patterns before editing.
- Build a checklist from every `DeviceModel` variant and its canonical product code/display name. Mark aliases that share one canonical image identity.
- Identify which existing categories and settings can be rearranged into the concept pages without duplicating state or bypassing current update messages.
- Record any image widget/decoder support already enabled through COSMIC/Iced dependencies. Add only the missing UI/image/network capability.

**Deliverable:** a short in-code mapping/design note and a list of existing messages/models each redesigned control will reuse.

### 2. Add URL-based model image lookup

- Add a small image catalog keyed by canonical product code (for example, `A3004`) and map each supported `DeviceModel` to its canonical code.
- Each catalog entry should allow an optional HTTPS image URL and source/attribution metadata. Do not fabricate URLs or infer CDN paths. Populate URLs only when the direct image URL is verified. A missing URL is a valid state and must use a neutral fallback plus the model name.
- Prefer stable direct image resources from a source the project can rely on. If only manufacturer/CDN hotlinks are available, document that they may change and that image-use rights must be checked. Do not fetch or parse product HTML pages to locate images at app startup.
- Fetch only the selected device's image, asynchronously. Validate HTTPS, expected image MIME types, response size, decode success, and sensible timeouts. Avoid blocking the GUI thread.
- Cache decoded images in memory and image bytes/metadata in the platform's cache directory (not the source tree or user settings directory). Use a bounded cache, reuse an existing cached image offline, and refresh stale URLs in the background when possible.
- Deduplicate concurrent requests for the same canonical model. Ignore or cancel a late response if the user has switched to a different selected device.
- On any URL/network/decode/cache error, keep the rest of the app fully usable and show the model-specific fallback. Do not repeatedly retry on every redraw.
- Make the external image request visible in the design/implementation notes: a runtime fetch contacts the configured image host. Do not send device identifiers or settings with it.

**Deliverable:** selected-model image component/service, complete model-code mapping with explicit missing-image states, and cache/fallback behavior. No product binaries added to Git.

### 3. Establish reusable visual components

- Define a small set of theme-aware tokens for page spacing, card padding, surface levels, title/body styles, muted text, dividers, selection, focus, and accent states.
- Create/reuse consistent components for page headers, feature cards, mode choices, setting rows, preset cards, and device identity/status.
- Ensure controls communicate selected, focused, disabled, loading, unavailable, and error states without relying on red alone.
- Keep copy and labels localized through the existing i18n flow; do not introduce image-baked UI text.

**Deliverable:** reusable view components and style rules that can support all categories without replacing dynamic setting metadata.

### 4. Redesign the application shell and Overview

- Retain current app navigation and device selection flows. Improve alignment, spacing, active navigation treatment, and page title hierarchy to match the concepts.
- Add an Overview for a connected device with model-matched image, connection state, battery when available, and cards/shortcuts assembled only from that device's actual capabilities.
- For Q20i, show ambient sound choices (Normal, Transparency, Noise Cancellation), the current equalizer/profile summary, and device details/shortcuts supported by A3004.
- If a category is absent for another model, omit that card/shortcut. Do not show fake battery, connection, or settings values.
- Keep disconnected, loading, and failed connection states intentional and readable.

**Deliverable:** a complete model-aware connected-device overview that continues to use existing state and messages.

### 5. Redesign Equalizer

- Preserve the existing equalizer settings model and `SetEqualizerBand` update path.
- Use the band count, bounds, labels, and values supplied by the setting metadata; do not hard-code Q20i frequencies or ten handles into a shared component.
- Present a readable graph/editor and preset area with a selected profile state. Support mouse/touchpad and keyboard operation, visible focus, and numeric/accessible value feedback.
- Keep preset selection and custom editing behavior distinct. Ensure reset/save actions map to existing behavior or are omitted if unsupported.
- Test the layout with devices having different equalizer band counts or no equalizer.

**Deliverable:** one reusable equalizer page that adapts to each device's actual equalizer definition.

### 6. Redesign Quick Presets and Device Information

- Restyle Quick Presets as a responsive card/list view with name, current/applied state, and clear Apply/Edit/Delete actions. Preserve all current model-specific storage and edit flows.
- Only show a curve preview when the saved preset data can provide one accurately; do not invent curve data from preset names.
- Present device identity and information fields from the selected device (including serial and firmware only when available). Keep sensitive/technical values secondary and copyable if the existing UI supports it.
- Do not show a customizable Controls page for Q20i unless the backend later implements it. Other devices may expose such a page when their actual categories/settings support it.

**Deliverable:** redesigned presets and information pages with existing actions intact and no unsupported controls.

### 7. Verify and finish

- Run the repository's established formatting/build checks after implementation; do not suppress existing failures or change unrelated protocol code.
- Manually inspect at least one representative device per capability shape: Q20i with ambient modes + EQ, a model with richer categories, and a model with fewer settings or no image URL.
- Verify image loading with cache hit, first fetch, offline startup, invalid URL, corrupt image, oversized response, model switch during fetch, and missing catalog entry.
- Check Linux and Windows layouts, plus macOS if the existing GUI build supports it; check resizing, light/dark themes, keyboard navigation, text scaling, and longer translated labels.
- Confirm no device address, serial number, firmware value, or setting state is sent to the image host.
- Update `design-thoughts.md` and `reference/README.md` only if the implementation changes the accepted direction or screen set.

## Acceptance criteria

- Every currently supported `DeviceModel` resolves to a canonical model identity, whether or not a URL is available.
- The selected model never displays another model's headset image. Missing images use the fallback cleanly.
- Images are fetched asynchronously and cached outside the repository; startup and core settings remain usable offline.
- The GUI presents the same visual direction as the four Q20i concepts while staying recognizable as OpenSCQ30.
- Existing settings, equalizer edits, quick preset operations, connection flow, and localization continue to work.
- All controls shown for a device are backed by that device's real settings/capabilities; absent capabilities are omitted.
- No product-image binaries are committed as part of the redesign.

## Suggested implementation order

1. Map model codes and capability shapes; establish URL catalog schema and fallback.
2. Implement and manually verify the async image fetch/cache service independently of layout work.
3. Build theme-aware shell and reusable components.
4. Deliver Overview and Equalizer for Q20i.
5. Restyle Quick Presets and Device Information.
6. Exercise the shared components with representative non-Q20i device models and fill in verified image URLs.
7. Polish cross-platform, localization, accessibility, and offline behavior.
