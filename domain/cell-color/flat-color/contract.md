# thaum-renderer/domain/cell-color/flat-color

## purpose
Own the direct flat-color path for the cell-color slot.

## owns
- the base direct color truth for a cell
- the simple fast path for a cell to carry explicit color without material indirection
- flat slot assignment over shared renderer channel identity without value-band-sensitive output

## does not own
- material definitions
- shader ownership
- graphic selection

## children-encapsulations
- none

## contents
- none

## dependencies
- `thaum-renderer/domain/cell-color/`

## exposed interfaces
- flat-color value
  - describes a direct color carried by the cell-color slot without material lookup

## interface consumers
- `thaum-renderer/domain/cell-color/`
- `thaum-renderer/domain/cell-shader/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- when flat color is assigned to slots like `A`, `B`, or `C`, all source value-band differences collapse to one output color for that slot
- blend channels should still resolve as blends of the assigned flat outputs
