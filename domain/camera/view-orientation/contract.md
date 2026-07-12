# /home/j/Repos/thaum-renderer/domain/camera/view-orientation

## purpose
Own the orientation truth of the renderer camera view.

## owns
- principal-view semantics
- facing and view-direction semantics that belong to the camera view
- orientation truth shared by projection, roll, and swing

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
