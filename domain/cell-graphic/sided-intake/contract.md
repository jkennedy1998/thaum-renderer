# thaum-renderer/domain/cell-graphic/sided-intake

## purpose
Own the portable Sided declaration format and its parser, so every consumer (renderer boot, painter authoring, game programs) reads one declaration file shape into one runtime `SidedGraphic`.

## owns
- the declaration file format: format tag, version, `base`, `sides` (facing tier), `orientations` (side+roll tier)
- the portable key vocabularies: kebab facing names (`pos-x` … `neg-z`) and roll names (`deg-0` … `deg-270`), plus the `<facing>/<roll>` orientation key
- entry shapes: full looks (graphic / color / weight / shader-stack) and `same-as` aliases within a tier
- look-field defaults: color defaults to flat white, weight to 0, shader-stack to empty
- declaration validation with author-facing errors: unknown keys, missing graphic, bad colors, multi-form graphics, missing/cyclic same-as targets (a broken alias is a broken asset — rejected at intake, not degraded silently)
- file loading and parsing of declarations into `SidedGraphic`

## does not own
- the Sided kind itself, the side-look shape, or the fallback ladder (facing-variants)
- resolution of a won look onto a cell at render time (cell; boot staging consumes it)
- where declarations live in a program's asset tree (each program's asset root owns layout; the shared sample lives in renderer-assets)
- sprite atlas parsing (atlas-intake)

## children-encapsulations
- none

## contents
- `sided_intake.rs`
  - format vocabulary constants, portable key helpers, the parser with validation, and the tests including the canonical stick sample

## dependencies
- `thaum-renderer/domain/cell-facing/`
- `thaum-renderer/domain/cell-graphic/facing-variants/`
- `thaum-renderer/domain/cell-graphic/` (SpriteGraphic)

## exposed interfaces
- declaration parsing
  - parse declaration text or load a declaration file into a runtime SidedGraphic
- portable vocabulary
  - facing/roll/orientation kebab names and their parsers, shared with authoring tools

## interface consumers
- `thaum-renderer/orchestration/boot/`: load_sided_declaration
- future painter authoring and game programs (via thaum-renderer-domain re-exports)

## artifacts
- none

## tests
- ladder end-to-end on the stick sample (upright, lying, ends), defaults, base/fall-through, sprite/none/material/shader-stack shapes, sample-file load from the asset root, and rejection of broken declarations (light)

## data
- none

## notes
- the canonical sample is `orchestration/renderer-assets/sided/stick.json`: upright horizontals `┃` declared non-rolled (facing tier), lying horizontals `━` as side+roll entries (deg-90 tips left, deg-270 tips right), end grain `▪` on top/bottom at every roll
- `deg-90` is 90° counter-clockwise viewed from the canonical front — an upright stick tips to the left
- unknown keys fail loudly on purpose: hand-authored files benefit from typo-shaped errors more than from silent leniency
