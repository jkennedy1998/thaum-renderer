# /home/j/Repos/thaum-renderer/domain/camera

## purpose
Own the renderer camera shape that defines what part of the global coordinate space is being viewed.

## owns
- the canonical renderer camera contract
- camera position on the global coordinate space
- camera-owned focus target as one global xyz coordinate
- camera-facing view semantics like pan, roll, swing, six-authored-view interpolation, view-relative depth behavior, and projection ownership
- the single source of truth for the camera depth anchor through the focus target xyz
- derived focus-plane truth through owned camera view state and focus target xyz
- the shared perspective truth consumed by both rotating 3d intake and non-rotating 2d intake

## does not own
- module-local panning inside app-level content
- reusable helper calculations better suited for `tools/camera/`
- authored scene logic outside renderer viewing

## children-encapsulations
- `projection/`
  - default
- `view-orientation/`
  - default
- `roll/`
  - default
- `swing/`
  - default
- `screen-world-remap/`
  - default

## contents
- `camera.rs`
  - rust camera shape and projection-facing helpers owned by this encapsulation

## dependencies
- `/home/j/Repos/thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- camera shape
  - describes the renderer-owned camera state consumed to determine view placement and orientation over the global coordinate space
- `/home/j/Repos/thaum-renderer/interfaces/camera/contract.md`
  - routed camera-control interface notes for consumers and proof apps

## interface consumers
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- the renderer needs a camera to render
- the camera should be the single source of truth for the focused point in world space
- the old `focus_world_z` wording is not the right long-term name; the real owned thing is the camera focus target xyz, with focus depth derived as facing-axis distance to that point
- camera should not own a separate free-standing focus-plane truth that can drift away from focus target xyz and view orientation; the focused plane should be derived from those owned inputs
- the focus target cell should land near pixel-perfect on the screen type grid, while nearby cells may sway slightly from perspective math
- cursor logic is not camera ownership; programs may render cursor-like cell-groups downstream, but renderer camera truth stops at focus target and remap semantics
- renderer may expose camera movement/swing/roll/zoom operations, but keybinding ownership stays with the consuming app
- visible depth range calculations and similar helpers can live under `tools/camera/` without moving camera design truth out of domain
