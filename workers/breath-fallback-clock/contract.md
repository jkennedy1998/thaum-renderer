# /home/j/Repos/thaum-renderer/workers/breath-fallback-clock

## purpose
Own the fallback worker that supplies renderer breath timing when an app does not provide breath on boot.

## owns
- fallback breath ticking for renderer startup cases with no app-provided breath value
- the default increment cadence for fallback breath
- safe wrap behavior that avoids overflow during long runs

## does not own
- app-authored breath values once provided
- shader logic
- renderer composition or camera behavior

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/data-lanes/`

## exposed interfaces
- fallback breath clock
  - describes the worker-owned fallback source for the renderer `breath` int lane

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
- this worker should only be used when apps do not provide `breath` on boot
- the fallback behavior should increment by `1` every `.25` seconds
- the fallback value should wrap at a high modulus value to avoid overflow
