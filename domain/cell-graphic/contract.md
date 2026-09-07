# thaum-renderer/domain/cell-graphic

## purpose
Own the renderer cell-graphic slot as the canonical graphic input consumed by a cell.

## owns
- the canonical cell-graphic slot contract
- the shared slot boundary for glyph-backed and sprite-backed cell graphics
- the rule that supported graphics resolve to the same-sized rendering unit and final resolution shape at the cell boundary
- the downstream graphic slot consumed after atlas-intake has already resolved packed atlas assets into sprite-usable sources

## does not own
- cell position
- atlas intake ownership
- cell weight behavior beyond consuming shared weight semantics
- cell color behavior
- shader override behavior
- renderer composition behavior

## children-encapsulations
- `glyph/`
  - default
- `sprite/`
  - default
- `shape-fade/`
  - contract drafted 2026-09-07, implementation pending: shape-space interpolation between graphic tiles (glyph- and sprite-backed), built on the glyph tile rasterization seam

## contents
- `cell_graphic.rs`
  - rust cell-graphic slot shapes for glyph and sprite-backed cells owned by this encapsulation

## dependencies
- `thaum-renderer/domain/atlas-intake/`
- `thaum-renderer/domain/cell-color/`
- `thaum-renderer/domain/cell-materials/`

## exposed interfaces
- cell-graphic slot
  - describes the graphic slot consumed by a cell whether the source is a thaum mono glyph unit or a sprite unit
  - expected to keep graphic mode differences behind one stable renderer-facing cell slot and one shared color or material resolution seam

## interface consumers
- `thaum-renderer/domain/cell/`
- `thaum-renderer/domain/cell-shader/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- glyphs and sprites should remain size-compatible and resolution-compatible at the cell boundary
- atlas intake stays separate from this seam; glyphs come directly through cell-graphic while sprites come through atlas-intake plus sprite logic
- graphic mode differences should not multiply the renderer color system
