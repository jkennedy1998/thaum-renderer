# /home/j/Repos/thaum-renderer/domain

## purpose
Own the renderer-specific design contracts for the atomic cell system, grouped render space, composition flow, global coordinate space, camera semantics, atlas intake, renderer-wide data lanes, post-effects, and renderer-consumed cell slots.

## owns
- the renderer-domain split for cell, cell-group, cell-weight, cell-color, cell-graphic, atlas-intake, cell-shader, cell-blur, cell-warble, data-lanes, post-effects, cell-materials, cell-adjacency, coordinate-space, composition, and camera
- the broad v1 semantic boundaries for recognizable renderer reconstruction
- renderer-owned design truth that should live in domain before implementation hardens

## does not own
- reusable low-level helpers that fit better in `tools/`
- boot-oriented orchestration outside the domain contracts
- app-level module semantics
- gameplay logic that should remain outside renderer-only rendering concerns

## children-encapsulations
- `cell/`
  - default
- `cell-group/`
  - default
- `cell-weight/`
  - default
- `cell-color/`
  - default
- `cell-graphic/`
  - default
- `atlas-intake/`
  - default
- `cell-shader/`
  - default
- `cell-blur/`
  - default
- `cell-warble/`
  - default
- `data-lanes/`
  - default
- `post-effects/`
  - default
- `cell-materials/`
  - default
- `cell-adjacency/`
  - default
- `coordinate-space/`
  - default
- `composition/`
  - default
- `camera/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/`
- `domains/contracts/`

## exposed interfaces
- none

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
- this v1 domain layer is intentionally broad and lean so the renderer can be rebuilt in a recognizable form before deeper refinement
- global coordinates are renderer-owned; local coordinates stay with cell-groups
- composition owns render ordering because overlap is an artifact of composition over shared space
- atlas intake should stay separate from sprite logic so atlas consumption rules can evolve without hiding inside the sprite seam
- renderer-wide data lanes should stay outside shader ownership even when shaders are the first main consumer
- world xyz and local xyz are now the standard coordinate language; older one-off axis naming should not return as primary shape
- cell groups are the renderer-facing submission shape; UI, tile, item, character, and similar app concepts should arrive through that shape rather than becoming renderer-owned concepts
