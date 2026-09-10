# /home/j/Repos/thaum-renderer/domain/modules/shared/tooltip

## purpose
Own the shared hover-tooltip seam: hotspot declarations, the 400ms hover-dwell state, and the one shared framing-card renderer, so any module or command-bar interactive piece (gizmos and bottom-bar buttons now) grows a tooltip by declaring copy — never by building its own tooltip UX.

## owns
- `tooltip.rs`
  - the `Hotspot` type: one hoverable piece of UI as an absolute screen-space rect plus two-line card copy (`title`, `description`) — identification microcontent, never task-critical info
  - the `TooltipState` dwell state: 400ms hover delay (`DWELL`, middle of the 300–500ms UX-research consensus), reset on subject change, cleared when the pointer leaves or a button is held (suppression policy lives with the host: it passes `None` while pressed or while a module holds pointer capture)
  - the `TooltipFrame` tick outcome: the card to draw plus a `dirty` flag so a demand-driven frame loop keeps ticking while a dwell runs and repaints on visibility changes
  - text wrapping at `TEXT_WRAP_COLUMNS` (35): break on spaces, hard-break overlong words
  - the framing-card renderer `tooltip_card_cells`/`tooltip_card_group`: solid weight-3 `█` backer whose rounded outer corners (`▟▙▛▜`, weight 3) are the backer's own corners at z 0 — no square backer cell behind them, so the rounding reads against the screen — framing the highlighted object, inner walls around it (`◩` top, `◪` bottom, `◧` left, `◨` right, weight 3), title centered above and wrapped description centered below (weight 2, standard UI colors). Backer and inner walls share the palette's `Dimmest` role — the walls are the backer fading in toward the highlighted object via their half-block glyph shapes. The highlighted cells themselves are never drawn — the owning module's glyph shows through.
  - depth layering: the backer, its rounded corners, and the inner ring all fill at z 0 — corners and ring replace backer cells rather than stacking over them; only the text overlays at `TEXT_DEPTH` (z 1) inside the same Flat2d group. Real depth, not same-cell overwrite — the backer stays valid behind every overlay cell, and under the camera's perspective the overlay picks up the Flat2d local-depth offset so the card participates in parallax/perspective instead of fighting it (Orthographic renders it flat).
  - placement: preferred title-above/description-below; when the card would leave the screen it stacks both text blocks below (anchor near the top edge) or above (near the bottom edge); horizontally centered on the object and clamped into the visible screen rect the host supplies

## does not own
- which hotspots exist or their copy — each module declares them via `Module::hotspots()` and command bars expose them through `CommandBar::hotspots()`; `GizmoBar::hotspots` and the Painter command bar are producers
- pointer/keyboard delivery or the frame loop — the host feeds `TooltipState::tick` per frame and pushes the composed group last
- pointer hit-testing for the card — it is a display-only overlay and must never block the thing it highlights
- click-initiated long-form help (pop-up tips) — a possible future sibling seam reusing `Hotspot` copy

## children-encapsulations
- none

## contents
- `tooltip.rs`
  - `Hotspot`, `TooltipState`, `TooltipFrame`, `wrap_lines`, `tooltip_card_cells`, `tooltip_card_group`, `DWELL`, `TEXT_WRAP_COLUMNS`, `TEXT_DEPTH`, and their inline tests

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/`
- `/home/j/Repos/thaum-renderer/domain/modules/` (`ModuleRect`, `Module::hotspots`)
- `/home/j/Repos/thaum-renderer/domain/modules/shared/module-gizmos/` (first hotspot producer)
- `/home/j/Repos/thaum-renderer/domain/modules/shared/ui-palette/`

## exposed interfaces
- `Hotspot::new(rect, title, description)`
- `TooltipState::new()` / `tick(hover: Option<&Hotspot>, dt) -> TooltipFrame`
- `tooltip_card_group(hotspot, screen_rect, palette) -> CellGroup`
- `wrap_lines(text, max)`
- `DWELL` (400ms), `TEXT_WRAP_COLUMNS` (35), `TEXT_DEPTH` (1)

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/modules/` (the `Module::hotspots()` seam)
- `/home/j/Repos/thaum-renderer/domain/command-bar/` (visible opted-in button hotspots)
- `/home/j/Repos/thaum-renderer/domain/modules/shared/module-gizmos/`
- `thaum-painter/orchestration/entrypoint/` (frame-loop dwell + topmost overlay push)

## artifacts
- none

## tests
- inline `#[cfg(test)]` in `tooltip.rs`
  - light
  - validates wrap behavior, dwell firing/reset/clear, preferred + both fallback layouts at screen edges, card rendering (corners, inner walls, untouched highlighted cells, present copy), weight-3 backer, depth-layered overlay above an intact backer, and Flat2d group anchoring

## data
- none

## notes
- design session with J: tooltips replace tutorial screens (contextual help over onboarding tours, per NN/g guidance); the framing-card placement is the deliberate experiment — precedent is the spotlight/coach-mark pattern, hover-triggered is the novel part; covering neighbors while highlighting is by design (attention spotlight, J's education-UX premise).
- Custom-gizmo coverage sweep (J 2026-09-10): modules surface custom controls as `Hotspot`s through `GizmoBar::hotspots_with(rect, custom)` — gizmo-bar anchors first, then the module's own control hotspots, all rendered by this one shared card implementation. Shared producers: `PropertyRows::hotspots` (property-row panels), color picker/block, ui-customization rows, graphic picker hits, layers/session panel rows, canvas-bounds wheel-mode toggle.
