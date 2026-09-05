# /home/j/Repos/thaum-renderer/domain/camera/perspective

## purpose
Own the depth-to-screen perspective shaping math: how one depth unit maps to
glyph scale and screen-position spread under user-tunable perspective
strengths, including the near-camera saturation floor.

## owns
- `PerspectiveProfile`: per-camera perspective knobs (`scale_strength`,
  `position_strength`, `near_floor_fraction`)
- `depth_scale_factor`: glyph scale factor for one depth unit
- `depth_position_spread`: screen-position spread factor for one depth unit
- the sublinear depth easing and focal-distance tuning constants

## does not own
- projection orchestration (`domain/camera/projection/`)
- camera intent, swing, roll, or focus resolution (`domain/camera/`)
- UI panels that edit the profile (renderer modules opt into those)
- persistence of camera state (`domain/persistence/ui-session-state/`)

## children-encapsulations
- none

## contents
- `contract.md`
  - perspective contract
- `perspective.rs`
  - `PerspectiveProfile`, `depth_scale_factor`, `depth_position_spread`,
    `eased_signed_depth_units`

## dependencies
- `thaum-renderer` `CameraProjectionMode`

## tests
- `perspective`
  - light
  - validates that the default profile reproduces the historical shared
    factor exactly, that zero strengths give a fully straight-on look, that
    the floor fraction saturates near-camera growth, that removing the floor
    lets near glyphs keep growing, and that strengths above the default push
    more extreme than the old renderer.
