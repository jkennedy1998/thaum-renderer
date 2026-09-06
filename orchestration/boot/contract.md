# thaum-renderer/orchestration/boot

## purpose
Own the minimal boot seam for standing up thaum-renderer usage when a reusable renderer boot handoff is useful.

## owns
- the startup contract for initializing the renderer into a usable state
- minimal boot flow semantics without taking over app behavior after startup
- boot-time intake of the consumer-chosen renderer asset root
- boot-time intake of optional hot-reload mode
- boot-time intake of renderer presentation target setup when the renderer is being asked to open or bind to one
- re-exporting `tools/window-surface`'s per-frame input/frame-context shape and a camera-zoom-aware `cell_clip_size_for_state` helper, so a consumer can convert a clip-space click into a world/cell coordinate without depending on `tools/window-surface` directly

## does not own
- domain design truth
- long-lived app runtime logic
- consumer app entrypoints
- consumer scene assembly
- renderer-specific low-level helpers

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/orchestration/`
- `thaum-renderer/orchestration/asset-root/`
- `thaum-renderer/domain/`
- `thaum-renderer/tools/window-surface/`
- `thaum-renderer/workers/asset-hot-reload/` when used

## exposed interfaces
- boot shape
  - describes the minimum startup seam required to prepare renderer usage

## interface consumers
- `thaum-renderer/orchestration/`
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- boot should stay lean
- the renderer should usually be booted by another program rather than owning the whole app launch story
- this seam is for reusable renderer boot handoff, not for absorbing consumer app behavior
- the consuming program should remain the owner of boot timing, selected scene inputs, and broader process lifecycle
- if boot code exists here, it should stay thin and focused on renderer bring-up only
