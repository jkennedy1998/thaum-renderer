# thaum-renderer/domain/cell-shader

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
- light-band shading: `CELL_SHADER_LIGHT_MINUS_2/MINUS_1/PLUS_1/PLUS_2/PLUS_3` walk brand black → four material bands → brand white; `CELL_SHADER_LIGHT_MINUS_3` owns the special darkest contrast remap; `resolve_shaded_color` is the color-slot counterpart to `resolve_shaded_weight`

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
- `thaum-renderer/domain/cell/`
- `thaum-renderer/domain/cell-adjacency/`
- `thaum-renderer/domain/cell-color/`
- `thaum-renderer/domain/cell-graphic/`
- `thaum-renderer/domain/cell-materials/`
- `thaum-renderer/domain/cell-texture/`
- `thaum-renderer/domain/cell-warble/`
- `thaum-renderer/domain/cell-weight/`
- `thaum-renderer/domain/data-lanes/`

## exposed interfaces
- cell-shader slot
  - describes the dynamic renderer slot carried by a cell for shader-stack based writes or overrides
- shader stack
  - describes an explicitly ordered array-like shader sequence, currently expected to map shader ints to shader definitions for fast consumption

## interface consumers
- `thaum-renderer/domain/cell/`
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- `cargo test -p thaum-renderer-domain --lib cell_shader`
  - light
  - validates pass-through, weight-sin stepping, later-shader-wins ordering, texture/warble overrides, the vivid-flash phase split, ordinary indexed light-ramp saturation, the darkest contrast remap, `resolve_shaded_color` leaving the authored range unshifted under `CELL_SHADER_PASS`, and light shader asset-name resolution

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
- source-of-truth from J (2026-09-18, mole-in-the-wall session): "it's just taking the current indexed palette for the material and shifting colors down and up. Brighter than normal gets shifted towards the bright colors, darker than normal gets shifted towards the darkest color" — ordinary light shaders use `IndexedColor::shift`, a plain clamped index shift through brand black, the four material bands, and brand white. The dedicated darkest state is the intentional later contrast-preserving exception.
- how many discrete light shifts an app actually uses (3-band vs 5-band window) is the app's clamp choice before picking a shader id, not a renderer concept — the renderer only needs to define enough shift constants to cover the widest window in use
