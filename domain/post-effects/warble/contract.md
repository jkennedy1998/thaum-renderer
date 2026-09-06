# thaum-renderer/domain/post-effects/warble

## purpose
Own the warble post effect that interprets warble codes from the post-effect bus and turns them into localized bending in screen-space.

## owns
- the canonical warble post-effect contract
- execution of warble codes carried in the post-effect bus green channel
- the renderer-side interpretation table or logic for warble code meanings
- localized warp behavior that preserves the medium warble band

## does not own
- authored cell-warble slot ownership
- shader stack ownership
- texture interpretation
- depth-of-field interpretation

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/post-effects/`
- `thaum-renderer/domain/cell-warble/`

## exposed interfaces
- warble post effect
  - describes the screen-space pass that reads warble bus codes and applies localized warping to the composed image

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
- warble should bend or wobble cell imagery more strongly than texture while still feeling local rather than full-frame
- code meanings are post-effect-owned so authoring stays compact at the cell layer
- warble and texture are siblings in the same bus family, with warble representing the larger visual band
