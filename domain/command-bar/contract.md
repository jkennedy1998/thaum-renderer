# thaum-renderer/domain/command-bar

## purpose
Own the always-attached command bar surface: the collapsed expand handle and the expanded text-button bar with nested menus, horizontal scroll, and seamless mode, locked to the screen edge independent of hud pan.

## owns
- the collapsed one-cell expand handle and its expand toggle
- the expanded bar body: top border row, button row, overflow scrollbar row, and the blank space rows toward the screen edge
- the button list order and nested menu navigation (`BACK` + `menu_path`)
- the horizontal scrollbar over the button list (thumb drag, track page-jump, either wheel axis) and its scroll offset persistence
- seamless mode (body shown only while hovered) and the `S` toggle
- `CommandBarClickOutcome` emission (`ExpandToggled`, `SeamlessToggled`, `ButtonPressed`)
- per-button idle color role and shared-tooltip hotspot declaration
- screen-edge anchoring of the bar via `update_layout` over the fully-visible screen camera rect

## does not own
- what the buttons do — callers own `ButtonPressed` handling and supply the button lists each frame
- generic module registry plumbing, pointer routing, or tooltip rendering (shared `modules/` seams)
- shared `Scrollbar` mechanics (`Scrollbar` is the reusable seam; this boundary only places and drives it)
- persistence storage — it only emits/consumes `PersistedCommandBarState`

## children-encapsulations
- none

## contents
- `contract.md`
  - command-bar contract
- `command_bar.rs`
  - `CommandBar`, `CommandBarButton`, `CommandBarLayout`, `PersistedCommandBarState`, `CommandBarClickOutcome` — layout, draw, pointer/wheel handling, nested menus, scrollbar, and persistence round-trip

## dependencies
- `thaum-renderer/domain/cell/`: Cell, CellColor, CellGraphic, CellGroup, CellGroupIntakeBehavior, CellPoint, CellWeight, WorldPoint
- `thaum-renderer/domain/modules/`: Hotspot, ModulePointerButton, ModulePointerEvent, ModuleRect, Scrollbar, ScrollbarSpan
- `thaum-renderer/domain/cell-color/` (via `crate`): UiColorRole, UiPalette

## exposed interfaces
### command bar module
send: `update_layout(cell_clip_size, hud_pan_offset)` each frame, pointer events routed by the module registry, wheel events, button/nested-button lists via `set_buttons`/`set_nested_buttons`, persisted state via `apply_persisted_state`
returns: `rect()`, `hotspots()`, `draw()` cell group, `persisted_state()`, at most one `CommandBarClickOutcome` per pointer event, scrollbar drag lifecycle (`is_scrollbar_dragging`/`drag_scrollbar`/`end_scrollbar_drag`)
effects: none
via: `CommandBar`

## interface consumers
- none

## artifacts
- none

## tests
- `command-bar` (in-module `#[cfg(test)]`)
  - light
  - validates collapsed handle draw/expand toggle, screen-locked expanded layout under hud pan, border/handle/S draw, button clicks, nested menus, scrollbar draw/click/drag/wheel, hotspot rects, and palette-role honoring

## data
- none

## notes
- deferred module migration: making modules (this bar included) sticky-to-screen-edges via a `ScreenAnchor { edge, inset }` bound-module pattern is deliberately postponed until that design matures; the bar keeps its direct `update_layout` screen anchoring for now
