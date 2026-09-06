# thaum-renderer/domain/cell-group

## purpose
Own the renderer cell-group shape as the layer-level matrix boundary that arranges cells on the global board or type grid.

## owns
- the canonical cell-group contract
- the local field or matrix of cells contained by the group
- cell-group size and xyz placement on the global board or type grid
- clipping of the group's own content
- transform-oriented group truth including facing and broader basis-oriented transforms
- the group-declared intake behavior that decides whether the group participates in rotating 3d intake or non-rotating 2d intake before shared projection/composition

## does not own
- renderer composition between multiple cell groups
- deep cell slot semantics for weight, color, graphic, shader, or material
- app-level painting semantics outside renderer layering needs

## children-encapsulations
- `transform/`
  - default
- `clipping/`
  - default
- `local-coordinate-space/`
  - default
- `sparse-storage/`
  - default

## contents
- `cell_group.rs`
  - rust cell-group shape, bounds, and traversal helpers owned by this encapsulation

## dependencies
- `thaum-renderer/domain/cell/`
- `thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- cell-group shape
  - describes the layer-level renderer unit that positions and bounds a field of cells
  - expected to carry matrix-oriented placement and clipping truth without taking over renderer composition

## interface consumers
- `thaum-renderer/domain/composition/`
- `thaum-renderer/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- cell-group is the layer shape in this renderer model
- facing is currently constrained to the six cardinal directions
- cell-group should know about its own matrix-oriented space, not how sibling groups are composited together
- the renderer should stay one renderer mode, while cell-groups can arrive through two intake behaviors: rotating 3d intake and non-rotating 2d intake
- non-rotating intake may still carry 3d shapes locally; it simply does not participate in perspective rotation like the fully rotating 3d intake path
- sparse storage is expected to be the canonical heavy-use storage path for group contents
