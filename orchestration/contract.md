# /home/j/Repos/thaum-renderer/orchestration

## purpose
Own boot-only orchestration for thaum-renderer.

## owns
- the renderer boot seam when orchestration is needed
- high-level startup wiring that belongs above domain contracts and below app-specific consumption

## does not own
- renderer design truth
- reusable low-level helpers
- app-owned runtime behavior after boot

## children-encapsulations
- `boot/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/`
- `/home/j/Repos/thaum-renderer/tools/`

## exposed interfaces
- renderer boot flow
  - describes the minimal orchestration seam used to stand up renderer usage

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
- orchestration is only boot
