# /home/j/Repos/thaum-renderer/domain/controls/tai

## purpose
Own the renderer's tool-assisted input (TAI) infrastructure: the script format and the breath-timed replay runner, so consumers of the renderer can own and run their own TAI tests against their own declared bindings instead of inventing per-app input-replay machinery.

## owns
- the TAI script JSON format (breath-indexed synthetic inputs + expected named-action fires)
- the runner that parses scripts and replays them through `ActionBindingMap`, reporting fires and expectation misses
- the copyable script template in `template/`
- the rule that individual TAI scripts and their registries belong to consuming programs, not to the renderer

## does not own
- the input shapes or the binding map themselves (see parent `../contract.md`)
- what a named action means to a consuming program
- individual TAI scripts, per-consumer registries, or per-consumer assertion helpers — consumers own those (see thaum-painter's `domain/tai/`)
- injection into a live OS window / real input capture — the runner is synthetic-input only
- default bindings; scripts assert against bindings the consuming program declares

## children-encapsulations
- none

## contents
- `tai.rs`
  - script parsing (`TaiScript::parse`), breath-timed replay (`TaiScript::run`), and run reporting (`TaiRunReport`)
- `contract.md`
  - this contract
- `template/script.json`
  - copyable starting script for a consumer's first TAI

## script format
```json
{
  "id": "snake_case_id",
  "description": "what this TAI verifies",
  "actions": [
    { "at": 1, "input": { "type": "key", "code": "A" } },
    { "at": 3, "input": { "type": "mouse_button", "button": 0 } },
    { "at": 5, "input": { "type": "wheel_up" } }
  ],
  "expect": [
    { "at": 1, "action": "the_named_action_that_should_fire" }
  ]
}
```
- `at` is a breath index (deterministic, no realtime clock — the renderer runs on breaths, so TAIs do too)
- inputs are the four raw shapes controls owns: key, mouse button, wheel up/down
- `expect` entries must be satisfied exactly once at their breath or the run fails
- an input bound to no action fires nothing and is not an error; assert silence by simply not expecting

## consumer growth path
1. consumer copies `template/` into its own tai seam and fills in actions + expectations
2. consumer keeps its own `individuals/` folder + `registry.json` and its own bindings declaration
3. this runner only changes when the script format genuinely needs a new input shape or expectation kind — never per-consumer-test

## dependencies
- `/home/j/Repos/thaum-renderer/domain/controls/` (ActionBindingMap, RawInput)
- serde + serde_json (workspace deps)

## exposed interfaces
- tai runner
  - describes `TaiScript::parse`, `TaiScript::run`, `TaiInput`, `TaiExpectation`, `TaiRunReport` consumed by consuming-program TAI seams

## interface consumers
- `/home/j/Repos/thaum-painter/domain/tai/` (painter-owned individuals, registry, and painter bindings)
- `tai.rs` inline tests

## artifacts
- none

## tests
- `tai.rs` inline `#[cfg(test)]` module
  - light
  - validates parsing of the template and synthetic scripts, breath replay through bindings, expectation pass/fail, out-of-range breaths, and unbound-input silence.

## data
- none

## notes
- Inspired by the old system's TAIs (`THAUMWORLD-AUTO-STORY-TELLER/local_data/tool_assisted_inputs/`), redesigned: one clock (breaths, not realtime_ms), app-agnostic expectations (action fires, not baked-in app asserts), and the format owned next to the input system it drives.
- Ownership split (J): renderer owns TAI infra; consumers of the renderer own the actual tests. Two seams total — this infra seam, plus one consumer seam per program (first: thaum-painter).
- The old system's per-app assert zoo is deliberately not carried over; when a consuming program needs richer assertions it wires its own harness on top of `TaiScript::run`'s fired-log.
