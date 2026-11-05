# thaum-renderer/domain/cell-graphic/typeface-intake

## purpose
Own the portable typeface declaration format and its parser, so a consuming program can list and load typefaces (any native weight count, monospace or proportional) without pulling in font rasterization.

## owns
- the declaration file format: format tag, version, `spacing` (monospace/proportional), `weights` (ordered native weight-file list, lightest first)
- the floor-of-one, no-ceiling weight-ladder vocabulary: `weights` must declare at least one font file; any count above that is valid
- declaration validation with author-facing errors: unknown keys, missing/invalid `spacing`, missing/empty `weights`, non-string weight entries
- file loading and parsing of declarations into a runtime `TypefaceDeclaration`
- carrying weight-file paths exactly as authored, unresolved: a relative path is the consuming program's asset-root concern (a bundled typeface); an absolute path is a user/system-supplied typeface this system loads but does not host

## does not own
- cell width/height (not a typeface property; it is the free-standing per-file setting owned by `thaum-painter/domain/file/file-schema/`)
- loading font bytes, rasterizing glyphs, or any `fontdue` usage (`cell-graphic/glyph/`)
- resolving a weight path against an asset root (`cell-graphic/glyph/`)
- sprite atlas parsing or ratio-keyed sprite batches (`atlas-intake/`)
- where declarations live in a program's asset tree (each program's asset root owns layout; the shared sample lives in renderer-assets)
- the cell-weight slot itself (`cell-weight/`)

## children-encapsulations
- none

## contents
- `typeface_intake.rs`
  - format vocabulary constants, the parser with validation, the runtime `TypefaceDeclaration`/`TypefaceSpacing` shape, and tests including the canonical thaum-mono sample

## dependencies
- none

## exposed interfaces
- declaration parsing
  - parse declaration text or load a declaration file into a runtime `TypefaceDeclaration`
  - `TypefaceDeclaration::native_weight_count`/`is_monospace` read the parsed manifest without re-deriving it

## interface consumers
- `thaum-renderer/domain/cell-graphic/glyph/`: resolves the parsed manifest into loaded fonts, fit to an externally-supplied cell profile
- future thaum-painter typeface-selection UI

## artifacts
- none

## tests
- declaration parsing/validation (format tag/version, missing/invalid `spacing`, missing/empty/non-string `weights`, unknown keys), a floor-of-one and an above-four weight count, an absolute external weight path passing through unresolved, and the canonical thaum-mono sample loading from the asset root (light)

## data
- none

## notes
- the canonical sample is `orchestration/renderer-assets/typefaces/thaum-mono.json`, declaring the same four `ThaumMono-W*.ttf` files `cell-graphic/glyph/` already loads today — this phase only adds the declaration shape, `glyph/` still resolves fonts the old hardcoded way until its own phase lands
- JetBrains Mono (the first new typeface target) is expected to declare its weights as absolute external paths, since this system bundles the *behavior* to load a typeface, not other people's typefaces (project 04n3fq3e, J 2026-11-05)
- a proportional typeface is representable by this format today (`"spacing": "proportional"`) but has no declared sample yet; that is explicitly out of scope for this project
