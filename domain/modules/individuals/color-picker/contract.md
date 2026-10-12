# thaum-renderer/domain/modules/individuals/color-picker

## purpose
Own a generic indexed-color picker: every color in one renderer-facing indexed palette drawn as one clickable swatch, selecting one on click. The renderer-owned equivalent of the old mono_ui `color_selector_module.ts`'s indexed swatch grid, minus its painter-specific material/appearance-slot rows.

## owns
- the `ColorPickerModule` type and its `Module` trait implementation (`draw`, `on_pointer_event`, `wants_pointer_capture`, `is_closed`)
- flowing-list swatch layout over one indexed palette: every swatch packed edge-to-edge with no gap, wrapping to the next row at however many columns fit the current content width, recomputed fresh every `draw()` from `PanelChrome::content_size()` so the list reflows as the panel is resized; swatches that do not fit the current content size are clipped (not drawn, not hittable) rather than drawn past the border
- swatch display order derived from the colors themselves — the gray ramp (dark→light, saturation at or below the 5% buffer zone) first, then hue families each running light→dark (value descending, saturation descending as tie-break) — exposed as the `display_order()` read view; clicking selects and never reorganizes the list (J 2026-10-11: the old most-recently-used reordering made the list annoying to use; J 2026-10-10: the value-major sort scattered hue families, hue-major ramps read as a palette breakdown)
- single-swatch selection state keyed by raw RGB (so a palette swap or re-sort never loses the selection) and the selected-swatch highlight (drawn with `UiColorRole::Vivid` from `domain/modules/shared/ui-palette/`)
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
- `thaum-renderer/domain/modules/`
- `thaum-renderer/domain/modules/shared/ui-palette/`
- `thaum-renderer/domain/modules/shared/panel-chrome/`
- `thaum-renderer/domain/modules/shared/module-gizmos/`
- `thaum-renderer/domain/cell-graphic/sprite/sprite-color-space/`

## exposed interfaces
- `ColorPickerModule::new(id, rect, palette)` — construct a picker over the renderer's default canonical palette with the standard move/close/resize/seamless gizmo bar; `rect` must be at least 16 cells wide by 8 tall to fit the gizmo bar and full "COLORS" title on the same top border row
- `ColorPickerModule::with_indexed_palette(indexed_palette)` — replace the default swatches with one consumer-provided indexed palette
- `ColorPickerModule::set_indexed_palette(indexed_palette)` — swap the swatch palette at runtime; the selection survives the swap when its color is still in the new list and drops when it is not; display order is always derived from the colors, never stored
- `ColorPickerModule::indexed_palette()` — read view of the palette currently driving the swatch list
- `ColorPickerModule::display_order()` — the display order as palette indices (value descending, hue ascending); `draw()`, the hotspots, and click hit-testing all read this same order
- `ColorPickerModule::selected_rgb()` — the currently selected swatch's raw RGB, if any
- `ColorPickerModule::select_display_position(order_index)` — select the color now displayed at `order_index`, clamped to the list; the repeated-`−` wipe seam for wrappers that remove the selected swatch
- `ColorPickerModule::set_rect(rect)` / `is_seamless()` / `set_seamless(seamless)` — direct rect and seamless-mode access for wrappers that reserve part of their own bounds around this picker
- `ColorPickerModule::with_footer_rows(footer_rows)` — reserve that many bottom content rows for a wrapper's own footer strip: the swatch grid shrinks to the content rows above them (capacity and hit-test both respect the reserve), so the panel keeps one chrome — one bottom border — while the wrapper renders its strip inside the content area's bottom

## interface consumers
- future renderer implementation surfaces
- `thaum-painter/orchestration/entrypoint/` and other consuming programs wanting a color picker without rebuilding one
  - `thaum-painter/domain/modules/individuals/paint-color-picker/` uses `with_footer_rows` (with `new`, `with_indexed_palette`, `set_indexed_palette`, `selected_rgb`, `display_order`, `select_display_position`, `set_rect`, `set_seamless`)

## artifacts
- none

## tests
- `color_picker_module.rs`'s inline `#[cfg(test)]` module
  - light
  - validates one swatch cell is drawn per canonical palette color, the chrome border/gizmo bar/title draw around the list, no swatch is selected before a click, clicking selects the exact palette color under the click through the derived display order, out-of-list and on-the-border clicks are ignored, the selected swatch draws with the palette's vivid role color, clicking move/resize then a `Move` event relocates/grows the rect in real time (and stops once `Up` is delivered), clicking close marks the module closed, clicking seamless hides the border/title while keeping the gizmo bar visible, a gizmo click never also selects a swatch, swatches pack edge-to-edge with no gap and wrap at the current content width, shrinking the panel reflows swatches into more rows while they fit and clips the overflow once they do not, a footer reserve keeps swatches out of the reserved bottom rows and makes clicks there non-selecting, display order sorts the gray ramp first then hue-major value ramps (buffer-zone near-grays ride the gray ramp, same hue+value sorts more saturated first), clicking a swatch reorders nothing, and selection survives a palette swap when the color stays and drops when it leaves.

