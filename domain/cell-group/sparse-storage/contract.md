# thaum-renderer/domain/cell-group/sparse-storage

## purpose
Own the canonical sparse storage contract used by a cell-group to store its cells efficiently across dense and sparse usage patterns.

## owns
- the cell-group-local sparse storage boundary
- stable storage semantics for large populated regions and wide sparse spans
- the contract surface where storage can be optimized without changing cell-group meaning

## does not own
- global renderer composition
- cell slot semantics
- the global coordinate space

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/cell-group/`
- `thaum-renderer/domain/cell/`

## exposed interfaces
- sparse storage shape
  - describes how a cell-group conceptually stores and retrieves contained cells without fixing implementation yet

## interface consumers
- future cell-group implementations

## artifacts
- none

## tests
- none

## data
- none

## notes
- this is a key rebuild target because old renderer bottlenecks lived here
- storage should stay owned by cell-group so all groups can optimize under one renderer contract
