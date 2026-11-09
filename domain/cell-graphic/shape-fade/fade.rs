//! Shape-fade resolution (J 2026-09-07, fourth pass): the consumer-facing
//! facade. A fade is a GRADIENT TOUR: the walk visits the shape-gradient
//! between the two endpoints — every loaded glyph/sprite from the endpoints'
//! neighbor pools whose image distance to the TARGET is at most the from
//! endpoint's, ordered by that distance (farthest first). So the tour starts
//! at `from`, sweeps through interpolative glyphs that progressively resemble
//! the target, and lands exactly on `to` — grabbing MULTIPLE characters in
//! between (J's feedback: one blend character was too sparse, and the
//! second-pass midpoint-tie intermediates were numerical accidents, not
//! stepping stones). Monotone by construction: image distance to the target
//! is non-increasing along the tour. Everything is image-only: L1 alpha
//! distance, no glyph identity, no semantics, no randomness. The tour is
//! computed once per pair over the progress grid and cached.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::shape_fade::neighbor_graph::{
    FadeTileProvider, NeighborGraph, WeightedFadeTileProvider,
};
use crate::shape_fade::similarity::alpha_distance;
use crate::{CellWeight, GlyphTileRaster};

/// Progress buckets of the precomputed walk. Fine enough that consecutive
/// interpolative glyphs surface distinctly; the cache holds one walk per pair.
pub const PROGRESS_BUCKETS: usize = 48;

const WALK_CACHE_CAP: usize = 4096;

/// Tie-break rank for the tour order: endpoints anchor their own ends.
fn glyph_rank(glyph: char, from: char, to: char) -> u8 {
    match glyph {
        _ if glyph == from => 0,
        _ if glyph == to => 2,
        _ => 1,
    }
}

type WalkKey = (char, CellWeight, char, CellWeight, CellWeight);

pub struct ShapeFade {
    graphs: Vec<NeighborGraph>,
    /// Walk cache behind a RefCell: resolution takes `&self` so the render
    /// path can hold a shared reference. Single-threaded consumer seam (see
    /// the contract notes); the cache clears wholesale on overflow.
    walks: RefCell<HashMap<WalkKey, Vec<char>>>,
}

impl ShapeFade {
    /// Builds a constant-weight fade for small/test tile sets. Production
    /// callers should use `build_weighted` so the four real render weights
    /// retain their distinct glyph shapes.
    pub fn build(provider: &dyn FadeTileProvider) -> Self {
        Self {
            graphs: (0..4).map(|_| NeighborGraph::build(provider)).collect(),
            walks: RefCell::new(HashMap::new()),
        }
    }

