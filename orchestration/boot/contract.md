# /home/j/Repos/thaum-renderer/orchestration/boot

## purpose
Own the minimal boot seam for standing up thaum-renderer usage.

## owns
- the startup contract for initializing the renderer into a usable state
- minimal boot flow semantics without taking over app behavior after startup

## does not own
- domain design truth
- long-lived app runtime logic
- renderer-specific low-level helpers

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/orchestration/`
- `/home/j/Repos/thaum-renderer/domain/`

## exposed interfaces
- boot shape
  - describes the minimum startup seam required to prepare renderer usage

## interface consumers
- `/home/j/Repos/thaum-renderer/orchestration/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- boot should stay lean
