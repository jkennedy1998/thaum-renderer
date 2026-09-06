# thaum-renderer/domain/camera/swing

## purpose
Own renderer camera swing semantics.

## owns
- swing meaning for the camera
- swing-oriented state and transition truth used by the renderer camera
- authored transitions between the six principal view families used to interpolate the displayed matrix

## does not own
- generic helper math
- roll ownership
- app-specific input handling

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/camera/`
- `thaum-renderer/domain/camera/view-orientation/`

## exposed interfaces
- camera swing shape
  - describes how the renderer camera swings between principal views and related orientations

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
- swing is camera-owned even if some low-level helper work later becomes reusable
- swing should follow the six authored view families rather than becoming fully free camera rotation as the default renderer truth
