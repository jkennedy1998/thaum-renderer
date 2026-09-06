# thaum-renderer/domain/composition

## purpose
Own the general composition pipeline of how cell-groups are combined over the shared renderer coordinate space.

## owns
- the composition flow that maps cell-groups onto renderer global coordinates
- overlap behavior as an artifact of composition
- render-order semantics that belong to composition rather than cell-groups themselves
- the design truth of how renderer output gets assembled from multiple groups
- the multi-pass assembly rule that lets rotating 3d intake and non-rotating 2d intake be rendered separately while still landing on one shared screen composition surface

## does not own
- cell-group local storage
- camera-local viewing semantics
- app gameplay logic
- boot ownership

## children-encapsulations
- `pass-order/`
  - default
- `overlap-policy/`
  - default

## contents
- `composition.rs`
  - rust composition flow and composed-cell reduction helpers owned by this encapsulation

## dependencies
- `thaum-renderer/domain/cell-group/`
- `thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- composition flow
  - describes how cell-groups are placed onto shared space and reduced into one renderer output

## interface consumers
- `thaum-renderer/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- composition is the home of render ordering because overlap is an artifact of shared-space composition
- last rendered cell on an exact xyz coordinate wins
- no fallback ordering is desired
- 2d and 3d content that resolve to the same global point should first be made to align on the same screen landing before any aggressive culling or cleanup policy is introduced
- 2d type-grid planes are intended to remain visible rather than being culled like fully 3d space; distance culling is more appropriate for fully rotating 3d cell-groups
