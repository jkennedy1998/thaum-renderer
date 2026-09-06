# thaum-renderer/domain/modules/shared

## purpose
Own the shared implementation logic that every individual module builds on: gizmo behaviors, base chrome, and any registry/hit-test helpers that grow beyond `domain/modules/module.rs`'s v1 scope.

## owns
- gizmo behaviors (move, close, resize, seamless) as reusable pointer-driven interactions over a `Module` (`module-gizmos/`)
- base chrome primitives (container, floating-panel, button) that individual modules compose rather than rebuild
- shared hit-test/z-order helpers once they outgrow `domain/modules/module.rs`
- the shared semantic UI color-role palette (`ui-palette/`) that any module's chrome draws from
- the base ASCII-border/background/title panel chrome primitive (`panel-chrome/`) individual modules draw themselves with

## does not own
- the `Module` contract itself, and the pointer-capture dispatch that makes gizmo drag sessions actually reach a module, owned by `domain/modules/`
- any specific individual module's content
- input capture or action-binding, owned by `domain/controls/`

## children-encapsulations
- `ui-palette/`
  - default
- `panel-chrome/`
  - default
- `module-gizmos/`
  - default
- `scroll-state/`
  - default
- `tooltip/`
  - default

## contents
- `contract.md`
  - shared contract

## dependencies
- `thaum-renderer/domain/modules/`
- `thaum-renderer/domain/controls/`
- `thaum-renderer/domain/camera/screen-world-remap/`

## exposed interfaces
- `UiPalette`/`UiColorRole`, described in `ui-palette/contract.md`
- `PanelChrome`/`PanelBorderStyle`, described in `panel-chrome/contract.md`
- `GizmoBar`/`GizmoKind`/`GizmoState`, described in `module-gizmos/contract.md`
- `ScrollState`, described in `scroll-state/contract.md`
- `Hotspot`/`TooltipState`/`tooltip_card_group`, described in `tooltip/contract.md`

## interface consumers
- `thaum-renderer/domain/modules/individuals/`
- `thaum-painter/domain/modules/individuals/`
- future game/other-program consumers

## artifacts
- none

## tests
- `ui-palette/`'s own tests, described in its contract
- `panel-chrome/`'s own tests, described in its contract
- `module-gizmos/`'s own tests, described in its contract
- `scroll-state/`'s own tests, described in its contract

## data
- none

## notes
- the higher-level floating-panel/button chrome wrapper (border+background+gizmos+arbitrary content in one reusable `Module`, matching the old mono_ui `floating_panel_module.ts`) remains unbuilt as of this pass — `domain/modules/module.rs` v1 (identity, rect, draw, pointer lifecycle, registry) was deliberately built first without gizmos so the base `Module` shape stayed provable before interaction behaviors landed on top of it.
- `ui-palette/` was the first child built here, landing ahead of gizmos/chrome because a concrete module (`domain/modules/individuals/color-picker/`) needed a real highlight color rather than a hardcoded one.
- `panel-chrome/` and `module-gizmos/` landed together next, once `color-picker/` needed to actually look and behave like the old system's bordered, gizmo-driven panels rather than a bare block of swatches, per direct user feedback requesting real move/close/resize/seamless gizmos "so I can actually click and move things around and see how that is being processed in real time."
- `scroll-state/` was extracted from thaum-painter's graphic-picker, which proved the row-offset + pinned-block scroll pattern; it lands here so future scrollable panels (layers list, hand settings, presence lists) reuse one offset/clamp shape instead of copy-pasting it per module.
- `module-gizmos/`'s `GizmoState` does not consume `domain/controls/` itself — it is driven directly by `ModulePointerEvent`s a consumer's frame loop already dispatches through `domain/modules/`'s `ModuleRegistry` (including its pointer-capture path), so a raw-input layer between the two was not needed for this pass.
