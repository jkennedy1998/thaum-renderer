# /home/j/Repos/thaum-renderer/domain/post-effects/fudge

## purpose
Own the renderer's fudge post-effect as the large coherent distortion pass applied to the fully composed image.

## owns
- the canonical fudge post-effect contract
- full-frame coherent drift or smear style distortion after composition
- the large-scale band in the texture / warble / fudge art vocabulary
- screen-space fudge expectations that operate on the composed image rather than individual cells

## does not own
- per-cell shader ownership
- fine cell-texture ownership
- medium cell-warble ownership
- composition ownership

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/post-effects/`
- `/home/j/Repos/thaum-renderer/domain/composition/`

## exposed interfaces
- fudge post-effect
  - describes the full-frame distortion pass used for large coherent image drift, smear, or similar screen-space motion language

## interface consumers
- `/home/j/Repos/thaum-renderer/`
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- fudge belongs in post-effects because it acts on the fully composed frame instead of on per-cell geometry or sampling
- this is the large band in the art vocabulary; it should feel broader and more image-wide than `cell-warble/`
- fine surface fuzz belongs in `cell-texture/`; medium visible waving belongs in `cell-warble/`
- the current boundary is intentionally descriptive rather than performance-locked; real renderer testing should decide later how coherent and how strong the composed-image distortion should be
