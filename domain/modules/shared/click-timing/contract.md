# thaum-renderer/domain/modules/shared/click-timing

## purpose
Own the shared double-click detection so modules stop re-rolling `Option<Recent…Click>` structs with a local 350 ms window and per-field equality checks (the painter layers panel grew two before this seam existed).

## owns
- the `DoubleClick<K>` detector: windowed double-click decision over any `Eq + Clone` click subject the caller picks (a layer id, a `(button, block-id, piece)` tuple, a cell coordinate)
- the canonical `DOUBLE_CLICK_WINDOW` (350 ms between clicks), matching the old modules' constant
- the triple-click rule: every click becomes the recent click, so a triple reads as double-click then double-click
- `clear()` for surfaces whose subject disappears or the pointer leaves

## does not own
- raw pointer event routing or capture (owned by `domain/modules/`'s `Module` contract)
- any clock — callers pass `Instant::now()` so the detector is pure state + timing and testable without sleeping
- single-click vs drag disambiguation — the owning module's pointer state machine decides that

## children-encapsulations
- none

## contents
- `click_timing.rs`
  - `DoubleClick`, `DOUBLE_CLICK_WINDOW`, plus inline tests

## dependencies
- none (std `time` only)

## exposed interfaces
- `DoubleClick`/`DOUBLE_CLICK_WINDOW`, described in this contract

## interface consumers
- `thaum-renderer/domain/modules/individuals/`
- `thaum-painter/domain/modules/individuals/`

## notes
- J 2026-09-12: born from the painter layers-panel deep dive — `RecentRasterClick` and `RecentNameClick` were the reference shapes; the seam generalizes both via the generic subject.
