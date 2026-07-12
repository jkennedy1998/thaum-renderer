# /home/j/Repos/thaum-renderer/domain/post-effects

## purpose
Own the renderer post-effects stack that runs on the fully composed image after composition is complete.

## owns
- the canonical renderer post-effects contract
- full-frame post-process ordering after composition
- built-in post-effect placement for the renderer proving set
- the rule that these effects are image-wide rather than per-cell

## does not own
- per-cell shader behavior
- app-owned lighting logic
- composition ownership
- cell-local blur or warble semantics

## children-encapsulations
- `bloom/`
  - default
- `index-color-clamp/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/composition/`

## exposed interfaces
- post-effects stack
  - describes the ordered full-frame passes applied after the renderer has already composed the image

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
- this is the home of renderer post processing
- bloom and index-color-clamp are the current minimum built-ins
- index-color-clamp should run last in the post stack
- cell-warble and cell-blur may exist as cell-local seams, but they are not substitutes for the renderer post-effects stack
