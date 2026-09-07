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

use std::collections::HashMap;

use crate::shape_fade::neighbor_graph::{FadeTileProvider, NeighborGraph};
use crate::shape_fade::similarity::alpha_distance;

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

pub struct ShapeFade {
    graph: NeighborGraph,
    walks: HashMap<(char, char), Vec<char>>,
}

impl ShapeFade {
    pub fn build(provider: &dyn FadeTileProvider) -> Self {
        Self {
            graph: NeighborGraph::build(provider),
            walks: HashMap::new(),
        }
    }

    /// The tour for `t` along the from -> to fade. Always a loaded
    /// glyph/sprite; `None` only when a graphic is unknown to the graph.
    pub fn resolve_shape_fade(&mut self, from: char, to: char, t: f32) -> Option<char> {
        let t = t.clamp(0.0, 1.0);
        let bucket =
            ((t * (PROGRESS_BUCKETS - 1) as f32).round() as usize).min(PROGRESS_BUCKETS - 1);
        if !self.walks.contains_key(&(from, to)) {
            if self.walks.len() >= WALK_CACHE_CAP {
                self.walks.clear();
            }
            let walk = self.compute_walk(from, to)?;
            self.walks.insert((from, to), walk);
        }
        self.walks.get(&(from, to)).map(|walk| walk[bucket])
    }

    /// The gradient tour: pool glyphs (endpoints' neighbor lists plus the
    /// endpoints) filtered to those at most as far from the target as `from`
    /// is, ordered by that distance descending, paced evenly across the
    /// progress grid. `from` opens the tour, `to` closes it.
    fn compute_walk(&self, from: char, to: char) -> Option<Vec<char>> {
        let from_index = self.graph.index_of(from)?;
        let to_index = self.graph.index_of(to)?;
        if from == to {
            return Some(vec![from; PROGRESS_BUCKETS]);
        }

        let max_distance = alpha_distance(self.graph.tile(from_index), self.graph.tile(to_index));
        let mut gradient: Vec<(f32, char)> = vec![(max_distance, from)];
        let push_pool = |pool_source: usize, gradient: &mut Vec<(f32, char)>| {
            for (neighbor_index, _) in self.graph.neighbors_of(pool_source) {
                let glyph = self.graph.chars[*neighbor_index];
                if glyph == from || glyph == to {
                    continue;
                }
                let distance =
                    alpha_distance(self.graph.tile(*neighbor_index), self.graph.tile(to_index));
                // Only glyphs on the target side of `from` keep the tour
                // monotone; duplicates of the target itself are skipped (the
                // tour must END on `to`, not on a look-alike).
                if distance < max_distance
                    && distance > 0.0
                    && !gradient.iter().any(|(_, g)| *g == glyph)
                {
                    gradient.push((distance, glyph));
                }
            }
        };
        push_pool(from_index, &mut gradient);
        push_pool(to_index, &mut gradient);
        gradient.sort_by(|(da, ga), (db, gb)| {
            db.total_cmp(da)
                .then_with(|| ga.cmp(gb))
                .then_with(|| glyph_rank(*ga, from, to).cmp(&glyph_rank(*gb, from, to)))
        });
        gradient.push((0.0, to));

        // Even pacing: every gradient glyph dwells for its share of the fade.
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

    pub fn graph(&self) -> &NeighborGraph {
        &self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape_fade::mask_space::MASK_PIXELS;
    use crate::shape_fade::neighbor_graph::FadeTileProvider;
    use crate::{GlyphTileRaster, GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH};

    fn tile_rect(x0: usize, y0: usize, x1: usize, y1: usize) -> GlyphTileRaster {
        let mut tile = GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: [0u8; MASK_PIXELS],
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
            alpha: [0u8; MASK_PIXELS],
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
            let graph = fade.graph();
            let from_index = graph.index_of('#').unwrap();
            let to_index = graph.index_of('o').unwrap();
            (from_index, to_index, graph.tile(to_index).clone())
        };
        let mut previous_distance = {
            let graph = fade.graph();
            alpha_distance(graph.tile(from_index), &target)
        };
        for step in 0..PROGRESS_BUCKETS {
            let t = step as f32 / (PROGRESS_BUCKETS - 1) as f32;
            let resolved = fade.resolve_shape_fade('#', 'o', t).unwrap();
            let distance = {
                let graph = fade.graph();
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
