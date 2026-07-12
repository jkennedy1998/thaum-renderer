# /home/j/Repos/thaum-renderer/domain/cell-shader

## purpose
Own the renderer cell-shader slot as the additive per-cell effector channel consumed by a cell.

## owns
- the canonical cell-shader slot contract
- the rule that a cell may carry shader-stack state and that `0` means pass or nothing when used
- explicit shader-stack ordering on a cell
- the rule that later shaders in the stack win on channels they override
- additive per-cell visual mutation over graphic, color, weight, blur, and warble-facing outputs
- shader-side absolute or relative color writes for flat-color or material-backed color
- shader-side absolute or relative weight writes
- shader-side graphic writes that may swap a cell between glyph-backed and sprite-backed graphics
- shader-side blur control writes consumed through the dedicated `cell-blur/` seam
- shader-side warble control writes consumed through the dedicated `cell-warble/` seam
- shader consumption of renderer context and renderer-owned data lanes
- shader consumption of renderer adjacency when used for rendering
- built-in standard shader placement such as the weight-driven sin shader

## does not own
- gameplay logic
- tags or app-side tag interpretation
- app-side adjacency logic
- cell-color system ownership
- cell-graphic system ownership
- cell-weight ownership
- full material-library ownership
- sprite or glyph color-space ownership

## children-encapsulations
- `custom-shaders/`
  - default
- `built-in-shaders/`
  - default
- `shader-data-intake/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell/`
- `/home/j/Repos/thaum-renderer/domain/cell-adjacency/`
- `/home/j/Repos/thaum-renderer/domain/cell-color/`
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/`
- `/home/j/Repos/thaum-renderer/domain/cell-blur/`
- `/home/j/Repos/thaum-renderer/domain/cell-warble/`
- `/home/j/Repos/thaum-renderer/domain/cell-weight/`
- `/home/j/Repos/thaum-renderer/domain/data-lanes/`

## exposed interfaces
- cell-shader slot
  - describes the dynamic renderer slot carried by a cell for shader-stack based writes or overrides
- shader stack
  - describes an explicitly ordered array-like shader sequence, currently expected to map shader ints to shader definitions for fast consumption

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell/`
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- shaders are not materials
- tags should stay outside renderer; apps may map tags to shaders before data reaches the renderer
- shaders are now the single preferred per-cell visual effector seam even though some of the effects they drive are still experimentally scoped
- warble and blur are intentionally being captured here before performance testing; if some of these controls prove too expensive or visually wrong per cell, they can move later without changing the higher-level art-direction intent
- shader writes may target flat-color, material-backed color, blur, warble, graphic, or weight without owning the underlying renderer-wide systems
- shader data intake is a first-class child boundary for how shader definitions or inputs are fed into renderer-facing shader use
- the initial standard shader should be a sin-style relative weight modulator driven by `breath` and renderer world xyz input
- checker does not need to be part of the built-in proving set
- there is no cheap-vs-expensive split locked yet; that line should wait for actual renderer tests
- shaders should only pay adjacency cost when adjacency is actually used
