# thaum-renderer/tools/camera/visible-depth-range

## purpose
Own the helper boundary for calculating visible depth range from camera state against renderer coordinates.

## owns
- visible depth range calculation semantics as a reusable helper seam

## does not own
- the camera contract itself
- render ordering or composition policy

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/tools/camera/`

## exposed interfaces
- visible depth range calculation
  - consumes camera state and coordinate context to derive a renderer-usable depth visibility range

## interface consumers
- future renderer runtime surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- this was explicitly called out as a good candidate for a camera tool rather than camera design truth
