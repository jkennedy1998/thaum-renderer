# /home/j/Repos/thaum-renderer/domain/camera/parallax

## purpose
Own the mouse-driven view parallax shaping: a user-tunable toggle and
strength plus the host-fed pointer offset, and the per-depth screen shift
that spreads the view slightly around the focus plane as the pointer moves
across the whole screen.

## owns
- `ParallaxProfile`: `enabled`, `strength`, host-fed clip-space `offset`
- `parallax_screen_offset`: per-depth view-plane shift under the profile
- the parallax strength range and default tuning constants

## does not own
- the pointer itself; hosts feed `offset` every frame from their input
- depth easing and perspective strengths (`domain/camera/perspective/`)
- projection orchestration (`domain/camera/projection/`)
- UI panels that edit the profile (renderer modules opt into those)
- persistence of camera state (`domain/persistence/ui-session-state/`)

## children-encapsulations
- none

## contents
- `contract.md`
  - parallax contract
- `parallax.rs`
  - `ParallaxProfile`, `parallax_screen_offset`, strength constants

## dependencies
- `thaum-renderer` `CameraProjectionMode`
- `thaum-renderer` `domain/camera/perspective/` `eased_signed_depth_units`

## tests
- `parallax`
  - light
  - validates that disabled/orthographic/centered states never shift, that
    the focus plane never moves, that near planes drift toward the pointer
    while far planes drift away, and that strength clamping stays in range.
