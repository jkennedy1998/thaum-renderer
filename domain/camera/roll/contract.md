# /home/j/Repos/thaum-renderer/domain/camera/roll

## purpose
Own renderer camera roll semantics.

## owns
- roll meaning for the camera
- roll-oriented state and transition truth used by the renderer camera

## does not own
- generic helper math
- swing ownership
- app-specific input handling

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/camera/`
- `/home/j/Repos/thaum-renderer/domain/camera/view-orientation/`

## exposed interfaces
- camera roll shape
  - describes how the renderer camera rolls around its current view orientation

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
- design truth can live here even if low-level repeatable roll helpers later split into `tools/camera/`
