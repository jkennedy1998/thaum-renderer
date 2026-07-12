# /home/j/Repos/thaum-renderer/domain/camera/projection

## purpose
Own camera projection semantics for renderer viewing.

## owns
- how camera view projects world or group space toward render space
- projection-facing contract truth that belongs to camera ownership rather than generic matrix tools

## does not own
- low-level reusable matrix math helpers
- app-specific raycasting logic
- camera roll or swing control semantics

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/camera/`

## exposed interfaces
- camera projection shape
  - describes the projection semantics owned by the renderer camera

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
- generic matrix math may still live in `tools/` later without moving projection ownership out of camera
