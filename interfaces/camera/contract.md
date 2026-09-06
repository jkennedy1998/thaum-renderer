# thaum-renderer/interfaces/camera

## purpose
Route the consumer-facing camera-control surface exposed by thaum-renderer without making the renderer itself the owner of keyboard policy.

## owns
- routed camera target movement interfaces
- routed camera swing interfaces
- routed camera roll interfaces
- routed zoom and visible-depth-radius interfaces
- the current proof-app mapping notes for those interfaces

## does not own
- raw keyboard event capture
- app-level input remapping UX
- non-camera gameplay controls

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/camera/`

## exposed interfaces
- camera target forward relative depth `+1`
- camera target forward relative depth `-1`
- camera target up relative `+1`
- camera target down relative `-1`
- camera target left relative step
- camera target right relative step
- camera swing left
- camera swing right
- camera swing up
- camera swing down
- camera roll left
- camera roll right
- zoom out
- zoom in
- cull nudge inwards depth distance
- cull nudge outwards depth distance

## interface consumers
- `thaum-renderer-test-user/`
- future renderer consumers

## artifacts
- none

## tests
- none

## data
- none

## notes
- renderer owns the camera state and the callable movement/view operations, not the keys
- current proof-app mapping in `thaum-renderer-test-user/` is:
  - camera target forward relative depth `+1`: `E`
  - camera target forward relative depth `-1`: `Q`
  - camera target up relative `+1`: `W`
  - camera target down relative `-1`: `S`
  - camera target left relative step: `A`
  - camera target right relative step: `D`
  - camera swing left: `Numpad4`
  - camera swing right: `Numpad6`
  - camera swing up: `Numpad8`
  - camera swing down: `Numpad2`
  - camera roll left: `Numpad7`
  - camera roll right: `Numpad9`
  - zoom out: `=`
  - zoom in: `-`
  - cull nudge inwards depth distance: `[`
  - cull nudge outwards depth distance: `]`
