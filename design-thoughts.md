# Design thoughts: desktop app inspired by the mobile references

## Purpose and scope

These notes translate the four screenshots in `inspiration/` into a visual direction for the OpenSCQ30 desktop app. The screenshots show a Nothing audio companion app on a phone. Treat them as references for hierarchy, restraint, and interaction cues, not as a screen-by-screen template: desktop has more width, pointer and keyboard input, and a different information density. The app controls Soundcore devices, so keep OpenSCQ30's product identity and terminology.

## What the references are doing

### Overall visual character

- **Dark, quiet canvas.** The detailed screens use near-black backgrounds, with raised content panels in a slightly lighter charcoal. The page feels spacious because the background is uninterrupted and content is grouped into a few large surfaces.
- **A small palette with a deliberate signal color.** Most content is black, charcoal, white, and subdued gray. Saturated red appears mainly to show a selected/custom equalizer state. In the lighter overview, the same layout logic is expressed with a pale background and dark text; the structure survives the theme change.
- **Two typographic voices.** Large page names such as “Controls” and “Equaliser” use a refined serif. Labels, values, tabs, and actions use a plain sans serif. The contrast makes the screen feel considered without making the controls hard to scan.
- **Product imagery acts as a hero.** A large, isolated headphone render anchors device and controls screens. Its scale and empty surrounding space make hardware identity clear before the user reaches configuration details.
- **Soft geometry, little decoration.** Cards and selection circles are rounded. Borders are rare; spacing and tonal contrast define grouping. Icons are simple, mostly white line or filled symbols.

### Screen 1: connected-device overview

- The device name and battery status sit above the controls, centered beneath a cropped product render. This establishes “which device” and “how charged” before presenting settings.
- Noise cancellation and spatial audio each have a full-width card. Each card has a plain section title and three large, equally weighted circular choices. The active mode is a bright filled circle; inactive modes remain dark and recede. Captions sit directly under the circles, with inactive captions muted.
- Secondary features (bass enhancement, equalizer, channel hop, controls) are arranged as compact two-column tiles. Each tile gives a title, a short current-state summary, and sometimes a small icon or direct toggle. This exposes common destinations without turning the page into a long settings list.
- The consistent card language makes unlike settings feel like one coherent device console. A small overflow affordance on spatial audio signals that more options exist without adding permanent clutter.

### Screen 2: control customization

- A back affordance precedes the serif page title. The large headphone illustration occupies the upper-middle area as orientation and visual breathing room.
- Actions are grouped by physical control (“Button”, “Roller”), rather than presented as one undifferentiated list. Each row pairs an action name with its current assignment, a small symbol, and a trailing chevron. The primary label is brighter and larger than the supporting assignment.
- The dark grouped rows have restrained separators and generous touch height. They read as navigable settings rather than buttons to trigger immediately.

### Screen 3: equalizer

- A back affordance and serif title establish page context. Simple/Advanced tabs sit on a thin baseline; the active tab is made legible by a bright underline rather than a pill.
- The custom EQ control is the visual center. It uses a circular, layered field with draggable points, sparse labels around the edge (bass, mid, treble), and a red point/glow for the current custom adjustment. This makes the sound-shaping task feel tangible rather than like a technical table.
- Presets are separated from the graph and presented as large rounded choices. A full-width red “Custom” action/state anchors the bottom, clearly distinct from the quieter neutral presets.
- The composite has a useful hierarchy: navigation, mode switch, expressive editor, then named presets. Red draws attention to the custom state without coloring every control.

### Screen 4: light overview and equalizer explore

- The overview uses the same card-and-tile layout on a warm near-white background. Dark active circles and light inactive circles invert the selected-state contrast while keeping the same hierarchy.
- The explore view uses a compact grid of preset cards. Each card has a miniature EQ curve, a preset name, and creator/attribution metadata. A short introduction explains the collection before the grid.
- The small curves are previews, not decoration: they help compare presets at a glance. Attribution is visually secondary but visible, and the grid gives a browsing mode without disrupting the editor.

## Direction for OpenSCQ30 desktop

### Layout concept

Use a persistent **device rail + page header + work area** structure:

