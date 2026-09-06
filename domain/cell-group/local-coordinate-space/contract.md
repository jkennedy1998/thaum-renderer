# thaum-renderer/domain/cell-group/local-coordinate-space

## purpose
Own the local coordinate space used inside a cell-group.

## owns
- the cell-group-local coordinate system
- the boundary between local cell coordinates and renderer global coordinates
- local positioning truth used by modules or other group-local users

## does not own
- renderer global coordinate ownership
- group-to-group composition ordering
- camera ownership

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/cell-group/`
- `thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- local coordinate space shape
  - describes the local spatial frame used by cells within a cell-group before composition into renderer global space

## interface consumers
- `thaum-renderer/domain/cell-group/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- local coordinates are group-owned even though renderer owns the shared global coordinate space
