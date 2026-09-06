# thaum-renderer/tools/camera

## purpose
Own reusable camera-related helper seams that support the renderer camera without becoming the source of camera design truth.

## owns
- camera-adjacent helper organization inside tools
- reusable calculations that consume camera state

## does not own
- the canonical camera contract
- renderer-global coordinate-space design truth

## children-encapsulations
- `visible-depth-range/`
  - default

## contents
- none

## dependencies
- `thaum-renderer/domain/camera/`
- `thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- none

## interface consumers
- future renderer runtime surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- this is the split point for camera helpers that should stay reusable and implementation-facing
