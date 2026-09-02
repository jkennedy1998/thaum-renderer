# /home/j/Repos/thaum-renderer/domain/cell-materials

## purpose
Own the renderer material format, default test materials, and shared storage shape used by cells and apps of thaum-renderer.

## owns
- the canonical renderer material contract
- the place to store renderer-owned default or testing-oriented materials in renderer-owned format
- the shared four-band material format that renderer users can also use when declaring their own materials
- renderer-wide material resolution semantics used by both sprite and glyph graphics
- a grayscale built-in test material for early validation work

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
  - rust material ids and band shapes owned by this encapsulation

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-color/`

## exposed interfaces
- material definition shape
  - describes how a renderer material is stored, named, and resolved for cell consumption
- renderer material store
  - describes the renderer-owned place where default or example materials can live
- consumer material compatibility shape
  - describes the shared format expected from consumer-authored materials loaded from a consumer-provided path established at boot
- `CellMaterialId::all()` / `CellMaterialId::label()`
  - small discovery helpers for UI surfaces listing available material choices

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-color/`
- `/home/j/Repos/thaum-renderer/domain/cell-color/material/`
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/sprite/`
- `/home/j/Repos/thaum-renderer/domain/cell-graphic/glyph/`
- future renderer implementation surfaces
- future apps consuming thaum-renderer

## artifacts
- none

## tests
- none

## data
- none

## notes
- renderer should own the ability for materials to exist and be piped through the renderer
- renderer should not own the whole authored material library of every consumer app
- consumer-authored materials should live outside this repo in an expected folder system relative to a consumer path established on boot
- the first built-in test material should be `gray-scale` with the four bands `000000`, `555555`, `a8a8a8`, and `ffffff`
