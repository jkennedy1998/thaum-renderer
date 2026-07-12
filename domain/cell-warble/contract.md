# /home/j/Repos/thaum-renderer/domain/cell-warble

## purpose
Own the renderer-facing per-cell warble vocabulary and output shape used when shaders drive cell-local displacement-style art direction.

## owns
- the canonical cell-warble contract
- renderer-owned per-cell warble categories
- display-location-level warble/displacement expectations at the cell render surface
- the initial three warble families:
  - `texture-warble` - low-kernel noise style displacement
  - `fudge-warble` - medium-kernel noise style displacement
  - `distort-warble` - large-kernel noise style displacement
- the rule that shaders may control warble outputs without owning the warble seam itself

## does not own
- shader stack ownership
- blur ownership
- full-frame post-effect warble behavior
- app-specific tag semantics

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell/`
- `/home/j/Repos/thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- cell-warble shape
  - describes the per-cell warble outputs and renderer-facing expectations for cell-local displacement-style visuals

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
- these warbles are intended to affect cells at display-location level like texture displacement or vector displacement rather than as simple screen-wide post effects
- the current boundary is intentionally descriptive rather than performance-locked; real renderer testing should decide later whether every captured warble remains viable per cell
