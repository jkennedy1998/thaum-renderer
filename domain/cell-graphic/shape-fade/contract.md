# thaum-renderer/domain/cell-graphic/shape-fade

## purpose
Own shape-space interpolation between cell graphics: downsampling each rendered glyph or sprite into a bounded, aspect-preserved comparison tile, then resolving what graphic best represents the blend between two graphic shapes at a progress point.

## owns
- the fixed-bounded, aspect-preserved comparison-tile downsample from render-resolution graphic tiles (glyph- and sprite-backed alike, with the same sprite-over-font precedence the tile seam uses); comparison tiles stay grayscale (area-averaged alpha), never binarized for the similarity metric itself
- the pairwise shape-similarity metric over comparison tiles: continuous L1 alpha distance, normalized to `0..=1` (the mean absolute per-pixel alpha difference over full-scale ink) — not a binary/popcount metric. The empty tile (space) is distance `0` from itself and `1` (maximum) from full ink, making fade-to-space the dissolve
- the per-typeface, per-render-weight shape-neighbor graphs (all-pairs L1 distance computed once per tile-set load for every declared glyph weight; each node keeps its `FADE_NEIGHBOR_COUNT` (8) nearest shapes by that distance)
- the gradient-tour fade walk: for one `(from, to)` pair the walk starts at `from`, sweeps through real loaded glyphs/sprites drawn from both endpoints' neighbor pools whose distance to the TARGET is non-increasing (farthest first), and lands exactly on `to` — monotone toward the goal by construction, since every candidate is scored against the global endpoint blend rather than against its own local segment (an earlier multi-step path-routed design drifted away from the endpoint shapes and was replaced; see notes)
- weight-aware shape-fade resolution: source and target use their authored render-weight tiles, while every intermediate comes from the current output-weight graph, so an interpolated glyph always has a shape supported by the weight the renderer will draw; intermediates are always real loaded glyphs or sprites, never invented shapes
- the progress-quantized walk cache bounding per-frame resolution cost: one precomputed `PROGRESS_BUCKETS`-length (48) walk per `(from, from_weight, to, to_weight, output_weight)` key, capped and cleared wholesale on overflow
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
- `contract.md` (this file)
- `compare_tile.rs` — render-resolution alpha -> fixed-bounded (`COMPARISON_MAX_WIDTH`x`COMPARISON_MAX_HEIGHT`, 12x16), aspect-preserved, area-downsampled comparison tile; never upscales, so a render tile smaller than the bound on either axis yields a proportionally smaller comparison tile
- `mask_space.rs` — `PackedMask`: comparison-tile alpha packed into a binary coverage bitset (half-coverage threshold). Used only as a cheap "is this tile empty" filter (`font_tiles.rs` drops chars that rasterize empty) — it is not the similarity substrate; see `similarity.rs`
- `similarity.rs` — `alpha_distance`: the continuous L1 metric the neighbor graph ranks by and the fade resolves against, operating directly on the grayscale comparison tiles
- `neighbor_graph.rs` — `NeighborGraph`: all-pairs `alpha_distance` + per-node 8-nearest lists, built from a tile provider at load; rebuild on reload
- `fade.rs` — `ShapeFade`/`resolve_weighted_shape_fade(from, from_weight, to, to_weight, output_weight, t) -> Option<char>`: the gradient-tour walk, one `NeighborGraph` per declared render weight, with the progress-bucketed walk cache
- `font_tiles.rs` — production `WeightedFadeTileProvider` over every loaded `GlyphFontSet` weight, downsampled before graph input; drops chars that rasterize to empty tiles (space stays — it IS the dissolve endpoint)
- `tests/` inline per module — property tests over hand-built tiles (no font files needed)

## dependencies
- `thaum-renderer/domain/cell-graphic/` (slot shapes)
- `thaum-renderer/domain/cell-graphic/glyph/` (tile rasterization seam: `GlyphFontSet::rasterize_glyph_tile`, dynamically sized `GlyphTileRaster` alpha)
- `thaum-renderer/domain/cell-graphic/sprite/` (tile parity only, through the same rasterize seam)
- `thaum-renderer/domain/cell-weight/` (canonical-weight decision for graph builds)

