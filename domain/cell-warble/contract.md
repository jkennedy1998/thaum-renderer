# /home/j/Repos/thaum-renderer/domain/cell-warble

## purpose
Own the renderer-facing per-cell warble slot language authored on cells and transferred into post effects.

## owns
- the canonical cell-warble contract
- the authored per-cell warble slot consumed by the renderer cell shape
- the warble byte-code language written into the post-effect bus green channel
- the rule that warble is a localized bending or displacement intent signal rather than direct execution behavior
- the rule that shaders may override warble without owning the warble seam itself

## does not own
- shader stack ownership
- post-effect execution ownership
- fine surface texture ownership
- depth-of-field ownership
- app-specific tag semantics

## children-encapsulations
- none

## contents
- `cell_warble.rs`
  - rust cell-warble value shape owned by this encapsulation

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell/`
- `/home/j/Repos/thaum-renderer/domain/post-effects/`

## exposed interfaces
- cell-warble shape
  - describes the authored per-cell warble signal as a single byte code carried by a cell and later packed into the post-effect bus

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell/`
- `/home/j/Repos/thaum-renderer/domain/cell-shader/`
- `/home/j/Repos/thaum-renderer/domain/post-effects/warble/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- `cell-warble/` is the medium band in the art vocabulary: visible bending, localized waving, and displacement-style life in the cell image
- warble is authored per cell so artists can place it directly, while shaders remain free to override it
- post effects decide what each nonzero warble code means at sample time
- warble should feel larger and more shape-bending than texture while still staying cell-authored in origin
