# /home/j/Repos/thaum-renderer/domain/camera/swing

## purpose
Own renderer camera swing semantics.

## owns
- swing meaning for the camera
- swing-oriented state and transition truth used by the renderer camera

## does not own
- generic helper math
- roll ownership
- app-specific input handling

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/camera/`
- `/home/j/Repos/thaum-renderer/domain/camera/view-orientation/`

## exposed interfaces
- camera swing shape
  - describes how the renderer camera swings between principal views and related orientations

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
- swing is camera-owned even if some low-level helper work later becomes reusable