1. **Device rail (left, about 220–260 px):** app mark/name, connected-device selector and connection status, then primary destinations. Keep General, Equalizer, Hear ID, Buttons, Quick Presets, and Device Information recognizable, but group and space them more deliberately. The active destination should have a clear quiet surface or slim accent marker.
2. **Page header (top of work area):** serif page title, device context where useful, and compact connection/battery status. A small product image may anchor device overview and controls pages, but should not consume the work area on dense settings pages.
3. **Main work area:** responsive columns of feature cards on overview; a wide editor plus preset panel on equalizer; labeled grouped rows for button assignments. Use a centered maximum content width on very wide windows so controls do not stretch excessively.

The current desktop screenshot already has a left navigation rail and dark shell. Keep that familiar structure, but make the settings page feel composed: more page-level hierarchy, consistent card grouping, intentional whitespace, and clearer selected states. Avoid simply enlarging the mobile screen or the current full-width segmented controls.

### Device-specific imagery and capabilities

- Show the image that matches the selected model. The generic headset in `inspiration/device.png` is only an inspiration reference; do not use it as the Q20i image or as a substitute for every model.
- Keep device images in a model-keyed asset catalog (for example `A3004 -> q20i`), with a distinct image for each supported model as assets become available. If a model has no approved image yet, use a neutral model-specific icon/silhouette and its name instead of showing the wrong headset.
- The Q20i is model **A3004**. Its current implementation exposes battery, ambient sound modes (Normal, Transparency, Noise Cancellation), a 10-band equalizer, serial number, and firmware version. It does not expose the button remapping, spatial audio, or bass-enhancement controls shown in the inspiration screens. The Q20i concept screens should omit those controls and omit their navigation destinations.
- The Q20i screens under `reference/` are static design concepts. Connection state, battery percentage, serial number, firmware, and EQ curves shown in them are sample values for layout only; they are not live or verified device data.

### Suggested desktop page compositions

- **Device overview / General:** top identity strip with selected model image, device name, connection state, and battery; below, cards for the sound modes and other capabilities available on that model; a lower grid of compact shortcuts for supported settings. Let each feature card use the mobile reference's title + current value + concise controls pattern. Hide unavailable device capabilities instead of leaving empty tiles.
- **Equalizer:** left/center area for a generously sized EQ graph; right column for named presets and saved/custom profiles. Keep Simple/Advanced (and any supported browse mode) as text tabs with an underline. Provide accessible numeric controls or keyboard-adjustable handles alongside the visual editor. Preset previews can be mini curves. The current desktop Equalizer likely needs more explicit axes/labels than the mobile illustration to take advantage of pointer precision and width.
- **Buttons / controls (where supported):** group rows by physical control, with assignment on the left and the selected action/value aligned consistently on the right. Use a chevron only when a row opens another choice. On desktop, do not let decorative hardware imagery push these groups below the fold; a small model-specific diagram can identify the control being configured. Hide this page for the Q20i until button remapping is supported.
- **Quick presets:** show cards or a tidy list with a name, concise summary, and apply affordance. Clearly mark the current preset and distinguish applying a preset from editing it.

### Palette and surfaces

Use these as starting estimates from the references, then tune against the app's actual GTK theme and accessibility needs:

| Token | Starting direction | Use |
| --- | --- | --- |
| Window | `#090909` to `#101010` | Main shell background |
| Surface | `#1D1D1D` to `#242424` | Feature cards and grouped rows |
| Surface raised | `#2A2A2A` | Hover, selected neutral, or nested controls |
| Text primary | `#F2F2F2` | Titles, selected values, primary labels |
| Text secondary | `#A5A5A5` | Descriptions, inactive labels, metadata |
| Divider | `#343434` | Rare separators and tab baselines |
| Accent | vivid warm red, around `#F21F2B` | Custom EQ and high-salience active states |

These are visual estimates, not sampled source colors. Preserve contrast in small text and focus indicators. Keep accent use scarce: reserve red for a meaningful state or action, not every hover, icon, and navigation item. A future light theme can invert the tonal roles while retaining the same semantic tokens and accent logic.

### Typography and iconography

- Use a restrained serif for large page titles if an available system font renders consistently across Windows, Linux, and macOS. If cross-platform fallback is uneven, use a humanist sans title with more size and weight contrast instead of shipping an inconsistent serif.
- Use a clean sans serif for controls and body copy. Establish three clear levels: page title, section/card title, and supporting value/description. Avoid the current screenshot's uniformly bold control labels as the only hierarchy.
- Keep labels in sentence case. Use concise setting names and show the current value directly below or beside them.
- Prefer a consistent set of simple icons. Pair icons with text for important actions; do not rely on unfamiliar symbols alone. Maintain a consistent stroke/filled style and optical size.

