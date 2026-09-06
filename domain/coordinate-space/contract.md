# thaum-renderer/domain/coordinate-space

## purpose
Own the renderer-global coordinate space used to place, traverse, and shade cell-groups together.

## owns
- the canonical global renderer coordinate contract
- shared world xyz placement semantics across cell-groups
- coordinate semantics needed for global shader inputs and screen-space derivation
- the standard naming split between renderer world xyz and cell-group local xyz
- canonical global direction truth over renderer world axes
- the renderer-owned mapping between world xyz axes and global cardinal direction language

## does not own
- the local internal coordinate space of a cell-group
- camera motion semantics
- sparse storage mechanics

## children-encapsulations
- `global-directions/`
  - default

## contents
- `coordinate_space.rs`
  - rust world and local coordinate shapes owned by this encapsulation
- `global-directions/`
  - renderer-global cardinal direction contract over world xyz

## dependencies
- `thaum-renderer/domain/`

## exposed interfaces
- global coordinate space
  - describes the renderer-owned world coordinate system used by cell-groups and camera-facing render logic

## interface consumers
- `thaum-renderer/domain/cell-group/`
- `thaum-renderer/domain/camera/`
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
- global directions like top, bottom, north, east, south, and west should resolve from renderer-owned world-axis truth here rather than becoming camera-owned names
- camera view orientation may reinterpret which world axis is acting as depth for the current view, but it should not redefine the renderer-global direction language
- 2d and 3d intake paths should remain compatible through this shared world space even if their pre-projection rotation behavior differs
