# /home/j/Repos/thaum-renderer/domain/post-effects/bloom

## purpose
Own the renderer bloom post effect as a full-frame glow-style pass over the composed image.

## owns
- the canonical bloom post-effect contract
- bloom behavior as a full-frame post process

## does not own
- per-cell emissive or lighting authorship
- app-owned light simulation
- final indexed palette clamping

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/post-effects/`

## exposed interfaces
- bloom effect shape
  - describes the renderer-facing bloom pass applied to the composed frame

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
- bloom belongs in post effects, not in the per-cell shader seam
- apps may author their own light look through materials and channels without moving light ownership into renderer
