# /home/j/Repos/thaum-renderer/domain/cell-color

## purpose
Own the renderer cell-color slot as the canonical color input consumed by a cell.

## owns
- the canonical cell-color slot contract
- the boundary where a cell consumes either direct flat color or material-driven color behavior
- the rule that color is a renderer-owned slot rather than ad hoc app-owned cell state
- the shared renderer resolution seam used by both sprite and glyph source graphics

## does not own
- shader override ownership
- sprite color-space ownership
- glyph color-space ownership
- cell position
- cell weight behavior
- cell graphic selection
- the full material library contract that belongs in `cell-materials/`

## children-encapsulations
- `flat-color/`
  - default
- `material/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-materials/`
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/sprite/sprite-color-space/`
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/glyph/glyph-color-space/`

## exposed interfaces
- cell-color slot
  - describes the color slot consumed by a cell whether it resolves from direct flat input or material-driven channel and band resolution

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell/`
- `/home/j/Repos/thaum-renderer/domain/cell-shader/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- the base flat-color path should stay simple and fast even though shaders may still write the slot
- material color is not shader override behavior; it is smart renderer-owned color resolution
- cell-color owns the slot; cell-shaders are the override seam that may write into it without owning the color-system rules behind it
