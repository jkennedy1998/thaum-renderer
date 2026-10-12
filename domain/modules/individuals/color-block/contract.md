# thaum-renderer/domain/modules/individuals/color-block

## purpose
Own a generic color-block picker: a renderer-owned floating module that exposes a hue-driven field plus slider for choosing one RGB color, with optional indexed-palette banding when a consumer wants branded colors instead of arbitrary continuous RGB.

## owns
- the `ColorBlockModule` type and its `Module` implementation
- HSV<->RGB conversion used by this picker
- optional indexed-palette snapping over the displayed field, slider, and committed selected color
- the clamp widget: a toggle cell on the hex readout row (`#` clamped / `*` free, vivid when free) that turns index clamping on and off; the toggle click never picks a color, and re-clamping re-snaps the current selection to the nearest palette entry
- field, hue-slider, drag, and wheel-scroll semantics for one selected RGB color
- direct hue-slider click-and-drag plus wheel-scroll hue movement
- `set_selected_rgb` preserves the continuous wheel-scrolled hue when re-applying the already-committed color, so consumer hand-syncs cannot drag the marker
- picker-local selected color state and marker drawing
- base color-cell weight: field and slider color cells draw at weight one so consumers can emphasize selected colors themselves
- the shared gizmo-bar-backed chrome behavior for this module

## does not own
- painter hand state or left/right assignment semantics
- which indexed palette a consumer wants to use; the consumer passes that palette in
- renderer material definitions, owned by `thaum-renderer/domain/cell-materials/`

## children-encapsulations
- none

## contents
- `color_block_module.rs`
  - `ColorBlockModule` and HSV picker behavior

## dependencies
- `thaum-renderer/domain/modules/`
- `thaum-renderer/domain/modules/shared/`

## exposed interfaces
- `ColorBlockModule::new(id, rect, palette)`
- `ColorBlockModule::with_indexed_palette(indexed_palette)`
- `ColorBlockModule::set_indexed_palette(indexed_palette)` — swap the banded palette at runtime without disturbing the current selection; the next set/snap resolves against the new list
- `ColorBlockModule::clamped()` / `set_clamped(clamped)` — read and toggle index clamping (the widget's backing state; turning clamping back on re-snaps the selection)
- `ColorBlockModule::selected_rgb()`
- `ColorBlockModule::set_selected_rgb(rgb)`

## interface consumers
- painter or any future renderer consumer needing arbitrary RGB picking

## artifacts
- none

## tests
- inline `#[cfg(test)]` in `color_block_module.rs`
  - light
  - validates field selection, field and hue-slider drag capture, wheel hue movement, and hue preservation across committed-rgb re-application; the clamp widget toggles free picking (an off-palette pick holds exactly) and back (re-snapping), never picks a color itself, and draws `#` clamped / `*` vivid when free

## data
- none

## notes
- old-system precedent: `THAUMWORLD-AUTO-STORY-TELLER/src/mono_ui/modules/color_picker_module.ts`.
- wheel input advances the hue slider with wraparound; direct click-and-drag on the slider provides the precise hue-setting path.
- consumers can leave the indexed palette empty for raw HSV behavior, or opt into a program-specific palette for banded colors.
- consumers own the palette policy: `set_indexed_palette` lets a live, document-owned palette the user edits be swapped in mid-session while the user's wheel-scrolled color stays where they left it (the painter's palette module re-bands the block on its next pass).
- selected-color weight highlighting is consumer-owned: this module draws all color cells at weight one; consumers post-process their drawn cell group to emphasize selected colors.
- the clamp widget (J 2026-10-11): with clamping off, field/slider/wheel picks keep their raw color — users can paint naturally outside the palette, then `+` in the palette module adds the hand color. The widget rides the hex readout row so the polished color-block module stays the single surface for free picking.
