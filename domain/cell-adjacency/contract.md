# /home/j/Repos/thaum-renderer/domain/cell-adjacency

## purpose
Own the rendering-only adjacency shape consumed by shaders and other renderer-facing cell systems.

## owns
- the canonical renderer adjacency contract for cells
- a fast and dumb rendering-only adjacency view over neighboring cells
- adjacency data that may be consumed by shaders, and potentially by other renderer-facing systems later
- the rule that adjacency is optional and should only be resolved when something actually consumes it

## does not own
- gameplay logic
- app-side neighborhood reasoning
- authoritative simulation adjacency
- the denser connectivity systems that consumer programs may keep for their own mechanics
- cell-group composition ordering

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell/`

## exposed interfaces
- cell adjacency shape
  - describes the lightweight rendering-focused neighboring-cell data made available to renderer consumers like shaders

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
- adjacency here is for rendering only
- `connection` is the preferred renderer-facing naming for tile-neighbor style connective use
- consumer programs may own richer or denser connectivity outside renderer
- connective tiles should flow through renderer-owned adjacency only when the renderer needs that fast simplified view
