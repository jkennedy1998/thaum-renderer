# /home/j/Repos/thaum-renderer/tools/color

## purpose
Own reusable low-level color helpers for fast nearest-color matching and related renderer-local color utility work.

## owns
- the renderer-local nearest-color helper seam
- fast nearest-color lookup against a provided collection
- utility support for both load-time matching and runtime or post-process matching
- low-level color utility work that does not belong in the higher-level renderer domain contracts

## does not own
- sprite color-space design truth
- material design truth
- app-specific authored color policy

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/tools/`

## exposed interfaces
- nearest-color-in-collection
  - describes the helper that takes one input color and one color collection and returns the nearest match

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/sprite/sprite-color-space/`
- future renderer implementation surfaces
- future post-process or effect surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- this tool exists because nearest-color matching is needed in more than one renderer path and should stay fast in both
