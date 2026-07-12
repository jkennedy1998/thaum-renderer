# /home/j/Repos/thaum-renderer/domain/coordinate-space

## purpose
Own the renderer-global coordinate space used to place, traverse, and shade cell-groups together.

## owns
- the canonical global renderer coordinate contract
- shared world xyz placement semantics across cell-groups
- coordinate semantics needed for global shader inputs and screen-space derivation
- the standard naming split between renderer world xyz and cell-group local xyz

## does not own
- the local internal coordinate space of a cell-group
- camera motion semantics
- sparse storage mechanics

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/`

## exposed interfaces
- global coordinate space
  - describes the renderer-owned world coordinate system used by cell-groups and camera-facing render logic

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-group/`
- `/home/j/Repos/thaum-renderer/domain/camera/`
- future renderer shader execution surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- cell-groups own local xyz; renderer owns the shared world xyz coordinate space
- older stopgap names like `world_z` should not be used as the primary coordinate shape now that full xyz is standard
- this space should be traversable and stable enough for world-space and screen-space shader inputs