    /// Builds one neighbor graph per render weight from the loaded tile set.
    pub fn build_weighted(provider: &dyn WeightedFadeTileProvider) -> Self {
        struct WeightTiles<'a> {
            provider: &'a dyn WeightedFadeTileProvider,
            weight: CellWeight,
        }
        impl FadeTileProvider for WeightTiles<'_> {
            fn tiles(&self) -> Vec<(char, crate::GlyphTileRaster)> {
                self.provider.tiles_at_weight(self.weight)
            }
        }

        Self {
            graphs: (0..provider.weight_count().max(1))
                .map(|index| {
                    NeighborGraph::build(&WeightTiles {
                        provider,
                        weight: CellWeight::from_index_clamped(index as i32),
                    })
                })
                .collect(),
            walks: RefCell::new(HashMap::new()),
        }
    }

    /// Resolves a graphic tour against the source and target's actual raster
    /// weights, while candidates come from the weight currently rendered by
    /// the cell. Every intermediate therefore has a tile shape valid for the
    /// output cell instead of a canonical-weight approximation.
    pub fn resolve_weighted_shape_fade(
        &self,
        from: char,
        from_weight: CellWeight,
        to: char,
        to_weight: CellWeight,
        output_weight: CellWeight,
        t: f32,
    ) -> Option<char> {
        let t = t.clamp(0.0, 1.0);
        let bucket =
            ((t * (PROGRESS_BUCKETS - 1) as f32).round() as usize).min(PROGRESS_BUCKETS - 1);
        let key = (from, from_weight, to, to_weight, output_weight);
        {
            let mut walks = self.walks.borrow_mut();
            if !walks.contains_key(&key) {
                if walks.len() >= WALK_CACHE_CAP {
                    walks.clear();
                }
                let walk =
                    self.compute_weighted_walk(from, from_weight, to, to_weight, output_weight)?;
                walks.insert(key, walk);
            }
        }
        self.walks.borrow().get(&key).map(|walk| walk[bucket])
    }

    /// The canonical-weight convenience path retained for the standalone
    /// shape-fade demos and static tile tests.
    pub fn resolve_shape_fade(&self, from: char, to: char, t: f32) -> Option<char> {
        self.resolve_weighted_shape_fade(
            from,
            CellWeight::ONE,
            to,
            CellWeight::ONE,
            CellWeight::ONE,
            t,
        )
    }

    /// The gradient tour: source/target endpoint distance uses their actual
    /// weights; the candidate graph and tiles use `output_weight`, matching
    /// the glyph tile the renderer will actually draw.
    fn compute_weighted_walk(
        &self,
        from: char,
        from_weight: CellWeight,
        to: char,
        to_weight: CellWeight,
        output_weight: CellWeight,
    ) -> Option<Vec<char>> {
        let source_graph = self.graph_for_weight(from_weight);
        let target_graph = self.graph_for_weight(to_weight);
        let output_graph = self.graph_for_weight(output_weight);
        let source_index = source_graph.index_of(from)?;
        let target_index = target_graph.index_of(to)?;
        let from_output_index = output_graph.index_of(from)?;
        let to_output_index = output_graph.index_of(to)?;
        if from == to {
            return Some(vec![from; PROGRESS_BUCKETS]);
        }

        let source_tile = source_graph.tile(source_index);
        let target_tile = target_graph.tile(target_index);
        let max_distance = alpha_distance(source_tile, target_tile);
        let mut gradient: Vec<(f32, char)> = vec![(max_distance, from)];
        let push_pool = |pool_source: usize, gradient: &mut Vec<(f32, char)>| {
            for (neighbor_index, _) in output_graph.neighbors_of(pool_source) {
                let glyph = output_graph.chars[*neighbor_index];
                if glyph == from || glyph == to {
                    continue;
                }
                let distance = alpha_distance(output_graph.tile(*neighbor_index), target_tile);
                // Only glyphs on the target side of the real source endpoint
                // keep the tour monotone; duplicates of the target itself are
                // skipped so the tour lands exactly on the authored target.
                if distance < max_distance
                    && distance > 0.0
                    && !gradient.iter().any(|(_, candidate)| *candidate == glyph)
                {
                    gradient.push((distance, glyph));
                }
            }
        };
        push_pool(from_output_index, &mut gradient);
        push_pool(to_output_index, &mut gradient);
        gradient.sort_by(|(da, ga), (db, gb)| {
            db.total_cmp(da)
                .then_with(|| ga.cmp(gb))
                .then_with(|| glyph_rank(*ga, from, to).cmp(&glyph_rank(*gb, from, to)))
        });
        gradient.push((0.0, to));

        let last = gradient.len() - 1;
        Some(
            (0..PROGRESS_BUCKETS)
                .map(|bucket| {
                    let position = bucket * last / (PROGRESS_BUCKETS - 1);
                    gradient[position].1
                })
                .collect(),
        )
    }

    pub fn graph_at_weight(&self, weight: CellWeight) -> &NeighborGraph {
        self.graph_for_weight(weight)
    }

    /// The loaded glyph nearest `tile` within `weight`'s own comparison-tile
    /// set — project 6kbosu52's detailer tool resolve step. Weight-scoped
    /// the same way `resolve_weighted_shape_fade` is: a hand-edited buffer
    /// never matches a shape the cell's current render weight cannot
    /// actually draw. Always returns the nearest match regardless of how far
    /// it is (snapping to the cell's own already-authored glyph is a valid,
    /// expected outcome).
    pub fn nearest_glyph_at_weight(
        &self,
        tile: &GlyphTileRaster,
        weight: CellWeight,
    ) -> Option<(char, f32)> {
        self.graph_for_weight(weight).nearest_to(tile)
    }

    /// The loaded inked glyph nearest `tile` within `weight`'s own
    /// comparison-tile set — the detailer tool's additive resolve. Blank
    /// candidates are excluded: the blank tile is every sparse hand mark's
    /// nearest neighbor, so including it resolves additive strokes to the
    /// invisible space glyph instead of a real glyph (J 2026-11-09: add on
    /// an empty cell must produce a real inked glyph, and a stroke that
    /// moves ink away from the seed must visibly change the cell).
    pub fn nearest_inked_glyph_at_weight(
        &self,
        tile: &GlyphTileRaster,
        weight: CellWeight,
    ) -> Option<(char, f32)> {
        self.graph_for_weight(weight).nearest_inked_to(tile)
    }

    /// `nearest_glyph_at_weight` with one glyph barred from winning — the
    /// detailer excludes a stroke's seed glyph so a hand edit always lands
    /// on a different, visible shape (J 2026-11-09: a subtractive line
    /// through full cells must author a visible negative, which snapping
    /// back to the seed made impossible).
    pub fn nearest_glyph_at_weight_excluding(
        &self,
        tile: &GlyphTileRaster,
        weight: CellWeight,
        exclude: Option<char>,
    ) -> Option<(char, f32)> {
        self.graph_for_weight(weight).nearest_to_excluding(tile, exclude)
    }

    /// `nearest_inked_glyph_at_weight` with one glyph barred from winning
    /// (see `nearest_glyph_at_weight_excluding`).
    pub fn nearest_inked_glyph_at_weight_excluding(
        &self,
        tile: &GlyphTileRaster,
        weight: CellWeight,
        exclude: Option<char>,
    ) -> Option<(char, f32)> {
        self.graph_for_weight(weight).nearest_inked_to_excluding(tile, exclude)
    }

    /// The loaded inked glyph nearest `tile` across EVERY declared render
    /// weight — the detailer's cross-weight resolve (J 2026-11-09: "what if
    /// this could select from different weights too"). Returns the glyph
    /// together with the weight whose graph matched, so the caller can
    /// author that weight onto the cell. Blank candidates stay excluded
    /// per weight.
    pub fn nearest_inked_glyph_across_weights_excluding(
        &self,
        tile: &GlyphTileRaster,
        exclude: Option<char>,
    ) -> Option<(char, CellWeight, f32)> {
        self.nearest_across_weights(tile, exclude, true)
    }

    /// The subtractive cross-weight lane: the blank candidate stays in
    /// play, so erasing across weights can still land on the invisible
    /// space glyph of whichever weight matches.
    pub fn nearest_glyph_across_weights_excluding(
        &self,
        tile: &GlyphTileRaster,
        exclude: Option<char>,
    ) -> Option<(char, CellWeight, f32)> {
        self.nearest_across_weights(tile, exclude, false)
    }

    fn nearest_across_weights(
        &self,
        tile: &GlyphTileRaster,
        exclude: Option<char>,
        inked_only: bool,
    ) -> Option<(char, CellWeight, f32)> {
        self.graphs
            .iter()
            .enumerate()
            .filter_map(|(index, graph)| {
                let (glyph, distance) = if inked_only {
                    graph.nearest_inked_to_excluding(tile, exclude)?
                } else {
                    graph.nearest_to_excluding(tile, exclude)?
                };
                Some((glyph, CellWeight::from_index_clamped(index as i32), distance))
            })
            .min_by(|a, b| a.2.total_cmp(&b.2))
    }

    /// One empty comparison tile at `weight`'s own tile dimensions — the
    /// additive detailer seed for a cell with no authored ink (J: add on an
    /// empty cell must be able to produce a valid glyph).
    pub fn blank_tile_at_weight(&self, weight: CellWeight) -> Option<GlyphTileRaster> {
        let graph = self.graph_for_weight(weight);
        let Some(seed) = graph.tiles.first() else {
            return None;
        };
        Some(GlyphTileRaster {
            width: seed.width,
            height: seed.height,
            alpha: vec![0; seed.width * seed.height],
        })
    }

    /// Clones one active-typeface comparison tile for the detailer seed.
    /// Unknown graphics deliberately have no editable glyph tile.
    pub fn glyph_tile_at_weight(&self, glyph: char, weight: CellWeight) -> Option<GlyphTileRaster> {
        let graph = self.graph_for_weight(weight);
        graph.index_of(glyph).map(|index| graph.tile(index).clone())
    }

    fn graph_for_weight(&self, weight: CellWeight) -> &NeighborGraph {
        let index = weight.as_index().min(self.graphs.len() - 1);
        &self.graphs[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape_fade::mask_space::MASK_PIXELS;
    use crate::shape_fade::neighbor_graph::{FadeTileProvider, WeightedFadeTileProvider};
    use crate::{CellWeight, GlyphTileRaster, GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH};

    fn tile_rect(x0: usize, y0: usize, x1: usize, y1: usize) -> GlyphTileRaster {
        let mut tile = GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: vec![0u8; MASK_PIXELS],
        };
        for y in y0..=y1 {
            for x in x0..=x1 {
                tile.alpha[y * GLYPH_TILE_WIDTH + x] = 255;
            }
        }
        tile
    }

    fn empty_tile() -> GlyphTileRaster {
        GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: vec![0u8; MASK_PIXELS],
        }
    }

    struct TwoTiles;
    impl FadeTileProvider for TwoTiles {
        fn tiles(&self) -> Vec<(char, GlyphTileRaster)> {
            vec![('#', tile_rect(0, 0, 11, 15)), (' ', empty_tile())]
        }
    }

    struct GradientSet(Vec<(char, GlyphTileRaster)>);
    impl FadeTileProvider for GradientSet {
        fn tiles(&self) -> Vec<(char, GlyphTileRaster)> {
            self.0.clone()
        }
    }

    struct WeightSet;
    impl WeightedFadeTileProvider for WeightSet {
        fn weight_count(&self) -> usize {
            2
        }

        fn tiles_at_weight(&self, weight: CellWeight) -> Vec<(char, GlyphTileRaster)> {
            let from = tile_rect(0, 0, 3, 0);
            let to = empty_tile();
            match weight {
                // At W160, `m` is a valid halfway coverage candidate.
                CellWeight::ONE => vec![('A', from), ('m', tile_rect(0, 0, 1, 0)), ('B', to)],
                // At W640 that candidate has no tile at all, so a weighted
                // fade must not select it for a W640 output cell.
                _ => vec![('A', from), ('B', to)],
            }
        }
    }

    /// A size gradient with containment: the full block, a mid block, a small
    /// block, and a dot nested inside them — a chain the walk can step down.
    fn gradient_set() -> GradientSet {
        GradientSet(vec![
            ('#', tile_rect(0, 0, 11, 15)),
            ('8', tile_rect(1, 2, 10, 13)),
            ('o', tile_rect(3, 4, 8, 11)),
            ('.', tile_rect(5, 7, 6, 8)),
        ])
    }

    #[test]
    fn endpoints_resolve_exactly_at_the_extremes() {
        let mut fade = ShapeFade::build(&TwoTiles);
        assert_eq!(fade.resolve_shape_fade('#', ' ', 0.0), Some('#'));
        assert_eq!(fade.resolve_shape_fade('#', ' ', 1.0), Some(' '));
        assert_eq!(
            fade.resolve_shape_fade('#', ' ', -0.5),
            Some('#'),
            "t clamps"
        );
        assert_eq!(fade.resolve_shape_fade('#', ' ', 1.5), Some(' '));
    }

    #[test]
    fn identical_graphics_resolve_to_themselves() {
        let mut fade = ShapeFade::build(&TwoTiles);
        assert_eq!(fade.resolve_shape_fade('#', '#', 0.37), Some('#'));
    }

    #[test]
    fn unresolved_pairs_return_none_and_known_pairs_always_resolve() {
        let mut fade = ShapeFade::build(&TwoTiles);
        assert_eq!(fade.resolve_shape_fade('#', 'Ω', 0.5), None);
        let mut fade = ShapeFade::build(&gradient_set());
        for from in ['#', '8', 'o', '.'] {
            for to in ['#', '8', 'o', '.'] {
                for step in 0..PROGRESS_BUCKETS {
                    let t = step as f32 / (PROGRESS_BUCKETS - 1) as f32;
                    let resolved = fade.resolve_shape_fade(from, to, t);
                    assert!(resolved.is_some(), "{from}->{to} at {t}");
                    if step == 0 {
                        assert_eq!(resolved, Some(from));
                    }
                }
                assert_eq!(
                    fade.resolve_shape_fade(from, to, 1.0),
                    Some(to),
                    "{from}->{to} must land on the target"
                );
            }
        }
    }

    #[test]
    fn the_walk_is_chain_deterministic_and_cache_hits_equal_cold_paths() {
        let mut fade = ShapeFade::build(&gradient_set());
        let cold: Vec<char> = (0..PROGRESS_BUCKETS)
            .map(|step| {
                fade.resolve_shape_fade('#', 'o', step as f32 / (PROGRESS_BUCKETS - 1) as f32)
                    .unwrap()
            })
            .collect();
        let warm: Vec<char> = (0..PROGRESS_BUCKETS)
            .map(|step| {
                fade.resolve_shape_fade('#', 'o', step as f32 / (PROGRESS_BUCKETS - 1) as f32)
                    .unwrap()
            })
            .collect();
        assert_eq!(cold, warm);
        let mut rebuilt = ShapeFade::build(&gradient_set());
        let fresh: Vec<char> = (0..PROGRESS_BUCKETS)
            .map(|step| {
                rebuilt
                    .resolve_shape_fade('#', 'o', step as f32 / (PROGRESS_BUCKETS - 1) as f32)
                    .unwrap()
            })
            .collect();
        assert_eq!(cold, fresh);
    }

    #[test]
    fn every_intermediate_is_a_loaded_graphic() {
        let mut fade = ShapeFade::build(&gradient_set());
        for step in 0..PROGRESS_BUCKETS {
            let t = step as f32 / (PROGRESS_BUCKETS - 1) as f32;
            let resolved = fade.resolve_shape_fade('#', 'o', t).unwrap();
            assert!(
                ['#', '8', 'o', '.'].contains(&resolved),
                "intermediate {resolved} is not a loaded graphic"
            );
        }
    }

    #[test]
    fn the_chain_steps_through_multiple_interpolative_graphics() {
        // J's ask: fades should grab MULTIPLE characters in between. The size
        // gradient gives the chain stepping stones; the walk must visit at
        // least three distinct graphics on its way down.
        let mut fade = ShapeFade::build(&gradient_set());
        let distinct: Vec<char> = {
            let mut seen = Vec::new();
            for step in 0..PROGRESS_BUCKETS {
                let t = step as f32 / (PROGRESS_BUCKETS - 1) as f32;
                let resolved = fade.resolve_shape_fade('#', '.', t).unwrap();
                if seen.last() != Some(&resolved) {
                    seen.push(resolved);
                }
            }
            seen
        };
        assert!(
            distinct.len() >= 3,
            "the chain should step through several graphics: {distinct:?}"
        );
        assert_eq!(distinct.first(), Some(&'#'));
        assert_eq!(distinct.last(), Some(&'.'));
    }

    #[test]
    fn the_walk_never_moves_away_from_the_target() {
        // The toward-goal rule: image distance to the TARGET tile is
        // non-increasing along the whole walk.
        let mut fade = ShapeFade::build(&gradient_set());
        let (from_index, _to_index, target) = {
            let graph = fade.graph_at_weight(CellWeight::ONE);
            let from_index = graph.index_of('#').unwrap();
            let to_index = graph.index_of('o').unwrap();
            (from_index, to_index, graph.tile(to_index).clone())
        };
        let mut previous_distance = {
            let graph = fade.graph_at_weight(CellWeight::ONE);
            alpha_distance(graph.tile(from_index), &target)
        };
        for step in 0..PROGRESS_BUCKETS {
            let t = step as f32 / (PROGRESS_BUCKETS - 1) as f32;
            let resolved = fade.resolve_shape_fade('#', 'o', t).unwrap();
            let distance = {
                let graph = fade.graph_at_weight(CellWeight::ONE);
                let resolved_index = graph.index_of(resolved).unwrap();
                alpha_distance(graph.tile(resolved_index), &target)
            };
            assert!(
                distance <= previous_distance + 1e-6,
                "step {step} moved away from the target ({previous_distance} -> {distance})"
            );
            previous_distance = distance;
        }
    }

    #[test]
    fn weighted_resolution_only_selects_glyphs_supported_by_the_output_weight() {
        let fade = ShapeFade::build_weighted(&WeightSet);
        assert_eq!(
            fade.resolve_weighted_shape_fade(
                'A',
                CellWeight::ONE,
                'B',
                CellWeight::ONE,
                CellWeight::ONE,
                0.5,
            ),
            Some('m'),
            "W160 has the midpoint glyph's real tile"
        );
        assert_ne!(
            fade.resolve_weighted_shape_fade(
                'A',
                CellWeight::ONE,
                'B',
                CellWeight::ONE,
                CellWeight::ZERO,
                0.5,
            ),
            Some('m'),
            "the other declared weight cannot select a glyph it does not provide"
        );
    }

    #[test]
    fn nearest_glyph_at_weight_is_scoped_to_one_weight_and_never_crosses() {
        // The detailer tool's resolve step (project 6kbosu52): a hand-edited
        // buffer identical to 'm' must find it exactly at W160, where 'm' is
        // loaded — and must never find it at W640, where that weight's graph
        // has no 'm' node at all.
        let fade = ShapeFade::build_weighted(&WeightSet);
        let probe = tile_rect(0, 0, 1, 0);

        assert_eq!(
            fade.nearest_glyph_at_weight(&probe, CellWeight::ONE),
            Some(('m', 0.0)),
            "W160 has 'm's exact tile loaded"
        );
        let (nearest, _) = fade
            .nearest_glyph_at_weight(&probe, CellWeight::ZERO)
            .unwrap();
        assert_ne!(nearest, 'm', "W640 never has an 'm' node to find");
        assert_eq!(nearest, 'A', "tie-break between equally-distant A and B");
    }

    #[test]
    fn cross_weight_nearest_reports_the_matching_weight() {
        // The detailer's cross-weight resolve (J 2026-11-09): a probe of
        // 'm's exact tile has no match inside weight ZERO's graph, but the
        // cross-weight search finds it at ONE and reports that weight so the
        // caller can author it onto the cell.
        let fade = ShapeFade::build_weighted(&WeightSet);
        let probe = tile_rect(0, 0, 1, 0);

        assert_eq!(
            fade.nearest_glyph_across_weights_excluding(&probe, None),
            Some(('m', CellWeight::ONE, 0.0)),
            "the cross-weight search finds 'm' at its own weight"
        );
        assert_eq!(
            fade.nearest_inked_glyph_across_weights_excluding(&probe, None),
            Some(('m', CellWeight::ONE, 0.0)),
            "inked lane keeps blank candidates excluded per weight"
        );
        // The seed glyph stays barred across every weight.
        assert_ne!(
            fade.nearest_glyph_across_weights_excluding(&probe, Some('m'))
                .unwrap()
                .0,
            'm'
        );
    }

    #[test]
    fn weighted_build_uses_the_provider_declared_weight_count() {
        let fade = ShapeFade::build_weighted(&WeightSet);
        assert_eq!(fade.graphs.len(), 2);
        assert_eq!(
            fade.graph_at_weight(CellWeight::from_index_clamped(40))
                .chars
                .len(),
            3
        );
    }

    #[test]
    fn a_two_tile_fade_flips_exactly_once() {
        // No stepping stones in the pool: the walk holds one endpoint then the
        // other, never oscillating.
        let mut fade = ShapeFade::build(&TwoTiles);
        let mut flips = 0;
        let mut last = fade.resolve_shape_fade('#', ' ', 0.0).unwrap();
        for step in 1..PROGRESS_BUCKETS {
            let t = step as f32 / (PROGRESS_BUCKETS - 1) as f32;
            let resolved = fade.resolve_shape_fade('#', ' ', t).unwrap();
            if resolved != last {
                flips += 1;
                last = resolved;
            }
        }
        assert_eq!(flips, 1);
    }
}
