# thaum-renderer/domain/modules/shared/text-cells

## purpose
Own the shared glyph-run text push so every module lays text into a cell list through one helper instead of re-rolling the `chars().enumerate()` loop per panel (copies had drifted, most silently dropping the weight and leaving text at invisible weight-0 chrome).

## owns
- `push_text_cells`: one left-to-right glyph run into a `Vec<Cell>`, with explicit color, explicit `CellWeight` (weight 0 is never a silent default), and an inclusive `max_x` column clamp
- the pushed-glyph count as the return value, so callers measure truncation without re-counting

## does not own
- drawing or rect truth — the owning module decides where the run lands
- hit-testing, hotspots, or tooltips — the owning module declares those separately
- text entry state (`../text-entry/`) or canvas typing sessions

## children-encapsulations
- none

## contents
- `text_cells.rs`
  - `push_text_cells` plus inline tests

## dependencies
- `thaum-renderer/domain/cell/`
- `thaum-renderer/domain/cell-graphic/`
- `thaum-renderer/domain/cell-weight/`
- `thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- `push_text_cells`, described in this contract

## interface consumers
- `thaum-renderer/domain/modules/individuals/`
- `thaum-painter/domain/modules/individuals/`

## notes
- J 2026-09-12: born from the painter layers-panel weight pass — the panel's `push_text` closure was the reference shape; the seam generalizes it so the next weight fix is a one-liner everywhere.
