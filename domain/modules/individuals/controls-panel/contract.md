# /home/j/Repos/thaum-renderer/domain/modules/individuals/controls-panel

## purpose
Own a generic big-list controls panel: one renderer-owned floating panel listing every declared action with its current binding, click-to-rebind capture, and inline conflict markers. Ships no binding behavior — the consumer wires the getters/setter, so the module works for any program's control map.

## owns
- the `ControlsPanelModule` type and its `Module` implementation
- the slot layout: one category header before each group of action rows, windowed by the scroll offset for both draw and hit-testing so what you click is what you see
- Flat2d-correct stacking: slot 0 renders at the content's screen-top row and later slots walk down the screen (larger local y is higher on screen)
- the never-detached-header rule: a section title is never the window's trailing visible slot; a header whose rows sit past the window edge is dropped from the window
- wheel scrolling through the shared `ScrollState` seam (`domain/modules/shared/scroll-state/`): one row per notch, clamped, unmoved offsets fall through to the host
- left-click toggling key-capture wait on the clicked action; right-click unbinding it
- keyboard-only capture via `capture_key` / `cancel_capture`; pointer and wheel bindings stay owned by their existing live paths
- gizmo-bar-backed move/close/resize/seamless behavior

## does not own
- the action set, its categories, or their order — the consumer supplies the `ControlActionRow` list
- the binding map itself — raw input capture or action-binding is `domain/controls/`
- what happens to a captured key after `capture_key` routes it — the consumer's setter decides
- delivering pointer/wheel events to the module — the consumer's registry/frame loop

## children-encapsulations
- none

## contents
- `controls_panel_module.rs`
  - `ControlsPanelModule`, `ControlActionRow`, and their inline tests

## dependencies
- `/home/j/Repos/thaum-renderer/domain/modules/`
- `/home/j/Repos/thaum-renderer/domain/modules/shared/` (PanelChrome, gizmos, ScrollState)

## exposed interfaces
- `ControlsPanelModule::new(id, rect, palette, rows, get_binding_label, get_conflicts, set_binding)`
- `waiting_action()` / `capture_key(label)` / `cancel_capture()`
- `ControlActionRow { action, label, category }`

## interface consumers
- `thaum-painter/orchestration/entrypoint/`
- future consumers wanting a remap panel over their own control map

## artifacts
- none

## tests
- inline `#[cfg(test)]` in `controls_panel_module.rs`
  - light
  - validates header-above-rows screen order, click-through what-is-drawn hit-testing, capture/wait/cancel routing, right-click unbind, shared-scroll wheel behavior with clamping, and the never-detached trailing header rule

## data
- none

## notes
- the original stacking drew slot 0 at increasing local y, which Flat2d renders upside down: every section title appeared below its section contents on screen. The 2025-xx pass flipped the stacking, added shared-seam scrolling, and pinned the no-detached-header rule.
