# /home/j/Repos/thaum-renderer/workers

## purpose
Own optional worker-shaped runtime support seams for thaum-renderer when background watching or fallback ticking is needed.

## owns
- worker seam organization inside the repo
- optional background helpers like fallback clocks and hot-reload watchers

## does not own
- renderer design truth
- core composition behavior
- app-authored scene logic

## children-encapsulations
- `breath-fallback-clock/`
  - default
- `asset-hot-reload/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/`

## exposed interfaces
- none

## interface consumers
- future renderer runtime surfaces
- future test scenes

## artifacts
- none

## tests
- none

## data
- none

## notes
- workers are optional support seams, not the home of renderer semantic ownership
