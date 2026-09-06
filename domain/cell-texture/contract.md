# thaum-renderer/domain/cell-texture

## purpose
Own the renderer-facing per-cell texture slot language authored on cells and transferred into post effects.

## owns
- the canonical cell-texture contract
- the authored per-cell texture slot consumed by the renderer cell shape
- the texture byte-code language written into the post-effect bus red channel
- the rule that texture is a small-scale surface-activity intent signal rather than direct execution behavior
- the rule that shaders may override texture without owning the texture seam itself

## does not own
- shader stack ownership
- post-effect execution ownership
- medium cell-local warble ownership
- depth-of-field ownership
- app-specific tag semantics

## children-encapsulations
- none

## contents
- `cell_texture.rs`
  - rust cell-texture value shape owned by this encapsulation

## dependencies
- `thaum-renderer/domain/cell/`
- `thaum-renderer/domain/post-effects/`

## exposed interfaces
- cell-texture shape
  - describes the authored per-cell texture signal as a single byte code carried by a cell and later packed into the post-effect bus

## interface consumers
- `thaum-renderer/domain/cell/`
- `thaum-renderer/domain/cell-shader/`
- `thaum-renderer/domain/post-effects/texture/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- `cell-texture/` is the fine band in the art vocabulary and should read as small living surface activity rather than large bending
- texture is authored per cell so artists can place it directly, while shaders remain free to override it
- post effects decide what each nonzero texture code means at sample time
- texture execution should use discrete reseeded behavior rather than soft continuous smearing when animation is desired
