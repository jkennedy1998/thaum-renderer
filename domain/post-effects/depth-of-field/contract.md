# thaum-renderer/domain/post-effects/depth-of-field

## purpose
Own the depth-of-field post effect that interprets relative depth from the post-effect bus and applies focus-plane-aware image softening in screen-space.

## owns
- the canonical depth-of-field post-effect contract
- execution of focus-relative depth carried in the post-effect bus blue channel
- the renderer-side interpretation of how far-from-focus image regions should soften
- depth-driven blur behavior as a post-effect concern rather than a cell-authored slot

## does not own
- authored cell slot ownership
- shader stack ownership
- texture interpretation
- warble interpretation
- camera projection ownership

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/post-effects/`
- `thaum-renderer/domain/camera/`

## exposed interfaces
- depth-of-field post effect
  - describes the screen-space pass that reads relative depth codes from the bus and softens pixels away from the focus plane

## interface consumers
- `thaum-renderer/domain/post-effects/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- the focus plane maps to bus code `128`
- darker depth codes indicate nearer-than-focus pixels and brighter depth codes indicate farther-than-focus pixels
- depth of field should consume the shared bus depth signal rather than requiring cells to author blur directly
