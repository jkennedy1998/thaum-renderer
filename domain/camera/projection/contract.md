# /home/j/Repos/thaum-renderer/domain/camera/projection

## purpose
Own camera projection semantics for renderer viewing.

## owns
- how camera view projects world or group space toward render space
- projection-facing contract truth that belongs to camera ownership rather than generic matrix tools
- world-to-view projection into view-relative plane coordinates such as `(u, v, plane)`
- visible plane-stack derivation from camera view state and focus target
- projected plane ordering and grouping semantics for renderer-visible plane stacks
- the shared perspective system used by both rotating 3d intake and non-rotating 2d intake
- the rule that 2d focus-plane landing and 3d focus-plane landing must align on the same screen type-grid positions

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
- `/home/j/Repos/thaum-renderer/domain/camera/view-orientation/`
- `/home/j/Repos/thaum-renderer/domain/coordinate-space/global-directions/`

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
- projection owns the derived plane-aware view model, but camera orientation and focus target remain the authoritative inputs
- visible plane stacks should be derived here rather than being independently invented by render consumers
- projection should preserve a legible focus plane and let nearby planes read depth mostly through perspective composition rather than becoming a separate renderer mode
- 2d and 3d content should pass through the same perspective rules even if they are rendered in separate passes
- the rotating 3d intake path may need matrix reinterpolation after view rotation, while the non-rotating 2d intake path stays still against the screen type grid
