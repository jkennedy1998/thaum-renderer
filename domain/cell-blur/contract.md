# /home/j/Repos/thaum-renderer/domain/cell-blur

## purpose
Own the renderer-facing per-cell blur vocabulary and output shape used when shaders drive cell-local blur-style art direction.

## owns
- the canonical cell-blur contract
- renderer-owned per-cell blur output expectations
- a single blur-amount control where `0` means no blur and larger values increase blur strength
- display-location-level blur expectations at the cell render surface
- the rule that shaders may control blur outputs without owning the blur seam itself
- the preference for the fastest practical blur interpretation because this control may run on thousands of cells
- the current planning assumption that blur is a cell-local visual control first and may move later only if testing proves that necessary

## does not own
- shader stack ownership
- warble ownership
- full-frame post-effect blur behavior
- app-specific tag semantics

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell/`
- `/home/j/Repos/thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- cell-blur shape
  - describes the per-cell blur outputs and renderer-facing expectations for cell-local blur-style visuals
  - current planned shape: one scalar blur amount with implementation-defined mapping from amount to the fastest practical blur behavior

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-shader/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- these blurs are intended to affect cells at display-location level rather than as simple screen-wide post effects
- blur is currently being captured at the same scale as warble because depth-of-field-like behavior and similar art-direction controls may need to happen per cell
- blur is intentionally one-dimensional for now: authors set an amount, and the renderer chooses the most practical blur curve or kernel behavior
- `0` should read as no blur; increasing values should increase blur magnitude without promising a fixed physical unit yet
- the current boundary is intentionally descriptive rather than performance-locked; real renderer testing should decide later whether every captured blur behavior remains viable per cell
