# thaum-renderer/domain/cell-group/transform

## purpose
Own the transform truth of a cell-group over renderer global space.

## owns
- cell-group xyz placement
- cell-group xyz size bounds relevant to transform reasoning
- six-cardinal facing truth
- matrix-oriented transform semantics needed for group placement
- intake-behavior truth for whether a group participates in rotating 3d world reorientation or remains still as non-rotating 2d intake

## does not own
- global composition ordering
- camera motion
- sparse storage ownership

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/cell-group/`
- `thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- cell-group transform shape
  - describes how a cell-group is positioned and oriented in renderer global space

## interface consumers
- `thaum-renderer/domain/cell-group/`
- `thaum-renderer/domain/composition/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- current facing scope is limited to the six cardinal directions
- fully rotating 3d intake should respect both the group-facing directions and the world-coordinate rotation implied by camera/view interpolation
- non-rotating 2d intake should stay compatible with the same world placement and focus-plane alignment rules without rotating through the authored view interpolation
