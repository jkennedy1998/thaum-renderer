# /home/j/Repos/thaum-renderer/domain/modules/individuals/color-picker

## purpose
Own a generic indexed-color picker: every color in one renderer-facing indexed palette drawn as one clickable swatch, selecting one on click. The renderer-owned equivalent of the old mono_ui `color_selector_module.ts`'s indexed swatch grid, minus its painter-specific material/appearance-slot rows.

## owns
- the `ColorPickerModule` type and its `Module` trait implementation (`draw`, `on_pointer_event`, `wants_pointer_capture`, `is_closed`)
- flowing-list swatch layout over one indexed palette: every swatch packed edge-to-edge with no gap, wrapping to the next row at however many columns fit the current content width, recomputed fresh every `draw()` from `PanelChrome::content_size()` so the list reflows as the panel is resized
- swatch display order (most-recently-clicked-first): clicking a swatch both selects it and moves it to the front of the list, ahead of every other swatch
- single-swatch selection state and the selected-swatch highlight (drawn with `UiColorRole::Vivid` from `domain/modules/shared/ui-palette/`)
- its own `GizmoState`/closed flag: the live rect while a move/resize drag is in progress, whether its close gizmo has been clicked, and (while seamless) whether the gizmo bar is currently visible based on pointer hover

## does not own
- any one program's palette policy; this module only consumes whichever indexed palette it is given
- the semantic color-role palette used for chrome/highlight color, owned by `domain/modules/shared/ui-palette/`
- the border/background/title chrome primitive itself, owned by `domain/modules/shared/panel-chrome/`
- the gizmo bar's layout, hit-testing, and move/resize/seamless drag math, owned by `domain/modules/shared/module-gizmos/`
- actually delivering captured pointer movement during a drag, or purging this module once it reports itself closed — both owned by `domain/modules/`'s `ModuleRegistry`
- what a consumer does with the selected color (assigning it to a brush, a material slot, etc.) — that is consumer wiring on top of `selected_rgb()`
- dual left/right-hand color selection (painting with two different mouse-button colors at once, like the old system's `left_rgb`/`right_rgb`) — deferred, see notes

## children-encapsulations
- none

## contents
- `color_picker_module.rs`
  - `ColorPickerModule` and its `Module` trait implementation

## dependencies
- `/home/j/Repos/thaum-renderer/domain/modules/`
- `/home/j/Repos/thaum-renderer/domain/modules/shared/ui-palette/`
- `/home/j/Repos/thaum-renderer/domain/modules/shared/panel-chrome/`
- `/home/j/Repos/thaum-renderer/domain/modules/shared/module-gizmos/`
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/sprite/sprite-color-space/`

## exposed interfaces
- `ColorPickerModule::new(id, rect, palette)` — construct a picker over the renderer's default canonical palette with the standard move/close/resize/seamless gizmo bar; `rect` must be at least 16 cells wide by 8 tall to fit the gizmo bar and full "COLORS" title on the same top border row, and to fit every swatch without the list overflowing its own content height at that default width
- `ColorPickerModule::with_indexed_palette(indexed_palette)` — replace the default swatches with one consumer-provided indexed palette
- `ColorPickerModule::selected_rgb()` — the currently selected swatch's raw RGB, if any

## interface consumers
- future renderer implementation surfaces
- `thaum-painter/orchestration/entrypoint/` and other consuming programs wanting a color picker without rebuilding one

## artifacts
- none

## tests
- `color_picker_module.rs`'s inline `#[cfg(test)]` module
  - light
  - validates one swatch cell is drawn per canonical palette color, the chrome border/gizmo bar/title draw around the list, no swatch is selected before a click, clicking selects the exact palette color under the click, out-of-list and on-the-border clicks are ignored, the selected swatch draws with the palette's vivid role color, clicking move/resize then a `Move` event relocates/grows the rect in real time (and stops once `Up` is delivered), clicking close marks the module closed, clicking seamless hides the border/title while keeping the gizmo bar visible, a gizmo click never also selects a swatch, swatches pack edge-to-edge with no gap and wrap at the current content width, shrinking the panel reflows swatches into more rows without losing any, and clicking a swatch reorganizes it to the front of the list (displacing what used to be first).

## data
- none

## notes
- old-system precedent: `THAUMWORLD-AUTO-STORY-TELLER/src/mono_ui/modules/color_selector_module.ts` (swatch grid + left/right-hand selection over materials and a 37-name indexed palette) and `color_picker_module.ts` (a separate, more complex HSV field + hue/value slider picker).
- this picker is now explicitly the indexed-color side of the system; the separate HSV field lives in `domain/modules/individuals/color-block/`.
- consumers can keep the renderer default palette or replace it with their own indexed palette through `with_indexed_palette(...)`.
- dual left/right-hand selection is deferred for the same reason documented in `domain/modules/shared/ui-palette/contract.md`: the pointer-event model doesn't carry which mouse button was clicked yet.
- selection highlight swaps the swatch glyph from `█` to `◆` and recolors it with the palette's `vivid` role, rather than drawing a separate border cell — keeps the module to one `Cell` per swatch.
- as of the border pass, `draw()` prepends `domain/modules/shared/panel-chrome/`'s border/background/title cells (titled "COLORS") before the swatch cells, and `on_pointer_event` subtracts `PanelChrome::content_inset()` before hit-testing a click against the grid — this is the module's first real ASCII-bordered panel, replacing the earlier bare block of swatches with no frame.
- as of this pass, it also carries the standard `domain/modules/shared/module-gizmos/` bar (move, close, resize, seamless — matching the old mono_ui gizmo set minus `save_position`, per direct user request). A `Click` first checks the gizmo bar; only an unhandled click falls through to swatch selection, so a gizmo click never also selects a swatch. `PanelChrome`'s title is pushed right with `with_title_start_x(gizmos.title_start_x())` so it never overlaps the gizmo glyphs — this is why `rect` grew from 12 to 16 cells wide (room for four gizmo glyphs plus the full "COLORS" title on the same row).
- move and resize are driven by real per-frame pointer capture, not simulated: a gizmo click arms the mode and requests capture (`wants_pointer_capture`); the owning `ModuleRegistry` then keeps delivering `ModulePointerEvent::Move` to this module every frame the button stays held (via `dispatch_captured_pointer_move`, driven by `thaum-painter/orchestration/entrypoint/`'s frame loop reading `WindowSurfaceInput.pointer_down`/`cursor_position`), and `Up` ends the drag.
- seamless always hides `panel-chrome/`'s border/background/title; the gizmo bar itself only draws while seamless and the pointer is currently hovering the module (`GizmoState::should_draw_gizmo_bar()`), so the UI can go fully immersive until moused over, then reveal the gizmos (including seamless itself, to toggle chrome back on) — per direct user request.
- as of this pass, the swatch grid became a flowing list: `swatch_layout()` reads `PanelChrome::content_size()`'s width/height every `draw()` to decide how many columns fit and how tall the content area is, and swatches draw packed edge-to-edge (step of exactly `1`, no gap) rather than the earlier grid's spaced-out layout — per direct user request that the previous grid's inter-swatch spacing "isn't what I was looking for." `order: Vec<usize>` (canonical-palette indices) replaces the fixed column/row indexing; a click now both selects a swatch and moves it (`Vec::remove` + `Vec::insert(0, ..)`) to the front of `order`, so display order becomes most-recently-clicked-first — the read of "the user can click them and they'll get reorganized."
- the list aggregates downward from the top border, not upward from the bottom: order index `0` (front of the list) lands at the highest content row (`content_height - 1`), each following order index one row lower, wrapping right-to-left-then-down within a row same as ordinary reading order — per direct user correction that the first pass had this flipped. Selection highlight, gizmo hit-testing, and `is_closed`/`wants_pointer_capture` plumbing are otherwise unchanged.
