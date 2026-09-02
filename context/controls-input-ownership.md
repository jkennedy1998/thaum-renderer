# controls/input ownership

## purpose
Record the refinement of truth-57 (`context/statement-truths.md`) reached in the 2026-08-31 operator session, and capture the resulting split between renderer-owned input infrastructure and app-owned interaction policy. Same role as thaum-painter's `context/module-concept-audit.md` "superseded" pattern: the quoted truth transcript stays untouched, this doc records how understanding of it moved.

## the truth being refined
- truth-57: "cursor logic is not camera ownership... get cursor logic out of here. Not relevant. It will downstream, but not native to the renderer."

Read narrowly, this could be taken to mean the renderer should own no input machinery at all, only expose screen/cell remap. Session source-of-truth from J narrows that back down:

> I don't think that the actual logic of the interactions needs to be in the renderer but I think that the renderer will need to expose the ability to get these inputs so the inputs can be built cleanly once in a great system that feels fluid and fast and then can be reimplemented in whatever programs use it. Kind of like how Unity does inputs.

## the resolved split
- **renderer-owned** (`domain/controls/`): raw input capture (pointer/keyboard/pressure), the action-binding/remap system (named action <-> physical input), pressure-sensitivity data, and a generic remap-UI module built on top of it. No default bindings shipped — programs declare and bind their own named actions.
- **app-owned**: what an action *means* (camera pan vs. brush stroke vs. UI click), keybinding choices/defaults for that program, and any interaction business logic downstream of an action firing.
- unchanged from truth-57: cursor *selection*/interaction *logic* is still not renderer ownership. What changed is that raw input capture and remapping infrastructure is now understood as renderer-owned plumbing underneath that logic, not something every consumer should hand-roll separately.

## why this matters now
- camera movement (pan/zoom/swing/roll) is a concrete case: `thaum-renderer/interfaces/camera/` already routes the movement operations, but *which physical key* triggers "camera target left relative step" is per-program (repo-space uses WASD+QE, thaum-painter used scroll+pan+space-drag, unevenly). A shared, remappable action-binding system removes the need for every consumer to build key-remap plumbing from scratch.
- `domain/modules/` (the UI-panel format) needs pointer/keyboard input to be interactive at all — gizmos, drag, click all sit on top of `domain/controls/`.
- pressure sensitivity is explicitly wanted by thaum-painter but is generic enough that any renderer consumer benefits from it living in one place.

## notes
- `domain/controls/` sits beside `domain/camera/` and `domain/modules/` as a sibling `domain/` child, same crate, same jobo-style folder+contract.md convention as the rest of `domain/` — no separate Cargo crate.
- the remap-UI module itself is generic enough (not app-specific) to live in `thaum-renderer/domain/modules/individuals/` as a real shipped module, not just a reference example — the app-specific/reference-only split noted for `domain/modules/individuals/` still applies to genuinely app-specific modules like a painter's color picker.
