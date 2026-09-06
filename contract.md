# thaum-renderer

## purpose
Own the thaum renderer rebuild as its own repo-shaped boundary for the ASCII rendering system, with clean contract-first organization before implementation.

## owns
- the repo-level thaum-renderer contract boundary
- renderer-owned cell, cell-group, coordinate-space, camera, composition, atlas-intake, data-lanes, post-effects, and cell-slot design truth
- renderer-level composition of cell-groups on the shared global coordinate space
- the contract-first rebuild of the old bloated renderer into cleaner encapsulated shapes
- the renderer-owned fallback seam for breath timing when apps do not provide breath on boot
- the primary renderer target as a real graphics window presented to a human viewer rather than png-first or terminal-first output
- the boot-configurable renderer asset-root seam and optional hot-reload development path

## does not own
- app-specific module logic
- game-specific or painter-specific behavior above rendering
- gameplay logic that depends on adjacency or simulation state outside rendering
- animation systems beyond consuming simple renderer inputs like breath

## children-encapsulations
- `domain/`
  - default
- `interfaces/`
  - default
- `orchestration/`
  - default
- `tools/`
  - default
- `tests/`
  - default
- `workers/`
  - default

## contents
- `context/`
  - lightweight design-truth artifacts gathered from workshop planning
- `domain/`
  - renderer design contracts and owned semantic boundaries
- `interfaces/`
  - routed exposed-interface notes for camera control surfaces and data pipelines
- `orchestration/`
  - boot-only renderer orchestration seam
- `tools/`
  - reusable low-level renderer helpers
- `tests/`
  - consumer-shaped renderer test seams and sample usage
- `workers/`
  - worker-shaped execution surfaces such as fallback clocks and hot-reload watchers when needed

## dependencies
- `domains/contracts/`

## exposed interfaces
- `interfaces/camera/contract.md`
  - routed camera-facing movement, swing, roll, zoom, and cull-distance interfaces; keybindings remain consumer-owned
- `interfaces/data-pipelines/contract.md`
  - routed renderer-fed data-lane interfaces including `breath` and the planned generic lane family

## interface consumers
- humans and operators shaping thaum renderer reconstruction work
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- `workspace-build-smoke`
  - light
  - proves the renderer workspace crates compile cleanly without any repo-local manual test launcher.

## data
- none

## notes
- orchestration is boot only
- domain is the home of design work and flow
- tools are reusable simple seams
- renderer owns composition of cell-groups rather than groups composing each other directly
- breath should enter through a renderer-owned time lane as simple input data, not repo-owned simulation state
- when apps do not provide breath on boot, a renderer worker may provide a fallback clock until an app-owned breath value exists
- atlas intake is its own renderer seam and should not stay hidden under sprite logic
- post-effects are renderer-owned final full-frame passes over the rendered image; bloom and index-color-clamp are the current minimum built-ins
- the primary implementation direction should be GPU-first and cross-platform rather than planning around a later CPU-to-GPU migration
- the renderer should present to a real window or platform surface suitable for Linux, Windows, Apple platforms, and Android-class targets
- apps should be able to choose one renderer asset-root location at boot rather than being forced into a repo-wide folder shape
- hot reload is an optional development seam that should be designed in from the start
- cell groups and composition are the renderer-facing pass structure; UI and other app concepts should remain outside renderer ownership
- the canonical manual proof app lives in `thaum-renderer-test-user/`; renderer should not keep a second bootable test exe in-repo
