# thaum-renderer/workers/asset-hot-reload

## purpose
Own the optional hot-reload watcher seam that listens for asset changes under the consumer-provided renderer asset root and triggers reloadable renderer resources.

## owns
- asset-watch behavior for renderer-facing hot reload
- the rule that hot reload is optional and boot-configurable
- change detection over the renderer asset root when enabled
- reload signaling for sprites, materials, shaders, cell effects, and post effects when their backing assets change

## does not own
- the canonical asset-root path contract
- renderer boot ownership
- app-specific authoring tools
- non-renderer asset watching

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/workers/`
- `thaum-renderer/orchestration/asset-root/`

## exposed interfaces
- asset hot-reload watcher
  - describes the optional worker seam that watches a renderer asset root and requests reload of renderer-facing resources

## interface consumers
- future renderer runtime surfaces
- future test scenes
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- hot reload should be built in mind from the start even if some asset families gain it before others
- the watcher should stay scoped to renderer-relevant assets under the chosen renderer root
- hot reload is for development ergonomics and should be cleanly disableable
