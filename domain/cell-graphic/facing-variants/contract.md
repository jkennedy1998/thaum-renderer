# thaum-renderer/domain/cell-graphic/facing-variants

## purpose
Own the Sided graphic kind and the fallback-ladder rules that resolve one composed relative orientation onto one declared side graphic, so a six-sided 3D cell rotates using its cell facing.

## owns
- the Sided graphic kind (`SidedGraphic`), sibling to glyph and sprite in `CellGraphic`
- the side-graphic shape a sided declaration may declare: sprite, glyph, or
  none per side — independently per side (J 2026-09-12). Side entries are
  graphic-only: color resolves through the cell's own color slots and weight
  belongs to cell-level state, so neither is declaration data.
- side keys at two specificity tiers: exact orientation entries (facing x roll)
  and roll-agnostic facing entries
- the one fallback-ladder resolution rule: exact orientation entry, then
  facing-level entry, then the declared base — or fall-through (`None`) to the
  cell's own authored appearance when no base is declared
- same-as alias chains within each tier, with cycle degradation to the next tier

## does not own
- the facing algebra or relative-facing composition (domain/cell-facing)
- the camera-facing carrier that produces the lookup input (camera
  view-orientation)
- the runtime application of a won side graphic onto a cell (domain/cell)
- declaration file parsing and format vocabulary (sided-intake)
- glyph and sprite graphics themselves
- cell slot ownership of the graphic reference itself

## children-encapsulations
- none

## contents
- `facing_variants.rs`
  - rust Sided kind: side graphics, orientation-tier and facing-tier tables,
    same-as resolution, and the fallback-ladder lookup rules

## dependencies
- `thaum-renderer/domain/cell-facing/`
- `thaum-renderer/domain/cell-graphic/`

## exposed interfaces
- Sided graphic kind and ladder lookup
  - resolves one effective side graphic for one composed relative orientation by
    the ladder: exact orientation entry, then roll-agnostic facing entry, then
    the declared base; `None` falls through to the cell's own appearance

## interface consumers
- `thaum-renderer/domain/cell-graphic/`
- `thaum-renderer/orchestration/boot/` (staging-time side resolution)
- future renderer implementation surfaces

## artifacts
- none

## tests
- ladder resolution, same-as chains, cycle degradation, exact-over-facing
  precedence, graphic-only side entries, base/fall-through behavior (light)

## data
- none

## notes
- the lookup key is the full 24-element relative orientation (6 sides x 4 rolls;
  24, not 26): an asset declares the 6 sides and adds side+roll entries only
  where custom roll art exists — one mechanism for every specificity level
- a side's graphic is sprite | glyph | none, never a nested Sided: a side is a
  flat look, and non-recursion keeps the type simple
- a won side replaces only the cell's graphic; the cell's color slots, weight,
  and shader stack survive side resolution
- boot resolves Sided cells at staging time (cell facing under group facing and
  camera), so downstream shading/rasterization only ever sees flat graphics
- consuming programs declare their own sprites and Sided objects at the renderer
  asset layer; the painter authors the same declarations
- same-as chains are bounded by their tier's size; a cycle is a broken asset and
  degrades to the next tier or the base
