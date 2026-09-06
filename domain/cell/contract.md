# thaum-renderer/domain/cell

## purpose
Own the atomic renderer cell shape as the pixel-like unit of thaum renderer.

## owns
- the canonical renderer cell contract
- a cell's own position within its cell-group field
- a cell's consumption of cell-weight, cell-color, cell-graphic, cell-texture, cell-warble, and cell-shader
- the rule that a cell is an atomic unit rather than a container of other cells
- lightweight slot-resolution truth that belongs directly to the cell definition rather than a separate child encapsulation

## does not own
- other cells' positions
- overlap or compositing math between cells
- layer composition between cell groups
- deep ownership of weight, color, graphic, texture, warble, shader, or material semantics

## children-encapsulations
- none

## contents
- `cell.rs`
  - rust cell slot shape and default behavior owned by this encapsulation

## dependencies
- `thaum-renderer/domain/cell-weight/`
- `thaum-renderer/domain/cell-color/`
- `thaum-renderer/domain/cell-graphic/`
- `thaum-renderer/domain/cell-texture/`
- `thaum-renderer/domain/cell-warble/`
- `thaum-renderer/domain/cell-shader/`

## exposed interfaces
- cell shape
  - describes the atomic renderer unit as a positioned cell that consumes the renderer cell slots
  - expected to carry only its own local cell truth rather than group-level spatial rules

## interface consumers
- `thaum-renderer/domain/cell-group/`
- `thaum-renderer/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- cell is pixel-like in renderer semantics
- position is part of the cell contract because positional and adjacency-aware shader behavior is expected
- the current minimum authored cell shape is position, graphic, color, weight, texture, warble, and optional shader stack state carried by the cell
- typeface and sprite rendering should fit through the same consumed slot boundary rather than forking the cell shape itself
- when shader stack state is present, the current planning shape is an ordered array of shader ints mapped by the renderer or consumer seam, with `0` available as pass or nothing
