# thaum-renderer/interfaces/data-pipelines

## purpose
Route the renderer-fed data intake surface in one place so consumers can see what dynamic values the renderer may accept.

## owns
- routed time-lane intake notes
- routed generic data-lane intake notes
- the current first proof of renderer-standard lanes

## does not own
- shader semantics
- app meaning layered on top of generic lanes
- coordinate-space ownership

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/data-lanes/`

## exposed interfaces
- time lane `breath`
  - int
- generic data intake family
  - `2 bool`
  - `2 float`
  - `4 existing generic lanes`

## interface consumers
- `thaum-renderer/domain/cell-shader/`
- future renderer consumers

## artifacts
- none

## tests
- none

## data
- none

## notes
- the only implemented renderer-standard lane right now is `breath`
- if `breath` is not provided on boot, the fallback breath worker currently seeds and advances it
- this routing space is where later generic lane names/formats can be linked once they are formalized in repo contracts
