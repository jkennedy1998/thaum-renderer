# thaum-renderer/domain/modules/shared/module-gizmos

## purpose
Own the shared move/close/resize/seamless gizmo bar every module can offer in its top-left corner — the renderer's port of the old mono_ui `module_gizmos.ts`. This is the "gizmo behaviors" piece `domain/modules/shared/contract.md` and `domain/modules/contract.md` had documented as unbuilt.

## owns
- the `GizmoKind` enum (`Move`, `Close`, `Resize`, `Seamless`) and its glyph (`#`, `X`, `╋`, `S`)
- the `GizmoBar` type: which gizmos a module offers, their fixed top-left layout (one cell in from the left corner, two cells apart), drawing them as `Cell`s, and hit-testing a click against them
- the `GizmoState` type: per-module move/resize/seamless toggle state, hover state, and the pointer-capture drag session (origin, original rect, live `drag_rect`) that lets a module reposition or resize itself as the pointer moves
- the `GizmoClickOutcome`/`ResizeEdge` types describing what a click into the gizmo interaction area resolved to
- the fixed drag rules: move translates the whole rect by the pointer delta; resize arms on a gizmo click, then a separate click grabbing one of the panel's four border edges drags only that edge (`x0`, `x1`, `y0`, or `y1`), each independently clamped to a minimum size; seamless and close have no drag session, just an immediate toggle/report
- `should_draw_gizmo_bar()`: whether the bar itself should currently draw — always true when not seamless; only while hovered, when seamless

## does not own
- drawing the border/background/title a gizmo bar sits on top of, owned by `domain/modules/shared/panel-chrome/`
- delivering real OS pointer movement or hover to a module — that is `ModuleRegistry::dispatch_captured_pointer_move`/`_up`/`update_hover_at` (`domain/modules/`), driven by a consumer's frame loop
- what "close" actually does to a module (hiding it, removing it from a registry) — a `GizmoState` only reports that the close gizmo was clicked; the owning module decides what its own `Module::is_closed()` should return
- deciding whether the border/background/title (not the gizmo bar) reappear on hover while seamless — that stays each module's own choice via `GizmoState::is_seamless()`, unaffected by hover
- the `save_position` gizmo from the old system — no consumer has asked for it

## children-encapsulations
- none

## contents
- `module_gizmos.rs`
  - `GizmoKind`, `GizmoBar`, `GizmoState`, `GizmoClickOutcome`, `ResizeEdge`, and their inline tests

## dependencies
- `thaum-renderer/domain/modules/`
- `thaum-renderer/domain/modules/shared/ui-palette/`
- `thaum-renderer/domain/cell-graphic/`

## exposed interfaces
- `GizmoBar::new(kinds)` / `title_start_x()` / `cells(rect, state, palette)` / `hit_test(rect, x, y)` / `hotspots(rect)` — `hotspots` is the first producer for the shared tooltip seam (`../tooltip/`): one single-cell anchor per gizmo glyph with title + description copy, surfaced by every gizmo-enabled module through `Module::hotspots()`
- `GizmoState::new()` / `is_seamless()` / `is_hovered()` / `set_hovered(bool)` / `should_draw_gizmo_bar()` / `wants_pointer_capture()` / `handle_click(bar, rect, x, y) -> Option<GizmoClickOutcome>` / `drag_rect(x, y)` / `end_drag()`

## interface consumers
- `thaum-renderer/domain/modules/individuals/color-picker/`
- future gizmo-enabled modules in `domain/modules/individuals/` and consuming programs' own `domain/modules/individuals/`

## artifacts
- none

## tests
- `module_gizmos.rs`'s inline `#[cfg(test)]` module
  - light
  - validates `title_start_x` clears every gizmo glyph and its gap, hit-testing finds each gizmo by its two-cell spacing, clicking move arms it and requests capture (with a drag translating the rect by the pointer delta), clicking move again with no drag in flight releases capture, clicking resize only arms it without starting a drag, grabbing each of the four border edges once armed resizes only that edge (min-size clamped) and a non-edge click while armed grabs nothing, ending a resize drag keeps resize armed for another edge grab, seamless toggles immediately without a drag session, `should_draw_gizmo_bar` stays true regardless of hover when not seamless but is hover-gated when seamless, close is reported to the caller without changing gizmo state, and a click outside the bar is ignored.

## data
- none

## notes
- old-system precedent: `THAUMWORLD-AUTO-STORY-TELLER/src/mono_ui/module_gizmos.ts`. This now ports both pieces that were originally deferred: per-edge resize (`get_resize_edge`/`handle_resize_drag`'s four independent branches, one `MIN_DRAG_SIZE` clamp per edge instead of the old system's separate min/max per axis) and hover-gated seamless chrome (a simplification of `should_draw_module_chrome`'s hover/active check — this version only gates the gizmo bar on hover; the border/background/title stay hidden while seamless regardless of hover, since the user's actual request was specifically about the gizmos reappearing/disappearing, not the border).
- resize is now a real two-step interaction matching the old system exactly: clicking the resize gizmo only arms resize mode (no drag starts yet); the user must then click-and-drag one of the panel's own four border edges to actually resize it from that edge. A click anywhere else while armed (interior content, for example) falls through to the module's own content handling untouched.
- `GizmoState::is_hovered`/`set_hovered` exist so a module can answer `should_draw_gizmo_bar()` from its own `draw()`; the hover signal itself comes from `Module::on_pointer_event(Enter/Leave)`, which `ModuleRegistry::update_hover_at` (`domain/modules/`) dispatches every frame based on continuous cursor position, independent of clicking or dragging.
- this and `panel-chrome/` landed together because a concrete consumer (`domain/modules/individuals/color-picker/`) needed real move/close/resize/seamless gizmos, not just chrome, per direct user feedback: "in the old painter UI I had the box and then gizmos and then the name... move close and resize... and silence [seamless]... so I can actually click and move things around and see how that is being processed in real time." The per-edge resize and hover-gated seamless bar landed in the very next pass, per further direct feedback: "the behavior is I click the resize it toggles and then I have to drag the top... the bottom... the left... the right" and "when I hit silence... the gizmos should only be visible when the user is mousing over the box."
- `GizmoState`'s pointer-capture flag (`wants_pointer_capture`) is read by `Module::wants_pointer_capture` on the owning module; `ModuleRegistry` (`domain/modules/`) is what actually turns that into real captured `Move`/`Up` dispatch once a consumer's frame loop feeds it continuous cursor position while the button is held.
