# thaum-renderer/domain/controls

## purpose
Own the renderer's input capture and remappable action-binding infrastructure, so every consumer gets one clean, fast, remappable input system instead of hand-rolling its own.

## owns
- raw pointer, keyboard, and pressure input event shapes
- the named-action binding/remap system: action name <-> physical input, many-to-one and remappable
- pressure-sensitivity data as a first-class input shape
- the typing-mode input-focus gate (`typing-mode/`): while a session owns the input surface, inputs route to the session, declared reserved inputs stay live, and everything else is suppressed
- the rule that this domain ships no default bindings; consuming programs declare and bind their own named actions

## does not own
- what an action means to a consuming program (camera pan vs. brush stroke vs. UI click)
- any program's default keybindings
- interaction business logic downstream of an action firing
- cursor/selection semantics (see `domain/camera/screen-world-remap/` for screen<->cell conversion; that stays the camera's job)
- `domain/modules/` gizmo/drag interaction logic, which consumes this domain rather than owning it

## children-encapsulations
- `thaum-renderer/domain/controls/tai/`
- `thaum-renderer/domain/controls/typing-mode/`

## contents
- `contract.md`
  - controls contract
- `controls.rs`
  - rust input event shapes and the action-binding map, owned by this encapsulation
- `tai/`
  - tool-assisted input testing surface: script format, breath-timed replay runner, template, and growable individuals registry
- `typing-mode/`
  - input-focus gate for typing sessions: `TypingMode` + `TypingRoute`, reserved set declared at `begin` by the consuming program

## dependencies
- `thaum-renderer/domain/`

## exposed interfaces
- controls shape
  - describes renderer-owned raw input events, the action-binding map, and pressure data consumed by programs and by `domain/modules/`

## interface consumers
- `thaum-renderer/domain/modules/`
- `thaum-renderer/domain/camera/` (movement/swing/roll/zoom operations are triggered by app-bound actions, not owned here)
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- `controls.rs`'s inline `#[cfg(test)]` module
  - light
  - validates action binding/rebinding and resolution from raw input to named action.

## data
- none

## notes
- see `context/controls-input-ownership.md` for the truth-57 refinement this encapsulation is built from.
- "like Unity does inputs" is the reference shape from J: a generic action-based system with no built-in default action map.
- pressure sensitivity lives here rather than in a painter-only seam so any renderer consumer (painter, game, other programs) gets it for free.
- the remap-UI module built on top of this lives in `domain/modules/individuals/`, not here — this domain owns the data/binding system, not its own settings-panel chrome.
- `RawInput::Key` holds an owned `String` (not `&'static str`) so runtime-captured and TAI-scripted key codes can both flow through without leaking.
