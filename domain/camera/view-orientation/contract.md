# /home/j/Repos/thaum-renderer/domain/camera/view-orientation

## purpose
Own the orientation truth of the renderer camera view.

## owns
- principal-view semantics
- facing and view-direction semantics that belong to the camera view
- orientation truth shared by projection, roll, and swing
- active depth-axis selection for the current camera view
- the derived plane basis used to interpret world axes as view-relative lateral, vertical, and depth directions
- the six authored camera view families used for display interpolation

## does not own
- low-level helper math
- group-local facing ownership
- composition overlap rules

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/camera/`
- `/home/j/Repos/thaum-renderer/domain/coordinate-space/global-directions/`

## exposed interfaces
- camera view-orientation shape
  - describes the orientation state used by the renderer camera to interpret the world

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/camera/`
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
