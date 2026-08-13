# /home/j/Repos/thaum-renderer/domain/post-effects

## purpose
Own the renderer post-effects stack that runs on the fully composed image after composition is complete, including the execution of cell-authored texture, warble, and depth-driven focus effects.

## owns
- the canonical renderer post-effects contract
- full-frame post-process ordering after composition
- the post-effect bus interpretation layer over the composed image
- execution ownership for texture, warble, and depth-of-field passes
- built-in post-effect placement for the renderer proving set
- the rule that these effects are image-wide passes even when some of their inputs originate from per-cell authored signals

## does not own
- per-cell shader behavior
- app-owned lighting logic
- composition ownership
- authored cell slot ownership

## children-encapsulations
- `bloom/`
  - default
- `depth-of-field/`
  - default
- `fudge/`
  - default
- `index-color-clamp/`
  - default
- `texture/`
  - default
- `warble/`
  - default

## contents
- `post_effects.rs`
  - rust post-effect helpers and proving behaviors owned by this encapsulation
- `debug_bus.rs`
  - rust debug helpers for validating post-effect bus routing and depth encoding

## dependencies
- `/home/j/Repos/thaum-renderer/domain/composition/`
- `/home/j/Repos/thaum-renderer/domain/cell-texture/`
- `/home/j/Repos/thaum-renderer/domain/cell-warble/`

## exposed interfaces
- post-effects stack
  - describes the ordered full-frame passes applied after the renderer has already composed the image
- post-effect bus
  - describes the auxiliary per-pixel signal bus consumed by post effects, with `R = texture code`, `G = warble code`, and `B = relative depth code`

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
- texture, warble, and depth-of-field are the immediate proving set for the bus-driven path
- the post-effect bus transfers authored cell intent into screen-space execution without moving ownership of those authored slots out of cells
- relative depth in the bus is focus-centered, with `128` representing the focus plane, darker values toward camera, and brighter values away
- index-color-clamp should run last in the post stack
