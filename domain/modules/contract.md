# /home/j/Repos/thaum-renderer/domain/modules

## purpose
Own the renderer's interactive UI-panel format: "module" means exactly one thing across the whole ecosystem — one grouped, bounded element meant to convey one piece of information or one interaction surface to the user (a color picker, a floating toolbar, one menu panel).

## owns
- the `Module` contract: identity, screen-space rect, draw-as-`CellGroup` output, pointer lifecycle, and the `wants_pointer_capture`/`is_closed` flags a module reports back
- module registry semantics: register/unregister, z-order, visibility, hit-test, ordered iteration, dispatching a pointer event to the topmost module under a point, pointer capture (continuous `Move`/`Up` delivery to one module regardless of hit-test, for drag sessions like gizmo move/resize), continuous hover tracking (`Enter`/`Leave` delivery as the topmost module under the pointer changes, independent of clicking), and purging modules that report themselves closed
- the rule that this domain is a utility other renderer code is not required to use — nothing in `domain/camera/`, `domain/composition/`, etc. depends on `domain/modules/`
- the rule that every consumer of thaum-renderer gets the same shape: its own `domain/modules/{shared/, individuals/}` mirroring this one, depending on this domain's shared logic rather than reinventing it
- the boundary for which individual modules are generic enough to live here (`individuals/`) versus app-specific ones that belong in a consumer's own `domain/modules/individuals/`

## does not own
- app-specific module content or business logic (a painter's color picker contents, a game's inventory panel)
- gameplay logic that depends on adjacency or simulation state
- interaction/keybinding policy, owned by consuming programs on top of `domain/controls/`
- raw input capture or the action-binding system, owned by `domain/controls/`

## children-encapsulations
- `shared/`
  - default
- `individuals/`
  - default

## contents
- `contract.md`
  - modules contract
- `module.rs`
  - rust `Module` shape and registry, owned by this encapsulation

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-group/`
- `/home/j/Repos/thaum-renderer/domain/camera/screen-world-remap/`
- `/home/j/Repos/thaum-renderer/domain/controls/`

## exposed interfaces
- module shape
  - describes the renderer-owned interactive UI-panel format and its registry

## interface consumers
- `thaum-painter/domain/modules/`
- `thaum-painter/orchestration/entrypoint/` — first real consumer boot proof, draws a registered `Module` in an actual window
- future game/other-program consumers of thaum-renderer
- future renderer implementation surfaces

## artifacts
- none

## tests
- `module.rs`'s inline `#[cfg(test)]` module
  - light
  - validates registry register/unregister, z-order (bring-to-front), rect-based hit-test, ordered iteration, pointer-event dispatch to the topmost module under a point, captured pointer move/up dispatch (including the no-capture no-op case), hover tracking (`update_hover_at` delivers `Enter` to the topmost module under a point, `Leave` when the point moves off it, is idempotent while the point stays over the same module, and moves hover cleanly between two modules), and that `remove_closed_modules` purges only modules reporting `is_closed() == true`.

## data
- none

## notes
- folder + `contract.md` only — same jobo-style encapsulation convention as every other `domain/` child, not a separate Cargo crate. A consumer either depends on `thaum-renderer-domain` (and gets this for free, unused unless called) or doesn't; there is no finer-grained opt-out.
- old-system precedent: `THAUMWORLD-AUTO-STORY-TELLER/src/mono_ui/` (`module_registry.ts`, `module_gizmos.ts`, 40+ `*_module.ts` files spanning painter and game). That proved the reuse pattern but also rolled its own `Canvas`/cell-set abstraction and an ad hoc `getLayerMode(): 'world_pannable' | 'screen_locked'` split that is now formalized for real as `CellGroupIntakeBehavior::Flat2d` on `CellGroup` — this domain should draw through that, not reinvent a parallel canvas.
- v1 scope was deliberately small at first: identity, rect, draw-as-`CellGroup`, pointer lifecycle, and registry z-order/hit-test/dispatch. Keyboard capture and focus are still `shared/`'s job once a real module needs them; drag sessions and gizmo move/resize behavior landed once `domain/modules/shared/module-gizmos/` needed real capture-based dispatch, not speculatively.
- module click events now carry which mouse button triggered them, because consumer modules like thaum-painter's toolbox need distinct left/right assignment semantics without bypassing the shared module seam.
- `dispatch_pointer_event_at`/`dispatch_captured_pointer_move`/`dispatch_captured_pointer_up` take plain cell-space `(x, y)`; converting a real OS click or drag position into that coordinate space is a consumer concern crossing `tools/window-surface` (raw clip-space cursor/click/pointer-down capture) and `domain/camera/screen-world-remap/` (clip-space to world/cell conversion) — this domain only owns what happens once that coordinate is known.
- pointer capture exists specifically so a gizmo drag keeps working once the cursor moves outside the module's own (possibly still-moving) rect; a consumer's frame loop is expected to call `dispatch_pointer_event_at` on a fresh click, then `dispatch_captured_pointer_move` every subsequent frame while the button stays held (`WindowSurfaceInput.pointer_down`), then `dispatch_captured_pointer_up` on release.
- `update_hover_at` exists independently of capture/click dispatch so a module can know it is moused over even when no button is held — `domain/modules/shared/module-gizmos/`'s seamless gizmo bar is the first real consumer, only drawing itself while hovered. A consumer's frame loop is expected to call it every frame with the current cursor position (converted to the same cell-space coordinates), regardless of whether a click or drag also happened that frame.
- `thaum-painter/domain/painter-document/groups/` is an unrelated concept (an authored content unit mapping to a `CellGroup`); see that repo's `context/module-concept-audit.md` for why "module" no longer names that.
