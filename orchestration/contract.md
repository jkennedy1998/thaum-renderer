# /home/j/Repos/thaum-renderer/orchestration

## purpose
Own boot-only orchestration documentation and thin boot-facing seams for thaum-renderer.

## owns
- the renderer boot seam when orchestration is needed
- high-level startup wiring that belongs above domain contracts and below app-specific consumption
- boot-time intake shape for the consumer-chosen renderer asset root
- boot-time enablement or disablement of development hot reload
- renderer-owned documentation about how outside programs should stand the renderer up

## does not own
- renderer design truth
- reusable low-level helpers
- long-lived app runtime behavior after boot
- consumer app entrypoints
- consumer scene setup
- consumer-owned asset contents

## children-encapsulations
- `boot/`
  - default
- `asset-root/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/`
- `/home/j/Repos/thaum-renderer/tools/`
- `/home/j/Repos/thaum-renderer/workers/`

## exposed interfaces
- renderer boot flow
  - describes the minimal orchestration seam used to stand up renderer usage
- asset-root intake
  - describes the boot-time path contract that lets apps point the renderer at one chosen renderer asset location

## interface consumers
- future renderer implementation surfaces
- future apps consuming thaum-renderer
- humans and operators shaping boot usage of thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- orchestration is only boot
- orchestration should stay lean
- this folder is canonical even when most real boot ownership lives in consumer programs
- renderer may expose thin reusable boot seams here, but should not turn orchestration into app runtime ownership
- the consuming app should still choose when to boot, what asset root to use, and what scene to submit
