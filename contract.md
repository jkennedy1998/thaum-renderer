# /home/j/Repos/thaum-renderer

## purpose
Own the thaum renderer rebuild as its own repo-shaped boundary for the ASCII rendering system, with clean contract-first organization before implementation.

## owns
- the repo-level thaum-renderer contract boundary
- renderer-owned cell, cell-group, coordinate-space, camera, composition, atlas-intake, data-lanes, post-effects, and cell-slot design truth
- renderer-level composition of cell-groups on the shared global coordinate space
- the contract-first rebuild of the old bloated renderer into cleaner encapsulated shapes
- the renderer-owned fallback seam for breath timing when apps do not provide breath on boot

## does not own
- app-specific module logic
- game-specific or painter-specific behavior above rendering
- gameplay logic that depends on adjacency or simulation state outside rendering
- animation systems beyond consuming simple renderer inputs like breath

## children-encapsulations
- `domain/`
  - default
- `orchestration/`
  - default
- `tools/`
  - default
- `tests/`
  - default

## contents
- `context/`
  - lightweight design-truth artifacts gathered from workshop planning
- `domain/`
  - renderer design contracts and owned semantic boundaries
- `orchestration/`
  - boot-only renderer orchestration seam
- `tools/`
  - reusable low-level renderer helpers
- `tests/`
  - consumer-shaped renderer test seams and sample usage
- `workers/`
  - future worker-shaped execution surfaces when needed

## dependencies
- `domains/contracts/`

## exposed interfaces
- none

## interface consumers
- humans and operators shaping thaum renderer reconstruction work
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- `thaum-renderer-test-scene`
  - planned
  - boot-oriented visual validation scene for sprite, glyph, material, and composition behavior.

## data
- none

## notes
- orchestration is boot only
- domain is the home of design work and flow
- tools are reusable simple seams
- renderer owns composition of cell-groups rather than groups composing each other directly
- breath should enter through a renderer-owned time lane as simple input data, not repo-owned simulation state
- when apps do not provide breath on boot, a renderer worker may provide a fallback clock until an app-owned breath value exists
- atlas intake is its own renderer seam and should not stay hidden under sprite logic
- post-effects are renderer-owned final full-frame passes; bloom and index-color-clamp are the current minimum built-ins
- cell groups and composition are the renderer-facing pass structure; UI and other app concepts should remain outside renderer ownership
