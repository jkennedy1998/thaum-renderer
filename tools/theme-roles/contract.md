# /home/j/Repos/thaum-renderer/tools/theme-roles

## purpose
Own the reusable named-slot color-indirection mechanism ("theme") that lets a consumer bind a small set of semantic UI color roles to concrete colors from one single source of truth, so changing one role recolors every UI element that references it.

## owns
- the semantic-role registry shape: named role → one bound color value
- the default role set proven by the old system: `background`, `dimmest`, `medium`, `bright`, `vivid`, `right_hand`, `left_hand`
- get/set of the bound color for one named role
- snapping a bound color to the nearest canonical color via `tools/color/`'s nearest-color helper, so theme colors stay inside the same indexed palette discipline as the rest of the renderer's color system
- the expectation that consumers get notified when a role's bound color changes, so referencing UI elements can react live

## does not own
- which UI elements or modules reference which role (consumer/app-owned, see `context/theme-roles-audit.md`)
- what a role "means" to a menu or module (app-owned, matches how the renderer stays ignorant of module semantics)
- persistence of theme state across sessions/save-slots (consumer-owned; the old system's save-slot JSON plumbing was game-specific, not a rendering concern)
- per-cell rendered color resolution, owned by `domain/cell-color/`
- nearest-color matching itself, owned by `tools/color/`
- any specific change-notification transport (event bus, callback, etc.) — only that a change is observable

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/tools/color/`

## exposed interfaces
- theme role state
  - describes the single source of truth mapping each named semantic role to its currently bound color
- get role color
  - describes reading the bound color for one named role
- set role color
  - describes rebinding one named role's color, snapped to the nearest canonical color, with the expectation that this is observable by consumers

## interface consumers
- future thaum-painter UI
- future thaumworld game UI
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- this seam exists because the old system proved this exact shape works well across two different consumers (the old game's `mono_ui` and the old painter's own UI) for years; do not redesign the shape, port it.
- the default 7-role set is the proven starting point, not necessarily final — only extend it once a real consumer needs a role that doesn't fit, matching this whole project's "don't invent seams ahead of pressure" discipline.
- this stays a `tools/` seam, not a `domain/` seam, because it is a generic reusable mechanism with zero knowledge of UI/module semantics — see `context/theme-roles-audit.md` for the full reasoning and old-code source.
