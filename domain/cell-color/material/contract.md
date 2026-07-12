# /home/j/Repos/thaum-renderer/domain/cell-color/material

## purpose
Own the material-backed color path for the cell-color slot.

## owns
- the rule that a cell can reference material-driven color behavior through the cell-color slot
- the seam between a cell color request and the shared renderer material format
- channel and value-band-based color resolution as consumed by a cell

## does not own
- the full material library
- shader ownership
- flat-color truth
- graphic selection

## children-encapsulations
- none

## contents
- none

## dependencies
- `/home/j/Repos/thaum-renderer/domain/cell-color/`
- `/home/j/Repos/thaum-renderer/domain/cell-materials/`

## exposed interfaces
- material color reference
  - describes a cell-color value that resolves through the renderer material format rather than direct flat rgb

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell-color/`
- `/home/j/Repos/thaum-renderer/domain/cell-shader/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- materials always expose exactly four fixed bands: darkest, medium-dark, medium-light, and lightest
- there is no interpolation requirement between bands in the current renderer shape
- blend channels should mix the final resolved outputs of the paired materials at the same band
