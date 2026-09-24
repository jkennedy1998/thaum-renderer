# thaum-renderer/domain/composition/overlap-policy

## purpose
Own the overlap policy used when composed cell-groups target the same renderer coordinates.

## owns
- exact-xyz overlap resolution semantics
- the layering rule: cells from different groups at the same exact world xyz STACK — every layer survives, later groups render on top of earlier ones (pass order is bottom-to-top layer order), the data side never overwrites a stacked coordinate (J, 2026-09-14)
- true alpha stacking within the one shared coordinate: the lower layer shows through the upper layer's transparent pixels; no extra quads, alpha stacking cleanly done
- the rule that stacked coordinates aggregate on the data side: lighting and occlusion take every layer into account (any tile, character, or heap on a cell contributes; characters collide by default, non-colliding overlays like ghosts are the future case)
- per-layer weight: each stacked layer carries its own weight value (type weight for rendering, never correlated with sim weight)

## does not own
- app gameplay collision logic
- local clipping inside a group
- camera semantics

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/composition/`

## exposed interfaces
- overlap-policy shape
  - describes how renderer composition resolves collisions when multiple composed cells occupy the same exact xyz coordinate

## interface consumers
- `thaum-renderer/domain/composition/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- `cargo test -p thaum-renderer-domain compose`
  - light
  - stacking proofs: same-xyz cells from two groups both survive in pass order (later group on top), explicit pass order drives layer order, distinct coordinates stay flat

## data
- none

## notes
- this is a rendering policy, not gameplay logic
