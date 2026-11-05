# thaum-renderer/domain/cell-weight

## purpose
Own the renderer cell-weight slot as the canonical weight input consumed by a cell.

## owns
- the canonical cell-weight slot contract
- a variable-length, non-negative weight index with no engine-enforced ceiling (`ZERO`/`ONE`/`TWO`/`THREE` are named convenience values for the common first four stops, not an exhaustive set)
- resolution-time clamping into whatever a loaded typeface or sprite batch's native weight count actually supports (`clamped_to_max`), so a stored weight never panics or crashes when something with fewer native weights is swapped in
- weight semantics that differ between Thaum Mono character graphics and sprite graphics when needed
- the slot staying independent of any one typeface: weight is not owned by or derived from typeface identity, since sprites and other consumers share the same slot

## does not own
- cell position
- cell color behavior
- cell graphic selection
- renderer composition behavior

## children-encapsulations
- none

## contents
- `cell_weight.rs`
  - rust cell-weight slot values and clamp helpers owned by this encapsulation

## dependencies
- none

## exposed interfaces
- cell-weight slot
  - describes the weight slot consumed by a cell as a non-negative index with no fixed ceiling
  - `from_index_clamped` floors a signed index at zero (never panics); `clamped_to_max` snaps down to whatever a loaded typeface/sprite batch supports at resolution time
  - expected to normalize how weight is interpreted across supported graphic modes

## interface consumers
- `thaum-renderer/domain/cell/`
- `thaum-renderer/domain/cell-shader/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- the first known weight system is the four-weight Thaum Mono behavior; the slot itself no longer caps at four (project 04n3fq3e, J 2026-11-05: "no cieling, floor of 1 ideally")
- sprite graphics may need different weight handling without changing the slot boundary itself
- clamping against a specific typeface's or sprite batch's weight count is a resolution-time concern for that consumer (e.g. `cell-graphic/glyph/`), not something this slot tracks itself
