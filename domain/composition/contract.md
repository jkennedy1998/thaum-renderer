# /home/j/Repos/thaum-renderer/domain/composition

## purpose
Own the general composition pipeline of how cell-groups are combined over the shared renderer coordinate space.

## owns
- the composition flow that maps cell-groups onto renderer global coordinates
- overlap behavior as an artifact of composition
- render-order semantics that belong to composition rather than cell-groups themselves
- the design truth of how renderer output gets assembled from multiple groups

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
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-group/`
- `/home/j/Repos/thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- composition flow
  - describes how cell-groups are placed onto shared space and reduced into one renderer output

## interface consumers
- `/home/j/Repos/thaum-renderer/`
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
