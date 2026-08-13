# /home/j/Repos/thaum-renderer/domain/cell-graphic/glyph

## purpose
Own glyph-based cell graphics rendered as thaum mono characters or equivalent mono source graphics.

## owns
- the glyph graphic form for a cell
- unicode-character-backed graphic truth tuned to thaum mono
- the first proving glyph source path for Thaum Mono font assets and weight variants
- the rule that glyph source decode resolves into the same final renderer shape used by sprites
- glyph compatibility with shared flat-color and material resolution

## does not own
- sprite sheet semantics
- material library ownership
- shader stack ownership
- app-specific text semantics above renderer graphic decode

## children-encapsulations
- `glyph-color-space/`
  - default

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/`
- `/home/j/Repos/thaum-renderer/domain/cell-color/`
- `/home/j/Repos/thaum-renderer/domain/cell-materials/`

## exposed interfaces
- glyph graphic shape
  - describes a glyph-sized cell graphic rendered through the thaum mono typegrid while staying compatible with shared renderer color and material resolution

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
- glyph binary and mono assumptions should not fork the wider renderer color system
- the first target tile shape is Thaum Mono at 12x16 with four weight variants
