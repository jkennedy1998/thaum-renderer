# /home/j/Repos/thaum-renderer/domain/camera

## purpose
Own the renderer camera shape that defines what part of the global coordinate space is being viewed.

## owns
- the canonical renderer camera contract
- camera position on the global coordinate space
- camera-owned focus target as one global xyz coordinate
- camera-facing view semantics like pan, roll, swing, focus-depth behavior, and projection ownership

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
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- camera shape
  - describes the renderer-owned camera state consumed to determine view placement and orientation over the global coordinate space

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
- visible depth range calculations and similar helpers can live under `tools/camera/` without moving camera design truth out of domain
