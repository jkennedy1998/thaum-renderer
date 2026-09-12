# thaum-renderer/domain/modules/individuals/color-block

## purpose
Own a generic color-block picker: a renderer-owned floating module that exposes a hue-driven field plus slider for choosing one RGB color, with optional indexed-palette banding when a consumer wants branded colors instead of arbitrary continuous RGB.

## owns
- the `ColorBlockModule` type and its `Module` implementation
- HSV<->RGB conversion used by this picker
- optional indexed-palette snapping over the displayed field, slider, and committed selected color
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
- `ColorBlockModule::selected_rgb()`
- `ColorBlockModule::set_selected_rgb(rgb)`

## interface consumers
- painter or any future renderer consumer needing arbitrary RGB picking

## artifacts
- none

## tests
- inline `#[cfg(test)]` in `color_block_module.rs`
  - light
  - validates field selection, field and hue-slider drag capture, wheel hue movement, and hue preservation across committed-rgb re-application

## data
- none

## notes
- old-system precedent: `THAUMWORLD-AUTO-STORY-TELLER/src/mono_ui/modules/color_picker_module.ts`.
- wheel input advances the hue slider with wraparound; direct click-and-drag on the slider provides the precise hue-setting path.
- consumers can leave the indexed palette empty for raw HSV behavior, or opt into a program-specific palette for banded colors.
- selected-color weight highlighting is consumer-owned: this module draws all color cells at weight one; consumers post-process their drawn cell group to emphasize selected colors.
