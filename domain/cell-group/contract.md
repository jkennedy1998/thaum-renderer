# /home/j/Repos/thaum-renderer/domain/cell-group

## purpose
Own the renderer cell-group shape as the layer-level matrix boundary that arranges cells on the global board or type grid.

## owns
- the canonical cell-group contract
- the local field or matrix of cells contained by the group
- cell-group size and xyz placement on the global board or type grid
- clipping of the group's own content
- transform-oriented group truth including facing and broader basis-oriented transforms

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
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell/`
- `/home/j/Repos/thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- cell-group shape
  - describes the layer-level renderer unit that positions and bounds a field of cells
  - expected to carry matrix-oriented placement and clipping truth without taking over renderer composition

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/composition/`
- `/home/j/Repos/thaum-renderer/`
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
- sparse storage is expected to be the canonical heavy-use storage path for group contents
