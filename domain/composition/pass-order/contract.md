# thaum-renderer/domain/composition/pass-order

## purpose
Own explicit pass ordering for renderer composition.

## owns
- explicit composition pass ordering
- the rule that pass order should be predictable and not rely on fallback creation order

## does not own
- cell-group local transforms
- camera viewing behavior
- adjacency logic

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/composition/`

## exposed interfaces
- pass-order shape
  - describes the explicit ordered composition stages used when reducing renderer content

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
- explicit only; no fallback ordering is desired
