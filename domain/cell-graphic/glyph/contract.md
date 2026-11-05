# thaum-renderer/domain/cell-graphic/glyph

## purpose
Own resolved glyph-cell rasterization: load one selected typeface declaration into font faces, fit those faces to the file-selected cell profile, and merge optional glyph sprite overrides into renderer-ready alpha tiles.

## owns
- `GlyphCellProfile`, the externally supplied target width and height used for one glyph render cell
- resolving `typeface-intake/` declarations: relative weight paths against an asset root and absolute paths as external user/system fonts
- loading one active typeface's arbitrary-length native weight ladder and clamping render-time weight selection to its last loaded face
- one face-wide font size per native weight, fitted against the active cell profile rather than per glyph
- baseline-aware glyph raster placement and clipping to one cell; proportional glyphs may overflow horizontally but never spill into a neighboring cell
- in-memory cell-profile refits of an already loaded typeface, without rereading font files
- selected glyph-sprite-batch precedence over font rasters, including stretch fitting a batch's native footprint to the active cell profile
- the temporary legacy Thaum Mono loader until boot owns per-file display-profile wiring

## does not own
- typeface declaration syntax or manifest validation (`typeface-intake/`)
- glyph spriteset sheet parsing or native atlas footprint facts (`atlas-intake/`)
- the per-file choice of cell width, height, typeface id, or spriteset id (`thaum-painter/domain/file/file-schema/`)
- atlas placement, GPU texture packing, or cell-quad sizing (`orchestration/boot/`)
- comparison-resolution downsampling and shape-fade graph construction (`shape-fade/`)

## children-encapsulations
- `glyph-color-space/`
  - default

## contents
- `glyph_graphic.rs`
  - cell-profile, resolved-font-set, dynamic alpha-raster, typeface/path resolution, font fitting, sprite override, and glyph rasterization implementation with local tests
- `glyph-color-space/`
  - child glyph color-space boundary
- `contract.md`
  - glyph rasterization ownership and interface contract

## dependencies
- `thaum-renderer/domain/cell-graphic/typeface-intake/`: `TypefaceDeclaration`, `TypefaceSpacing`, `load_typeface_declaration`
- `thaum-renderer/domain/atlas-intake/`: `load_glyph_sprite_batch_from_asset_root`, `GlyphSpriteBatchTile`
- `thaum-renderer/domain/cell-weight/`: `CellWeight`

## exposed interfaces
- `GlyphCellProfile`
  - validated target glyph-cell dimensions supplied by the active file profile
- `GlyphFontSet::load_from_declaration`
  - loads one declared typeface, target cell profile, and optional explicitly selected spriteset into the active raster source
- `GlyphFontSet::with_cell_profile`
  - refits already loaded faces and raw sprite tiles to a new target profile without filesystem reads
- `GlyphFontSet::rasterize_glyph_tile`
  - returns one target-profile alpha raster, with selected sprite tiles taking precedence over the resolved font

## interface consumers
- `thaum-renderer/domain/cell-graphic/shape-fade/`: `GlyphFontSet`, `GlyphTileRaster`
- `thaum-renderer/orchestration/boot/`: temporary `GlyphFontSet::load_from_asset_root`, glyph rasters; future per-file loader wiring

## artifacts
- none

## tests
- inline `#[cfg(test)]` in `glyph_graphic.rs` (light)
  - proves declared typeface loading at arbitrary target dimensions, cell-size-only in-memory refits, declared weight-ladder swaps, selected spriteset stretch/override precedence, proportional clipping, and legacy-default compatibility

## data
- none

## notes
- `GlyphFontSet::load_from_asset_root` remains only as the legacy Thaum Mono adapter while phase 7 passes the saved file display profile through renderer boot. It reads the canonical declaration and no longer owns a hardcoded font-filename ladder.
