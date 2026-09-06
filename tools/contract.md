# thaum-renderer/tools

## purpose
Own reusable low-level helper surfaces for thaum-renderer that are not the primary home of renderer-specific design truth.

## owns
- shared helper boundaries that support renderer implementation
- renderer-local tool organization for reusable calculations and utility work

## does not own
- primary renderer design contracts that belong in `domain/`
- app-specific higher-order behavior

## children-encapsulations
- `camera/`
  - default
- `color/`
  - default
- `theme-roles/`
  - default
- `window-surface/`
  - default

## contents
- none

## dependencies
- `thaum-renderer/`

## exposed interfaces
- none

## interface consumers
- future renderer runtime and worker surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- design work goes in domain; repeatable low-level helper work goes in tools
- nearest-color matching belongs in tools because it is reusable low-level logic used by multiple renderer paths
- cross-platform window and GPU-surface setup belongs in tools when it is reusable low-level implementation support rather than renderer semantic truth
- placeholder artifacts should not be treated as stable tool surfaces unless they are accounted for by a real contract boundary
