# thaum-renderer/domain/cell-group/clipping

## purpose
Own the clipping truth of a cell-group's own contents.

## owns
- local clipping bounds for a group
- rules for trimming or limiting the visible content of a cell-group before shared composition

## does not own
- sibling-group overlap resolution
- camera clipping
- shader override behavior

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/cell-group/`

## exposed interfaces
- cell-group clipping shape
  - describes how a group limits its own content before composition across the renderer space

## interface consumers
- `thaum-renderer/domain/cell-group/`
- `thaum-renderer/domain/composition/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- clipping belongs to the group before renderer-wide overlap or pass-order reasoning happens
