# /home/j/Repos/thaum-renderer/domain/modules/individuals

## purpose
Own the concrete modules shipped directly by thaum-renderer, reserved for modules generic enough that every consumer benefits from one shared implementation rather than each program rebuilding its own.

## owns
- the boundary rule: a module lives here only if it is not app-specific (e.g. a controls remap panel), never business/content modules for one particular program
- individual module folders, one per module, each with its own `contract.md`

## does not own
- app-specific modules (a painter's color/character picker, a game's inventory panel) — those live in the consuming program's own `domain/modules/individuals/`
- the shared gizmo/chrome/registry logic those modules are built from, owned by `domain/modules/shared/`

## children-encapsulations
- `camera-perspective/`
  - default
- `color-picker/`
  - default
- `ui-customization/`
  - default

## contents
- `contract.md`
  - individuals contract

## dependencies
- `/home/j/Repos/thaum-renderer/domain/modules/`
- `/home/j/Repos/thaum-renderer/domain/modules/shared/`

## exposed interfaces
- `CameraPerspectiveModule`, described in `camera-perspective/contract.md`
- `ColorPickerModule`, described in `color-picker/contract.md`
- `UiCustomizationModule`, described in `ui-customization/contract.md`

## interface consumers
- future renderer implementation surfaces
- future consuming programs that opt into shipped-in modules
- `thaum-painter/orchestration/entrypoint/`

## artifacts
- none

## tests
- none directly here; see `color-picker/` and `ui-customization/`'s own tests

## data
- none

## notes
- `color-picker/` is the first module built here, since a grid color picker over the renderer's own canonical palette is generic across any consuming program.
- `ui-customization/` is the next generic panel: semantic UI role editing belongs in the renderer because every consumer module already draws from the same `UiPalette` role set.
- next candidate, not yet built: a controls remap panel over `domain/controls/`'s `ActionBindingMap` — generic across any consuming program, so it belongs here rather than in a per-program `individuals/`.
- consuming programs mirror this exact `domain/modules/{shared/, individuals/}` shape locally for their own app-specific modules; see `thaum-painter/domain/modules/`.
