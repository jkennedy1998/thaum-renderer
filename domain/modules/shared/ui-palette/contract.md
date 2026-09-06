# thaum-renderer/domain/modules/shared/ui-palette

## purpose
Own the renderer's shared semantic UI color-role palette, so every module's chrome can draw consistent colors from one settable place instead of hardcoding RGB values.

## owns
- the `UiColorRole` enum: `Background`, `Dimmest`, `Medium`, `Bright`, `Vivid`, `LeftHand`, `RightHand`
- the `UiPalette` type: one settable color per role, with sensible defaults
- the default role -> color mapping

## does not own
- what a module draws with a role's color — that stays each module's own chrome logic
- the renderer's canonical working palette used by color pickers and sprite decoding (`sprite_color_space::canonical_sprite_palette`)
- per-consumer palette persistence (saving a customized palette to disk) — a future consumer concern, not owned here

## children-encapsulations
- none

## contents
- `ui_palette.rs`
  - `UiColorRole`, `UiPalette`, and their default color mapping

## dependencies
- `thaum-renderer/domain/cell-color/`

## exposed interfaces
- `UiPalette::get(role) -> CellColor` / `UiPalette::set(role, color)`
- `UiColorRole::ALL` — every role, for iterating a full palette

## interface consumers
- `thaum-renderer/domain/modules/individuals/color-picker/`
- future `domain/modules/shared/` chrome primitives (floating-panel background, borders, etc.)
- `thaum-painter/domain/modules/individuals/` and other consuming programs' modules

## artifacts
- none

## tests
- `ui_palette.rs`'s inline `#[cfg(test)]` module
  - light
  - validates every default role has a distinct color and that `set` only overrides its targeted role.

## data
- none

## notes
- role names and default mapping (`background` = off_black, `dimmest` = deep_blue, `medium` = medium_gray, `bright` = pale_gray, `vivid` = vivid_cyan, `left_hand` = vivid_blue, `right_hand` = vivid_red) are carried over from the old mono_ui `ui_customization_store.ts` so existing intuition (e.g. "vivid" as the accent/highlight color, "left hand"/"right hand" as the two mouse-button paint colors) still applies.
- `left_hand`/`right_hand` exist as named roles now, but nothing dispatches distinct colors per mouse button yet — `ModulePointerEvent`/`WindowSurfaceInput` only capture the primary (left) button today. Real per-button color assignment is deferred until the pointer-event model carries which button was pressed.
- deliberately does not persist customization to disk; that old-system behavior (`load_ui_customization_state`/`save_ui_customization_state`) is a consumer/workshop concern for later, not this domain's job.
