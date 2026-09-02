# /home/j/Repos/thaum-renderer/domain/modules/shared/panel-chrome

## purpose
Own the base ASCII-border/background/title chrome primitive every bordered panel module draws itself with — the renderer's port of the old mono_ui `module_borders.ts` (`draw_module_border`, `BORDER_STYLES`). This is the piece `domain/modules/shared/contract.md` had documented as unbuilt "base chrome primitives."

## owns
- the `PanelBorderStyle` enum (`Single`, `Double`, `Thick`) and its box-drawing glyph sets, carried over from the old `BORDER_STYLES.single/double/thick`
- the `PanelChrome` builder: border + optional background fill + optional title, producing rect-local `Cell`s a module merges into its own `CellGroup`
- the fixed `content_inset()` contract (always `1`) a module's own content must respect to clear the border
- `content_size(rect)`: the `(width, height)` in cells actually available to a module's own content inside a given rect, after clearing the border — what a responsive module reads every `draw()` to lay itself out for whatever size it currently is

## does not own
- the higher-level "floating panel" wrapper (background + border + gizmos + arbitrary child content in one reusable `Module`), matching the old `floating_panel_module.ts` — still deferred, needs gizmo behaviors first (see `domain/modules/shared/contract.md`)
- gizmo behaviors (move/resize/close) — separate, unbuilt, see `domain/modules/shared/contract.md`
- dividers/junction glyphs (the old style's `junction_t/b/l/r/x`) — no consumer needs an internal divider yet; only the four corners + straight edges are ported
- which colors a panel uses — callers pass a `UiPalette`, owned by `domain/modules/shared/ui-palette/`
- deciding whether a given module should have a border at all — that stays each module's own choice

## children-encapsulations
- none

## contents
- `panel_chrome.rs`
  - `PanelBorderStyle`, `PanelChrome`, and their inline tests

## dependencies
- `/home/j/Repos/thaum-renderer/domain/modules/`
- `/home/j/Repos/thaum-renderer/domain/modules/shared/ui-palette/`
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/`
- `/home/j/Repos/thaum-renderer/domain/cell-color/`

## exposed interfaces
- `PanelChrome::new(rect, palette)` — a chrome builder defaulted to the old system's "standard UX" chrome colors (thick border in `dimmest`, background in `background`, title in `medium`)
- `PanelChrome::with_style/with_border_color/without_background/with_title/with_title_start_x` — builder overrides
- `PanelChrome::cells()` — the border/background/title `Cell`s, positioned local to the panel's own rect
- `PanelChrome::content_inset()` — how far a module's own content must sit inside the rect to clear the border
- `PanelChrome::content_size(rect)` — the interior width/height in cells currently available, for a module to make its own content responsive to live resizing rather than assuming a fixed size

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/modules/individuals/color-picker/` (reads `content_inset()`/`content_size()` to lay out and hit-test a resize-responsive swatch grid)
- `/home/j/Repos/thaum-renderer/domain/modules/shared/module-gizmos/` (a gizmo-enabled module passes `GizmoBar::title_start_x()` into `with_title_start_x` so the title clears the gizmo row)
- future bordered modules in `domain/modules/individuals/` and consuming programs' own `domain/modules/individuals/`

## artifacts
- none

## tests
- `panel_chrome.rs`'s inline `#[cfg(test)]` module
  - light
  - validates all four corners draw the chosen style's glyphs, background fill covers only the interior (not the border ring), `without_background` omits interior fill cells, `content_size` subtracts the border inset from every side (and never goes negative for a too-small rect), a title draws into the top border row starting two cells in by default (or at `with_title_start_x`'s override, for a gizmo bar to reserve room), truncating rather than overrunning the right border.

## data
- none

## notes
- old-system precedent: `THAUMWORLD-AUTO-STORY-TELLER/src/mono_ui/module_borders.ts`. This ports the corner/edge/title drawing (`draw_module_border`'s core loop) but drops the header's dedicated interior row, its divider/marker options, and the separate `draw_container_box`/divider helpers — none of those have a consumer yet, so they stay out until one needs them, per the same "don't build ahead of a real consumer" pattern as the rest of `domain/modules/`.
- title placement is a deliberate simplification, not a straight port: the old system draws the title on the first interior row (`y1 - 1`, just inside the border), which implicitly reserves that row so a panel's real content has to start one row lower. This version draws the title directly into the top border row itself instead, so a panel's content only ever has to clear the fixed `content_inset()` and never has to know whether it also has a title reserving a row.
- this landed because a concrete consumer (`domain/modules/individuals/color-picker/`) needed to actually look like a bordered panel instead of a bare block of swatches — user-visible feedback was "I see color blocks but don't see modules built out like they used to be."
- the "floating panel" wrapper and gizmos remain the next unbuilt layer on top of this, exactly as already noted in `domain/modules/shared/contract.md`.
