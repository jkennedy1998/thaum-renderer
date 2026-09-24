# thaum-renderer/domain/camera/view-orientation

## purpose
Own the orientation truth of the renderer camera view.

## owns
- principal-view semantics
- facing and view-direction semantics that belong to the camera view
- orientation truth shared by projection, roll, and swing
- active depth-axis selection for the current camera view
- the derived plane basis used to interpret world axes as view-relative lateral, vertical, and depth directions
- the six authored camera view families used for display interpolation
- the camera-facing carrier and view-relative facing resolution consumed by facing-variant lookup
- the camera's full 24-element rotation carrier (swing + roll) for variant lookup

## does not own
- low-level helper math
- group-local facing ownership
- composition overlap rules

## children-encapsulations
- none

## contents
- `view_orientation.rs`
  - rust view-orientation truth: principal-view tables, the camera-facing
    carrier, relative-facing resolution, the camera rotation carrier, and
    view/world projection

## dependencies
- `thaum-renderer/domain/camera/`
- `thaum-renderer/domain/cell-facing/`
- `thaum-renderer/domain/coordinate-space/global-directions/`

## exposed interfaces
- camera view-orientation shape
  - describes the orientation state used by the renderer camera to interpret the world
- `camera_facing_for_swing(swing)` — the world side the camera views from, as a facing; the camera-facing carrier for relative-facing resolution
- `camera_rotation_for_camera(swing, roll)` — the camera's full orientation as one 24-element rotation: the unrolled view-from facing composed with its roll on the canonical side (complemented for the look-axis handedness; pinned by test)
- `view_relative_facing(swing, cell, group)` — the composed relative facing under this camera swing; `PosZ` means the viewer sees the cell's front
- `world_depth_along_direction(point, direction)` — signed world coordinate of a point along a direction's axis, honoring the direction sign; the depth read behind `Camera::focus_depth`

## interface consumers
- `thaum-renderer/domain/camera/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- this seam exists so orientation truth does not get lost inside projection or control helpers
- view orientation should decide which world axis is acting as depth for the current view without redefining renderer-global direction truth
- focus-plane derivation should depend on this seam together with the camera focus target xyz
- the intended view family is authored around six directions rather than unconstrained free rotation as the design truth
