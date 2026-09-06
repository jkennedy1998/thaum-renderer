# thaum-renderer/domain/composition/overlap-policy

## purpose
Own the overlap policy used when composed cell-groups target the same renderer coordinates.

## owns
- exact-xyz overlap resolution semantics
- the rule that the last rendered cell at the same exact xyz coordinate wins
- composition-facing conflict resolution between otherwise separate groups

## does not own
- app gameplay collision logic
- local clipping inside a group
- camera semantics

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/composition/`

## exposed interfaces
- overlap-policy shape
  - describes how renderer composition resolves collisions when multiple composed cells occupy the same exact xyz coordinate

## interface consumers
- `thaum-renderer/domain/composition/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- this is a rendering policy, not gameplay logic
