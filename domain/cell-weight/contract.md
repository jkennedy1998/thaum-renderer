# /home/j/Repos/thaum-renderer/domain/cell-weight

## purpose
Own the renderer cell-weight slot as the canonical weight input consumed by a cell.

## owns
- the canonical cell-weight slot contract
- the discrete weight range `0`, `1`, `2`, and `3`
- weight semantics that differ between Thaum Mono character graphics and sprite graphics when needed

## does not own
- cell position
- cell color behavior
- cell graphic selection
- renderer composition behavior

## children-encapsulations
- none

## contents
- none

## dependencies
- none

## exposed interfaces
- cell-weight slot
  - describes the weight slot consumed by a cell as one of four canonical values
  - expected to normalize how weight is interpreted across supported graphic modes

## interface consumers
- `/home/j/Repos/thaum-renderer/domain/cell/`
- `/home/j/Repos/thaum-renderer/domain/cell-shader/`
- future renderer implementation surfaces

## artifacts
- none

## tests
- none

## data
- none

## notes
- the first known weight system is the four-weight Thaum Mono behavior
- sprite graphics may need different weight handling without changing the slot boundary itself
