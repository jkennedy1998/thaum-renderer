# thaum-renderer/domain/post-effects/index-color-clamp

## purpose
Own the renderer final indexed color clamp post effect over the composed image.

## owns
- the canonical index-color-clamp post-effect contract
- final palette/index reduction over the composed frame

## does not own
- per-cell color authorship
- palette editing UX outside renderer
- bloom behavior

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/post-effects/`

## exposed interfaces
- index-color-clamp effect shape
  - describes the final frame-wide indexed color clamp applied after other post effects

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
- this should be the last post-effect pass
- this is the current renderer-safe indexed-color finishing seam
