# thaum-renderer/domain/cell-materials

## purpose
Own the renderer material format, default test materials, and shared storage shape used by cells and apps of thaum-renderer.

## owns
- the canonical renderer material contract
- the place to store renderer-owned default or testing-oriented materials in renderer-owned format
- the shared four-band material format, bracketed by program-wide brand black and brand white as the complete indexed lighting ramp
- renderer-wide material resolution semantics used by both sprite and glyph graphics
- renderer-resolved grayscale, wood, stone, mole, and dirt material ids; mole and dirt remain separate ids despite their currently identical authored palettes

## does not own
- shader override ownership
- app-specific gameplay material logic
- cell-local color slot ownership
- sprite or glyph graphic ownership
- the full authored material library of every consumer app

## children-encapsulations
- none

## contents
- `cell_materials.rs`
  - rust material ids and four-band shapes; current wood/stone/mole/dirt entries plus the neutral grayscale test material

## dependencies
- `thaum-renderer/domain/cell-color/`

## exposed interfaces
- material definition shape
  - describes how a renderer material is stored, named, and resolved for cell consumption
- renderer material store
  - describes the renderer-owned place where default or example materials can live
- consumer material compatibility shape
  - describes the shared format expected from consumer-authored materials loaded from a consumer-provided path established at boot
- `CellMaterialId::all()` / `CellMaterialId::label()`
  - small discovery helpers for UI surfaces listing available material choices
- `ColorBand::from_index_clamped(i32)` / `IndexedColor::shift(i32)`
  - `ColorBand` clamps material-local reads; `IndexedColor` walks the complete brand-black → material-band → brand-white ramp and saturates at either endpoint for lighting

## interface consumers
- `thaum-renderer/domain/cell-color/`
- `thaum-renderer/domain/cell-color/material/`
- `thaum-renderer/domain/cell-graphic/sprite/`
- `thaum-renderer/domain/cell-graphic/glyph/`
- `thaum-renderer/domain/cell-shader/`
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- `cargo test -p thaum-renderer-domain --lib cell_materials`
  - light
  - validates material-band clamping/round-trip, exact wood/stone/mole/dirt authored bands, and complete indexed-ramp shifts through both brand endpoints

## data
- none

## notes
- renderer should own the ability for materials to exist and be piped through the renderer
- renderer should not own the whole authored material library of every consumer app
- consumer-authored materials should eventually live outside this repo in an expected folder system relative to a consumer path established at boot; the current small `CellMaterialId` table is the bootstrap asset bridge
- the neutral built-in test material is `gray-scale` with the four bands `000000`, `555555`, `a8a8a8`, and `ffffff`; Mole's current bootstrap entries are `wood`, `stone`, `mole`, and `dirt`