## data
- none

## notes
- old-system precedent: `THAUMWORLD-AUTO-STORY-TELLER/src/mono_ui/modules/color_selector_module.ts` (swatch grid + left/right-hand selection over materials and a 37-name indexed palette) and `color_picker_module.ts` (a separate, more complex HSV field + hue/value slider picker).
- this picker is now explicitly the indexed-color side of the system; the separate HSV field lives in `domain/modules/individuals/color-block/`.
- consumers can keep the renderer default palette or replace it with their own indexed palette through `with_indexed_palette(...)`, and re-swap it mid-session through `set_indexed_palette(...)` when their palette policy owns a live, user-edited list (the painter's palette module re-seeds the swatches on its next pass).
- dual left/right-hand selection is deferred for the same reason documented in `domain/modules/shared/ui-palette/contract.md`: the pointer-event model doesn't carry which mouse button was clicked yet.
- selection highlight swaps the swatch glyph from `█` to `◆` and recolors it with the palette's `vivid` role, rather than drawing a separate border cell — keeps the module to one `Cell` per swatch.
- as of the border pass, `draw()` prepends `domain/modules/shared/panel-chrome/`'s border/background/title cells (titled "COLORS") before the swatch cells, and `on_pointer_event` subtracts `PanelChrome::content_inset()` before hit-testing a click against the grid — this is the module's first real ASCII-bordered panel, replacing the earlier bare block of swatches with no frame.
- as of this pass, it also carries the standard `domain/modules/shared/module-gizmos/` bar (move, close, resize, seamless — matching the old mono_ui gizmo set minus `save_position`, per direct user request). A `Click` first checks the gizmo bar; only an unhandled click falls through to swatch selection, so a gizmo click never also selects a swatch. `PanelChrome`'s title is pushed right with `with_title_start_x(gizmos.title_start_x())` so it never overlaps the gizmo glyphs — this is why `rect` grew from 12 to 16 cells wide (room for four gizmo glyphs plus the full "COLORS" title on the same row).
- move and resize are driven by real per-frame pointer capture, not simulated: a gizmo click arms the mode and requests capture (`wants_pointer_capture`); the owning `ModuleRegistry` then keeps delivering `ModulePointerEvent::Move` to this module every frame the button stays held (via `dispatch_captured_pointer_move`, driven by `thaum-painter/orchestration/entrypoint/`'s frame loop reading `WindowSurfaceInput.pointer_down`/`cursor_position`), and `Up` ends the drag.
- seamless always hides `panel-chrome/`'s border/background/title; the gizmo bar itself only draws while seamless and the pointer is currently hovering the module (`GizmoState::should_draw_gizmo_bar()`), so the UI can go fully immersive until moused over, then reveal the gizmos (including seamless itself, to toggle chrome back on) — per direct user request.
- as of this pass, the swatch grid became a flowing list: `swatch_layout()` reads `PanelChrome::content_size()`'s width/height every `draw()` to decide how many columns fit and how tall the content area is, and swatches draw packed edge-to-edge (step of exactly `1`, no gap) rather than the earlier grid's spaced-out layout — per direct user request that the previous grid's inter-swatch spacing "isn't what I was looking for." The list aggregates downward from the top border, not upward from the bottom, per direct user correction that the first pass had this flipped.
- as of the 2026-10-11 palette pass, the click-to-front most-recently-used order is gone: display order is derived from the colors themselves (`display_order()`, value descending then hue ascending, ties keeping palette order) and clicking only selects — per direct user request, the reordering "makes it really annoying to use... just a list." Selection is keyed by raw RGB instead of list position so palette swaps and re-sorts never lose it, and swatches that no longer fit the shrunken content are clipped instead of drawn past the border.
- `select_display_position` exists for the wipe flow (J 2026-10-11: deleting one color at a time "sucks... what if the color selected when we deleted went to the first color... bonus lopoints if it goes to the index of the deleted color"): after a wrapper removes the selected swatch, the color that takes its display slot becomes the selection, so `−` can be pressed repeatedly to wipe in sorted order. The wrapper owns the remove + reselect sequencing; the picker only owns the clamped reselect.
- the footer reserve (`with_footer_rows`) exists because a wrapper that renders its own footer strip inside the panel needs the swatch grid to stop above it while the panel keeps exactly one chrome — one bottom border. Swatch rows stay anchored to the content area's top (the reserve clips capacity, not the anchor), matching the header-row precedent: reserved rows are subtracted from what a layout may use, never hardcoded into the chrome.
- source-of-truth from J 2026-10-10 (sort pass): "How can we sort colors so that they are helpful for painting and indexing / looking at a palettes color setup?" and "Maybe a slight buffer zone for saturation instead of just 0 saturation bands. Like saturation 5% and lower." — the display order is hue-major with a light→dark value ramp per hue family, grays pulled out ahead as their own dark→light ramp, and the gray band uses a 5% saturation buffer so near-neutrals don't strand as one-color hue families.
