# thaum-renderer/domain/cell-facing

## purpose
Own the six-cardinal facing vocabulary and the facing composition algebra that lets camera, cell-group, and cell facing combine into one relative-facing resolution.

## owns
- the canonical cell-facing value type over the six cardinal directions
- the facing rotation algebra: applying and inverting a facing's rotation onto directions and points
- the additive relative-facing resolution: camera-inverse composed with group facing composed with cell facing
- the mapping between the facing vocabulary and renderer global direction language
- the pure point-remap math that cell-group positioning delegates to

## does not own
- where the facing values are stored (cell slots, cell-group fields, camera view state)
- the camera view orientation semantics that produce the camera-facing input
- which art or variant a facing resolves to (cell-graphic facing-variant territory)
- group placement, origin, or world positioning beyond the pure point remap

## children-encapsulations
- none

## contents
- `cell_facing.rs`
  - rust facing vocabulary, the roll vocabulary, the 24-element FacingRotation
    group (compose/inverse/relative_rotation), and exhaustive group-law tests

## dependencies
- `thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- cell-facing, roll, and rotation value types
  - the six-cardinal facing type, the four-roll type, and the full
    cube-rotation type plus rotate/invert/remap operations over them
- relative-facing resolution
  - composes camera-inverse with group facing with cell facing into one
    relative-facing result for variant lookup
- relative-rotation resolution
  - `FacingRotation::relative_rotation` composes the full 24-element chain;
    its facing component recovers the facing-only resolution

## interface consumers
- `thaum-renderer/domain/cell/`
- `thaum-renderer/domain/cell-group/`
- `thaum-renderer/domain/camera/`
- `thaum-renderer/domain/cell-graphic/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- exhaustive facing direction-action roundtrips (light)

## data
- none

## notes
- facing values denote the world direction a local front (+z) maps to; PosZ is the identity
- the full rotation group materializes roll: `FacingRotation` (facing x roll) is the 24-element cube rotation group, resolved by matching composed actions on all six directions; facing-only resolution is the roll-Deg0 quotient of it
- the camera-facing input convention (which world direction counts as the view forward) is decided by camera view-orientation, not here
- per-cell facing fields, cell-group facing fields, and variant lookup all consume this one algebra so UI seams stay single-sourced
