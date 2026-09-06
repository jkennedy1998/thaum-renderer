# thaum-renderer/interfaces

## purpose
Route repo-level exposed interfaces in one place so consumers can see the intended intake surface without confusing those interfaces with app-owned keybinding policy.

## owns
- the repo-level interface routing surface
- camera-control interface routing
- renderer data-pipeline interface routing
- test-user mapping notes for the current manual proof consumer

## does not own
- renderer domain semantics
- app-owned control policy
- shader or material implementation details

## children-encapsulations
- `camera/`
  - default
- `data-pipelines/`
  - default

## contents
- `camera/`
  - routed camera-facing movement and view-control interfaces
- `data-pipelines/`
  - routed renderer-fed data interfaces

## dependencies
- `thaum-renderer/domain/camera/`
- `thaum-renderer/domain/data-lanes/`

## exposed interfaces
- `camera/contract.md`
  - camera-facing control interface routing for consumers and proof apps
- `data-pipelines/contract.md`
  - renderer data-lane intake routing for consumers and proof apps

## interface consumers
- `thaum-renderer-test-user/`
- future renderer consumers

## artifacts
- none

## tests
- none

## data
- none

## notes
- renderer exposes controllable camera and data intake surfaces, but does not own keyboard mappings
- the test-user program is allowed to pick temporary concrete keybindings while proving these routed interfaces
