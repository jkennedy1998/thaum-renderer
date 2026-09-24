# thaum-renderer/domain/camera/presentation

## purpose
Own smooth presentation interpolation between the renderer's authored 24 camera frames.

## owns
- a one-step pre-snap/post-snap presentation transition for six swings and four rolls
- the visual-midpoint handoff to the next authored semantic frame
- residual presentation basis between the current semantic frame and its temporary display tilt

## does not own
- the six swing or four roll vocabularies
- depth projection or perspective shaping
- cell-facing resolution, app input bindings, or multiplayer actor/world-facing truth

## children-encapsulations
- none

## contents
- `presentation.rs`
  - authored-frame transition timing, midpoint semantic handoff, and residual-basis math
- `truth.md`
  - J's presentation-transition design truth

## dependencies
- `thaum-renderer/domain/camera/view-orientation/`
- `thaum-renderer/domain/camera/swing/`
- `thaum-renderer/domain/camera/roll/`

## exposed interfaces
- `CameraPresentation::begin` / `advance` — run one smooth 90-degree authored-frame transition and return its midpoint semantic handoff
- `CameraPresentation::residual_basis` — return the render-only transform from the semantic frame to its temporary display tilt

## interface consumers
- `thaum-renderer/domain/camera/`

## artifacts
- none

## tests
- a transition commits its semantic frame only at the visual midpoint
- post-snap tilt settles back to identity and active transitions refuse retargeting

## data
- none

## notes
- presentation is per viewer; only the existing `CameraSwing` and `CameraRoll` flow downstream.
- transition tilt is deliberately not a second perspective system: normal camera projection has already applied the world-depth perspective before this residual reaches the GPU.
