# theme-roles audit

## purpose
Record where the proven UI-theme color system was found in the old code, what it actually did, and the design decision for the new `tools/theme-roles/` seam.

## source evidence
- `THAUMWORLD-AUTO-STORY-TELLER/src/mono_ui/runtime/ui_customization_store.ts`
- `THAUMWORLD-AUTO-STORY-TELLER/src/mono_ui/colors.ts`

## what the old system actually did
- defined a fixed set of named semantic UI color roles: `background`, `dimmest`, `medium`, `bright`, `vivid`, `right_hand`, `left_hand`
- each role held exactly one RGB value, defaulted from a canonical named color (e.g. `background` → `off_black`, `vivid` → `vivid_cyan`), and every saved/set value was snapped to the nearest color in a canonical indexed palette (`nearest_indexed_color`) rather than staying arbitrary RGB
- one setter per role (`set_ui_customization_role_color(role, rgb)`) mutated the single in-memory state and fired a change notification so every UI element referencing that role could react live
- dozens of UI modules across both the game (`mono_ui/modules/*`) and the old painter (`canvas_app/painter_app_state.ts`, `painter_canvas_module.ts`) read colors exclusively through `get_ui_semantic_rgb(role)` — never a hardcoded value — which is what made "change one, recolor everything" actually true
- persistence (save-slot JSON files, profile scoping) was layered on top of the same module, not a separate concern

## what carries over vs. what doesn't
- carries over: the named-role indirection, snap-to-canonical-palette behavior, single setter/getter pair, and change-notification expectation.
- carries over as a dependency, not a reimplementation: nearest-color snapping already has a home in `thaum-renderer/tools/color/` (`nearest-color-in-collection`); `tools/theme-roles/` should consume that rather than duplicate it.
- does not carry over: persistence/save-slot file I/O (that was game-specific profile/slot plumbing, not a rendering concern) and the browser `CustomEvent` change-notification mechanism (implementation detail of the old web runtime, not part of the contract shape).
- the specific 7 role names are carried over as the proven starting default, not necessarily a hard ceiling — see the new contract's notes.

## placement decision
`tools/theme-roles/` (new), sibling to `tools/color/`. It owns the generic named-slot indirection mechanism only — never which UI elements use a given role or what a role "means" to a menu, which stays consumer-owned (game, thaum-painter), matching how `tools/color/` already disclaims "app-specific authored color policy."
