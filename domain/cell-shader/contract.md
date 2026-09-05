# /home/j/Repos/thaum-renderer/domain/cell-shader

## purpose
Own the renderer cell-shader slot as the additive per-cell effector channel consumed by a cell.

## owns
- the canonical cell-shader slot contract
- the rule that a cell may carry shader-stack state and that `0` means pass or nothing when used
- explicit shader-stack ordering on a cell
- the rule that later shaders in the stack win on channels they override
- additive per-cell visual mutation over graphic, color, weight, texture, and warble-facing outputs
- shader-side absolute or relative color writes for flat-color or material-backed color
- shader-side absolute or relative weight writes
- shader-side graphic writes that may swap a cell between glyph-backed and sprite-backed graphics
- shader-side texture control writes consumed through the dedicated `cell-texture/` seam
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

## contents
- `cell_shader.rs`
  - rust shader-stack constants and shaded slot resolution helpers owned by this encapsulation

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell/`
- `/home/j/Repos/thaum-renderer/domain/cell-adjacency/`
- `/home/j/Repos/thaum-renderer/domain/cell-color/`
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/`
- `/home/j/Repos/thaum-renderer/domain/cell-texture/`
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
- shaders are the per-cell override seam layered on top of authored cell slot defaults
- shader writes may target flat-color, material-backed color, texture, warble, graphic, or weight without owning the underlying renderer-wide systems
- texture and warble shader outputs should resolve as slot overrides before those values are packed into the post-effect bus
- the initial standard shader should be a sin-style relative weight modulator driven by `breath` and renderer world xyz input
- source-of-truth from J: the vivid flash pair (lasso/selection previews) must blink fast enough to read — `VIVID_FLASH_BREATH_PERIOD` is 1 breath tick per phase (fastest readable blink)
- checker does not need to be part of the built-in proving set
- there is no cheap-vs-expensive split locked yet; that line should wait for actual renderer tests
- shaders should only pay adjacency cost when adjacency is actually used
