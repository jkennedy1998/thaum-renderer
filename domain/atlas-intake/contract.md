# thaum-renderer/domain/atlas-intake

## purpose
Own the renderer-facing atlas intake seam that turns packed atlas assets into sprite-usable graphic sources with explicit atlas consumption behavior.

## owns
- the canonical atlas-intake contract
- renderer-facing atlas packing and lookup expectations
- atlas form definitions and the consumption rules paired with those forms
- stacked weight interpretation for atlas forms that vertically pack multiple weight variants
- the rule that atlas tile width and height come from shared renderer configuration rather than hardcoded per atlas asset
- atlas intake as a renderer seam separate from sprite logic and separate from sprite color-space

## does not own
- sprite color-space rules
- glyph graphic ownership
- material library ownership
- shader stack ownership
- app-specific asset pipeline behavior beyond the renderer-facing atlas seam

## children-encapsulations
- none

## contents
- `atlas_intake.rs`
  - rust atlas intake shapes and sprite-atlas loading helpers owned by this encapsulation

## dependencies
- `thaum-renderer/domain/cell-weight/`
- `thaum-renderer/domain/cell-graphic/sprite/`

## exposed interfaces
- atlas-intake shape
  - describes how atlas pngs are interpreted, tiled, and resolved into sprite-usable source graphics for renderer use

## interface consumers
- `thaum-renderer/domain/cell-graphic/sprite/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- atlas intake only owns turning atlases into sprite-usable graphic sources; glyphs remain owned by `cell-graphic/glyph/`
- atlas management is separate from sprite color-space so packed asset layout and decode color semantics can evolve independently
- the first atlas forms currently captured are:
  - `single`
    - `tiles: 1x4`
    - top tile is highest weight and bottom tile is lowest weight
  - `six-sided`
    - `tiles: 6x4`
    - hardcoded facing order: front, top, back, bottom, left, right
  - `connecting-cardinal`
    - `tiles: 4x16`
    - four stacked 4x4 connection atlases ordered from highest weight at the top to lowest weight at the bottom
