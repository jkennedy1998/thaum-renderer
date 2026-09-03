# /home/j/Repos/thaum-renderer/domain/controls/typing-mode

## purpose
Own the renderer-level input-focus gate for typing sessions: while one
session owns the input surface, inputs route to the session, declared
reserved inputs stay live, and everything else is suppressed.

## owns
- the `TypingMode` gate state (active flag + reserved set)
- the `TypingRoute` routing decision: reserved / owned / suppressed for one
  raw input while a session is active
- the rule that unreserved keys route to the session and pointer/wheel
  input is suppressed while typing mode owns the surface

## does not own
- key semantics inside a session (chars/Enter/Backspace/cursor — the
  consuming program's text-entry state)
- which inputs are reserved: the consuming program declares the reserved
  set at `begin`, matching the controls rule that no default bindings ship
- the session state itself (`TypingMode` knows nothing about what the owner
  does with owned inputs)
- module/input plumbing: consumers gate their own dispatch paths on
  `is_active`/`route`

## children-encapsulations
- none

## contents
- `contract.md`
  - typing-mode contract
- `typing_mode.rs`
  - `TypingMode`, `TypingRoute`, and the routing decision

## dependencies
- `/home/j/Repos/thaum-renderer/domain/controls/`

## exposed interfaces
### typing-mode gate
send: `begin(reserved inputs)` when a session starts, one `RawInput` per
input while active, `end()` when the session finishes
returns: `is_active` and one `TypingRoute` per input
effects: none (pure focus state; consumers gate their own dispatch)
via: `TypingMode::begin`, `route`, `is_active`, `reserved`, `end`

## interface consumers
- `/home/j/Repos/thaum-painter/orchestration/entrypoint/` (text tool typing
  sessions reserve the camera/depth bindings and suppress everything else)
- future renderer text-entry surfaces (command bar text input, modal fields)

## artifacts
- none

## tests
- `typing_mode.rs` inline `#[cfg(test)]` module
  - reserved inputs stay live, unreserved keys route to the session,
    pointer/wheel suppressed unless declared reserved, `end` clears state,
    `begin` replaces a previous session's reserved set

## data
- none

## notes
- Grew out of the painter text tool: its hand-rolled suppression leaked held
  pan keys, deadened the reserved camera keys, and left pointer/wheel input
  live while typing. The unified seam moves the routing decision into the
  renderer so any consumer gets the same focus behavior.
- `RawInput`'s `Hash`/`Eq` make the reserved set a plain `HashSet`; remapped
  bindings flow in unchanged because the consumer declares whatever its
  current binding map says.
