# thaum-renderer/domain/atlas-intake

## purpose
Own the renderer-facing atlas intake seam that turns packed atlas assets into sprite-usable graphic sources with explicit atlas consumption behavior.

## owns
- the canonical atlas-intake contract
- renderer-facing atlas packing and lookup expectations
- atlas form definitions and the consumption rules paired with those forms
- named glyph/sprite-batch decoding from `sections.txt` plus sheet assets, reporting the batch's native tile ratio and weight rows without coupling it to the active typeface or cell profile
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
  - rust atlas intake shapes, ordinary sprite-atlas loading helpers, and named glyph/sprite-batch decoding owned by this encapsulation
- `truth.md`
  - J's authored-atlas form and renderer-ownership decisions

## dependencies
- `thaum-renderer/domain/cell-weight/`
- `thaum-renderer/domain/cell-graphic/sprite/`

## exposed interfaces
- atlas-intake shape
  - describes how atlas pngs are interpreted, tiled, and resolved into sprite-usable source graphics for renderer use
- `load_glyph_sprite_batch_from_asset_root`
  - decodes one explicitly selected glyph spriteset into glyph/weight alpha tiles and reports its native tile dimensions; a missing directory returns no batch so its caller can apply its own asset policy

## interface consumers
- `thaum-renderer/domain/cell-graphic/sprite/`
- `thaum-renderer/domain/cell-graphic/glyph/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- inline `#[cfg(test)]` in `atlas_intake.rs`
  - validates fixed atlas form decoding plus named glyph-spriteset decode, explicit-id selection, native-footprint reporting, missing-set handling, sections parsing, and weight-row order

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
  - `glyph-sprite-batch`
    - one named directory under `cell-sprites/<spriteset-id>/`, using its existing `sections.txt` and `monothaum_<section>.png` sheets
    - selected by explicit spriteset id; its native 12x16 footprint is reported, not treated as a requirement for the active file profile
