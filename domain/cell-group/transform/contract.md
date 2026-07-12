# /home/j/Repos/thaum-renderer/domain/cell-group/transform

## purpose
Own the transform truth of a cell-group over renderer global space.

## owns
- cell-group xyz placement
- cell-group xyz size bounds relevant to transform reasoning
- six-cardinal facing truth
- matrix-oriented transform semantics needed for group placement

## does not own
- global composition ordering
- camera motion
- sparse storage ownership

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-group/`
- `/home/j/Repos/thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- cell-group transform shape
  - describes how a cell-group is positioned and oriented in renderer global space

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-group/`
- `/home/j/Repos/thaum-renderer/domain/composition/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- current facing scope is limited to the six cardinal directions
