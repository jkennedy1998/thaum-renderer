# /home/j/Repos/thaum-renderer/domain/camera

## purpose
Own the renderer's camera: view orientation (swing/roll), projection to the
view plane, visible-plane stacking, and the user-tunable perspective shaping
applied while projecting.

## owns
- the `Camera` state: position, focus target, swing, roll, projection mode,
  perspective profile, visible-plane radius/offset, zoom, HUD pan offset
- projection of world points to view-plane (u, v, plane) coordinates and the
  inverse, for both rotating-3d and flat-2d intake behaviors
- visible-plane stack derivation around the focus plane
- depth-to-screen perspective shaping via `perspective/`
- camera swing/roll transitions between orientations

## does not own
- app-side camera intent or subject resolution (consumer domain, e.g.
  thaum-painter `domain/rendering/camera/`)
- which physical keys or mouse gestures move the camera (consumer controls /
  `domain/controls/`)
- persistence of camera UI state (`domain/persistence/ui-session-state/`)
- UI panels that edit camera settings (renderer modules, opt-in per app)

## children-encapsulations
- `perspective/`
  - `PerspectiveProfile` knobs and the depth-to-scale/position math
- `projection/`
  - view-plane projection and inverse, visible-plane stacks
- `roll/`
  - camera roll state
- `screen-world-remap/`
  - camera-unit to world-space remapping helpers
- `swing/`
  - camera swing state
- `view-orientation/`
  - world-to-view-relative mapping for swing/roll combinations

## contents
- `contract.md`
  - camera contract
- `camera.rs`
  - `Camera` state shape, defaults, zoom bounds, module registration

## dependencies
- `thaum-renderer` `coordinate-space` (`CellPoint`, `WorldPoint`),
  `GlobalDirection`

## tests
- camera behavior is validated inside each child encapsulation plus the
  projection tests over orientation-specific cases
