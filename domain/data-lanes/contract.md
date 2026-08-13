# /home/j/Repos/thaum-renderer/domain/data-lanes

## purpose
Own the renderer-wide generic data-lane seam used by shaders and other renderer parts without forcing app semantics into the renderer.

## owns
- the canonical renderer data-lanes contract
- renderer-wide availability of externally fed dynamic values
- the rule that these lanes are renderer-owned rather than shader-owned even when shaders are a primary consumer
- the current standard time lane:
  - `breath`
    - int
    - used by standard shaders like sin when apps provide or accept renderer timing

## does not own
- app-specific meaning of the values beyond the renderer-standard breath lane
- shader ordering
- adjacency resolution
- material definitions
- app-owned light channels like `light_mag`

## children-encapsulations
- none

## contents
- `data_lanes.rs`
  - rust renderer data-lane shape and standard lane helpers owned by this encapsulation

## dependencies
- none

## exposed interfaces
- data-lanes shape
  - describes the generic externally fed value channels renderer parts may consume without requiring renderer awareness of app-level semantics
- `/home/j/Repos/thaum-renderer/interfaces/data-pipelines/contract.md`
  - routed renderer-fed data-pipeline interface notes for consumers and proof apps

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-shader/`
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- `breath` is the current renderer-standard time lane and should be treated as an int
- if an app does not provide `breath` on boot, the fallback timing seam should increment it by `1` every `.25` seconds and wrap at a high modulus value to avoid overflow
- that fallback should live in `workers/breath-fallback-clock/`
- world xyz and local xyz are standard renderer coordinates, but they are coordinate-space inputs rather than data-lane names
- app-owned light channels like `light_mag` should stay outside renderer ownership
- this seam should stay simple and fast-friendly, and may grow with more named renderer-standard lanes later if a real shared need appears
