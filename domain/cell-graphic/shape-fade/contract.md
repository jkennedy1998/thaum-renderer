# thaum-renderer/domain/cell-graphic/shape-fade

## purpose
Own shape-space interpolation between cell graphics: treating every 12x16 graphic tile (glyph or sprite) as a mask shape, and resolving what graphic best represents the blend between two graphic shapes at a progress point.

## owns
- the 12x16 binary mask-space packing derived from graphic tiles (glyph- and sprite-backed alike, with the same sprite-over-font precedence the tile seam uses)
- the pairwise shape-similarity metric over packed masks (Dice coefficient on popcount intersection; the empty mask — space — has similarity zero to every non-empty shape and one to itself, making fade-to-space the dissolve)
- the per-typeface shape-neighbor graph (all-pairs similarity computed once per tile-set load, kept as per-graphic k-nearest-neighbor lists)
- multi-step fade paths: ANY graphic can fade with ANY other graphic by routing through interpolative glyphs — a shortest-path walk over the neighbor graph (bounded hop count, small per-hop penalty), with a direct-edge fallback guaranteeing a path exists for every pair even across disconnected shape components
- shape-fade resolution: walking the fade path, each step resolves the graphic whose mask best approximates the ideal per-pixel alpha blend between its segment endpoints — accepted only when closer than both endpoints (never-worse-than-cutoff rule); intermediates are always real loaded glyphs or sprites, never invented shapes
- the progress-quantized fade cache bounding per-frame resolution cost (path cache per pair, resolved-char cache per pair+bucket)
- rebuild-on-load semantics: the graph and caches are derived state of the loaded tile set, rebuilt whenever the typeface/sprite batches load or update, never persisted

## does not own
- tile production (rasterization stays in `glyph/`'s font-set seam; sprite sheet decode stays in `sprite/` + atlas-intake)
- the interpolation timeline: eased progress, keyframes, property tracks (painter-document owns that; this encapsulation only consumes an already-eased `t`)
- document storage or persistence of any shape data
- cell color and cell weight behavior (weight keeps its own numeric lerp; color keeps its own channel lerp in the painter)
- randomness or session state: resolution is a pure function of (from graphic, to graphic, t, loaded tile set)

## children-encapsulations
- none

## contents
- `contract.md` (this file — phase 0; implementation modules land here in build order)

## planned contents (land with their phases)
- `mask_space.rs` — tile alpha -> packed `[u64; 3]` binary mask; empty-mask truth
- `similarity.rs` — Dice metric over packed masks
- `neighbor_graph.rs` — all-pairs similarity + kNN lists, built from a tile provider at load; rebuild on reload
- `fade.rs` — `resolve_shape_fade(from, to, t) -> graphic` walking the per-pair gradient tour, t-quantized walk cache
- `font_tiles.rs` — production `FadeTileProvider` over a loaded `GlyphFontSet` at the canonical weight; drops chars that rasterize to empty tiles (space stays — it IS the dissolve endpoint)
- `tests/` — property tests over hand-built masks (no font files needed)

## dependencies
- `thaum-renderer/domain/cell-graphic/` (slot shapes)
- `thaum-renderer/domain/cell-graphic/glyph/` (tile rasterization seam: `GlyphFontSet::rasterize_glyph_tile`, `GlyphTileRaster` 12x16 alpha)
- `thaum-renderer/domain/cell-graphic/sprite/` (tile parity only, through the same rasterize seam)
- `thaum-renderer/domain/cell-weight/` (canonical-weight decision for graph builds)

## exposed interfaces
### shape-fade resolver
send: from graphic (CellGraphic), to graphic (CellGraphic), eased progress t in 0..=1
returns: the resolved CellGraphic for that point — walking the pair's fade path, the segment covering t resolves an intermediate graphic when one beats both endpoints against the ideal blend mask, otherwise the nearer segment endpoint
effects: none at call time (reads derived state; may insert into the bounded caches)
via: `resolve_shape_fade`

### shape-neighbor graph build
send: a tile provider (anything that yields `(graphic identity, GlyphTileRaster)` for the loaded set)
returns: a built, queryable graph (kNN lists + mask table)
effects: CPU precompute bounded by tile-set size; triggered by tile-set load/update
via: `build_neighbor_graph`

## interface consumers
- `thaum-painter/domain` render/compositing path (injected resolver: the raster interpolation blend asks this seam instead of hard-cutting the graphic; default no-resolver behavior stays the halfway cutoff)
- future renderer surfaces wanting shape-aware graphic transitions

## artifacts
- none (all state is derived from the loaded tile set at runtime)

## tests
- (phase 1) mask packing round-trip, metric properties: symmetry, self-similarity, empty-mask (space) dissolve, sprite-over-font mask parity
- (phase 2) fade resolution: multi-glyph tours for shape-neighbor pairs, clean two-shape dissolves when pools hold nothing between the endpoints, cache hit == cold path, determinism, and the monotone-toward-goal property (each tour step is strictly closer to the target shape)
- (phase 3, painter side) injected-resolver blend, default-cutoff parity, end-to-end scrub through intermediates

## data
- none

## notes
- the never-worse rule is the contract's core promise: any step this encapsulation cannot bridge well degrades to the endpoint nearer to the ideal blend, never to something uglier
- the first pass routed multi-step paths through the graph and resolved each segment against its own endpoints; walks drifted away from the endpoint shapes (J's quality feedback). The second pass scores every candidate against the global endpoint blend, so the walk is monotone toward the goal by construction — the removed path router is documented history, not pending work
- every intermediate is a real loaded glyph or sprite — the encapsulation projects onto the glyph set, it never invents shapes
- pairs whose neighbor pools hold nothing closer than the endpoints fade as a clean two-shape dissolve (with the painter's weight fade softening it), which is the honest result for shape-disjoint pairs
- the graph builds at one canonical weight; per-weight graphs are a future child only if the canonical graph proves too coarse
- alpha lerp uses the u8 alpha arrays from `GlyphTileRaster` directly; binary masks are for the metric and candidate pruning
- kept deterministic by contract: no randomness, no wall-clock, no session state, sorted candidate iteration — scrub-back stability and replay identity depend on it
- single-threaded consumer seam by design (the render path); sharing across threads would need an external wrapper