## exposed interfaces
### shape-fade resolver
send: from graphic + authored render weight, to graphic + authored render weight, current output render weight, eased progress t in 0..=1
returns: the resolved `char` for that point — the gradient-tour walk whose source/target use their authored tiles and whose intermediates use the current output-weight tile set
effects: none at call time (reads derived state; may insert into the bounded walk cache)
via: `ShapeFade::resolve_weighted_shape_fade` (canonical-weight convenience: `resolve_shape_fade`)

### shape-neighbor graph build
send: a tile provider (`FadeTileProvider`/`WeightedFadeTileProvider`: yields `(char, GlyphTileRaster)` for the loaded set, per weight)
returns: a built, queryable graph (`NeighborGraph`: per-node 8-nearest lists + tiles)
effects: CPU precompute bounded by tile-set size (all-pairs over the loaded set); triggered by tile-set load/update
via: `NeighborGraph::build`, or `ShapeFade::build_weighted` for one graph per declared render weight

## interface consumers
- `thaum-painter/domain` render/compositing path (injected resolver: the raster interpolation blend and the picker's footprint fold both ask this seam instead of hard-cutting the graphic; default no-resolver behavior stays the halfway cutoff)
- future renderer surfaces wanting shape-aware graphic transitions

## artifacts
- none (all state is derived from the loaded tile set at runtime)

## tests
- mask packing (`mask_space.rs`): half-coverage threshold, empty-tile truth
- metric properties (`similarity.rs`): symmetry, self-identity at distance 0, full-ink-vs-empty at distance 1, proportional to disagreement
- neighbor graph (`neighbor_graph.rs`): ranking by distance, dedupe, determinism across rebuilds
- fade resolution (`fade.rs`): endpoint exactness, identical-graphic resolution, multi-glyph tours for shape-neighbor pairs, clean two-shape dissolves when pools hold nothing between the endpoints, cache hit == cold path, determinism, the monotone-toward-goal property, and weight-scoped candidate selection (a glyph without a tile at the output weight is never selected)

## data
- none

## notes
- the never-worse rule is the contract's core promise: any step this encapsulation cannot bridge well degrades to the endpoint nearer to the ideal blend, never to something uglier
- the first pass routed multi-step paths through the graph and resolved each segment against its own endpoints; walks drifted away from the endpoint shapes (J's quality feedback). The current pass scores every candidate against the global endpoint blend, so the walk is monotone toward the goal by construction — the removed path router is documented history, not pending work
- every intermediate is a real loaded glyph or sprite — the encapsulation projects onto the glyph set, it never invents shapes
- pairs whose neighbor pools hold nothing closer than the endpoints fade as a clean two-shape dissolve (with the painter's weight fade softening it), which is the honest result for shape-disjoint pairs
- one graph builds for every declared typeface weight; weight lookup clamps to the highest loaded graph, and cache keys include both endpoint weights and the current output weight so no tour crosses incompatible glyph shapes — the system never pulls a "more similar" shape from a different weight than the one actually being rendered
- comparison tiles are area-downsampled once under a fixed 12x16 maximum bound without upscaling and retain their render tile aspect ratio; the metric operates on this grayscale tile directly (continuous alpha distance), not on a binarized mask — `PackedMask`'s only live use is as an emptiness filter
- kept deterministic by contract: no randomness, no wall-clock, no session state, sorted candidate iteration — scrub-back stability and replay identity depend on it
- single-threaded consumer seam by design (the render path); sharing across threads would need an external wrapper
- project 6kbosu52 (the detailer tool, 2026-11-08) adds one new primitive here: resolving an arbitrary hand-edited comparison buffer (not an existing graph node) to its nearest loaded glyph — this contract will gain that interface when it lands
