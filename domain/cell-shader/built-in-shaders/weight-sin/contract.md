# thaum-renderer/domain/cell-shader/built-in-shaders/weight-sin

## purpose
Own the standard sin shader that modulates cell weight relative to renderer time and world xyz.

## owns
- the built-in `weight-sin` shader contract
- relative weight add or subtract behavior driven by renderer-fed breath timing
- world-xyz-aware modulation across the shared renderer space

## does not own
- data-lane ownership
- camera ownership
- checker-pattern shader behavior

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/cell-shader/`
- `thaum-renderer/domain/data-lanes/`
- `thaum-renderer/domain/coordinate-space/`

## exposed interfaces
- weight-sin shader shape
  - describes the built-in relative weight modulation shader driven by `breath` and world xyz inputs

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
- this is the preferred initial built-in over a checker shader
- the intended feel is a stepping breath-driven modulation like `-1, 0, +1, 0, -1 ...`
- output should stay within the valid cell-weight range after relative adjustment
