# thaum-renderer/domain/post-effects/texture

## purpose
Own the texture post effect that interprets texture codes from the post-effect bus and turns them into fine surface activity in screen-space.

## owns
- the canonical texture post-effect contract
- execution of texture codes carried in the post-effect bus red channel
- the renderer-side interpretation table or logic for texture code meanings
- fine displacement or reseeding behavior that preserves the small-scale texture band

## does not own
- authored cell-texture slot ownership
- shader stack ownership
- warble interpretation
- depth-of-field interpretation

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/post-effects/`
- `thaum-renderer/domain/cell-texture/`

## exposed interfaces
- texture post effect
  - describes the screen-space pass that reads texture bus codes and applies fine texture behavior to the composed image

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
- texture should make static text and sprites feel alive without reading as large bending
- code meanings are post-effect-owned so authoring stays compact at the cell layer
- animated texture should prefer discrete reseeded shifts over soft blur-like motion when movement is desired
