# /home/j/Repos/thaum-renderer/domain/cell-graphic/sprite

## purpose
Own sprite-based cell graphics rendered at the same unit size and final resolution shape as glyph graphics.

## owns
- the sprite graphic form for a cell
- the rule that sprites share the same unit boundary as glyphs at the cell slot edge
- the seam where sprite source assets resolve through renderer-owned sprite color-space and atlas-intake rules
- sprite compatibility with renderer flat-color and material resolution

## does not own
- glyph semantics
- material library ownership
- shader stack ownership
- app-specific light semantics

## children-encapsulations
- `sprite-color-space/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/`
- `/home/j/Repos/thaum-renderer/domain/atlas-intake/`
- `/home/j/Repos/thaum-renderer/domain/cell-color/`
- `/home/j/Repos/thaum-renderer/domain/cell-materials/`
- `/home/j/Repos/thaum-renderer/domain/cell-weight/`

## exposed interfaces
- sprite graphic shape
  - describes a sprite-backed cell graphic that fits the same renderer unit boundary as a glyph and resolves through shared renderer color and weight seams

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- atlas intake turns packed atlases into sprite-usable source surfaces; sprite logic then consumes those resolved atlas products at the cell-graphic layer
- sprite stacking belongs to atlas management rather than sprite color-space
- sprite color-space decode is renderer-owned and not shader-owned