### Spacing, shape, and density

- Establish a repeatable spacing scale (for example 8 / 12 / 16 / 24 / 32 px) and use it consistently inside cards and between sections.
- Use medium-to-large corner radii for cards and mode selectors, with less rounding on small utility controls. Avoid pill shapes for every desktop button; reserve them for compact mode/preset choices where the reference uses them.
- Build clear card padding and alignment before adding borders. Use subtle tonal differences and whitespace for hierarchy; use dividers only inside related rows or tabs.
- Desktop can show more controls at once, but each card should still present a small, scannable set. Use two or three columns only when labels and controls retain comfortable width. On resize, collapse columns cleanly rather than shrinking text.

### Interaction and state

- Selected, hovered, focused, disabled, and unavailable states must be visually distinct. The reference's filled circle is a good selected-state cue for mutually exclusive modes; use a subtle outline or surface shift for keyboard focus, not color alone.
- Make the full mode option clickable, not only its icon. Keep the selected text bright and inactive text muted but still readable.
- For toggles, show both the switch position and a nearby On/Off label where ambiguity matters. For values such as battery or EQ level, pair any graphic with readable text.
- Preserve clear action semantics: a tile opens a setting, a switch changes a setting now, and an apply button commits/applies a preset. Avoid making a whole card clickable if it contains an independent toggle.
- Add keyboard navigation and visible focus treatment. Provide tooltips for icon-only secondary actions and support sensible minimum window widths.
- Use restrained transitions for selection and card hover (short fades/surface shifts); avoid motion that delays setting changes or distracts from device feedback.

## What to carry over, and what to reinterpret

**Carry over:** quiet dark canvas, layered charcoal cards, serif/sans contrast, product identity, large clear mode choices, compact state summaries, meaningful red accent, consistent rounding, and the distinction between editor and preset browsing.

**Reinterpret for desktop:** replace phone-like centered stacks with a stable navigation rail and purposeful columns; use width for side-by-side editing and preset discovery; expose labels and keyboard affordances; show the selected model's own image; and make cards fit the actual capabilities of each supported Soundcore model.

**Avoid:** copying Nothing's branding, product renders, exact iconography, or proprietary assets; using a large product image as filler; making every action red; compressing desktop content into a phone-shaped column; overusing rounded cards until every region has the same weight; or hiding important settings behind unlabeled icons/overflow menus.

## Practical design checklist

- Can a user identify the selected device, connection state, and current page immediately?
- Does every card have one clear purpose and a readable current state?
- Is the active mode obvious without depending on red or brightness alone?
- Does the equalizer support both visual adjustment and precise accessible input?
- Can users distinguish opening a setting from changing it immediately?
- Does the layout remain balanced at a normal laptop width and a wider desktop window?
- Are device-specific unsupported options omitted or clearly unavailable?
- Does the interface remain recognizably OpenSCQ30 rather than a copy of the source app?

## Reference files

- **Selected model image:** `reference/q20i-official-product.png` — Soundcore's official Q20i product render, used only to identify the selected model in these concepts. [Source: Soundcore Q20i product page](https://www.soundcore.com/products/q20i-a3004z31). Confirm usage rights before shipping this image in the app. Preserve its aspect ratio and transparency; never use it for another model.
- **Screen concepts:** see `reference/01-overview-q20i.png`, `reference/02-equalizer-q20i.png`, `reference/03-quick-presets-q20i.png`, and `reference/04-device-information-q20i.png`. These cover the Q20i's overview, EQ editing, saved presets, and device details. The mobile button customization screen was reinterpreted as device information because Q20i button remapping is not currently implemented.
- `inspiration/device.png` — generic headphone image from the visual reference set. Do not use it to represent the selected Soundcore model.

- `inspiration/1.webp` — dark connected-device overview, noise cancellation, spatial audio, and feature tiles.
- `inspiration/2.webp` — control customization page with physical-control groupings.
- `inspiration/3.webp` — equalizer editor, simple/advanced tabs, red custom state, and presets.
- `inspiration/6f200902a6f426315c3f12bc0c958fcea178b880.webp` — light overview plus equalizer preview and explore/preset cards.
- `screenshots/gui_general.png` — current desktop shell and navigation reference used for adaptation notes.
