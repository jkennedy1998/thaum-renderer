# thaum-renderer/domain/modules/shared/scroll-state

## purpose
Own the shared row-scroll state primitive for scrollable module panels: row offset, max offset, wheel stepping, and edge clamping, plus the pinned-region viewport math that lets a panel hold header/footer blocks still while the middle scrolls.

## owns
- the `ScrollState` type (row offset)
- max-offset math over total content rows and viewport rows
- wheel-delta stepping with one row per step, clamped at both edges
- pinned-region math: available middle rows = viewport minus pinned top and bottom blocks

## does not own
- what the rows are (content building), owned by each module
- drawing or crop logic, owned by each module's draw pass
- pointer capture or input routing, owned by `domain/controls/` and the module contract
- field-value nudging on wheel (property-row semantic), owned by `property-rows/` consumers

## children-encapsulations
- none

## contents
- `scroll_state.rs`
  - `ScrollState` and the pinned-region helpers
- `contract.md`
  - this contract

## dependencies
- none (pure std)

## exposed interfaces
- `ScrollState::new()` — offset starts at the top
- `ScrollState::offset()` — current row offset
- `ScrollState::max_offset(total_rows, viewport_rows)` — `total - viewport`, saturated at 0
- `ScrollState::available_rows(viewport_rows, pinned_top, pinned_bottom)` — middle rows between pinned blocks
- `ScrollState::wheel(&mut self, delta_y, max)` — wheel up scrolls toward the top, down toward the bottom, clamped; returns whether the offset moved
- `ScrollState::to_top(&mut self)` / `ScrollState::to_end(max)` — jump to either edge

## interface consumers
- none yet (first adopter: thaum-painter `graphic-picker`, which previously kept this pattern private)

## artifacts
- none

## tests
- `scroll-state`
  - light
  - offset clamping at both edges, wheel stepping direction, max-offset saturation, pinned-region availability math

## data
- none

## notes
- Extracted from thaum-painter's graphic-picker, which proved the pattern: unified scrollable category rows with a pinned RECENT block.
- One wheel step = one row, matching the graphic-picker behavior panels are already tuned to.
- Modules with a different wheel semantic (field nudging, swatch cycling) should not adopt this; it is only for list-scroll panels.
