//! Shape-fade resolution (J 2026-09-07, second pass): the consumer-facing
//! facade. Every step of a fade resolves against the GLOBAL ideal blend — the
//! per-pixel alpha lerp of the two endpoint tiles at the eased progress — so
//! the walk can only ever move toward the target shape, never wander off it
//! (J's quality feedback on first-pass walks: intermediates were resolving
//! against intermediate-to-intermediate blends and drifted away from the
//! endpoint shapes). Candidates come from the two endpoints' shape-neighbor
//! pools plus the endpoints themselves; the winner is the candidate whose
//! tile is closest in L1 alpha to the ideal blend, accepted only when it
//! beats BOTH endpoints (the never-worse rule). Everything is image-only:
//! no glyph identity, no semantics, no randomness.

use std::collections::HashMap;

use crate::shape_fade::mask_space::MASK_PIXELS;
use crate::shape_fade::neighbor_graph::{FadeTileProvider, NeighborGraph};
use crate::GlyphTileRaster;

/// Progress buckets for the resolved-char cache. The eased progress is
/// continuous, but visually indistinguishable within 1/32 of the fade.
const PROGRESS_BUCKETS: u32 = 32;

const RESOLVED_CACHE_CAP: usize = 65_536;

pub struct ShapeFade {
    graph: NeighborGraph,
    resolved: HashMap<(char, char, u32), char>,
}

impl ShapeFade {
    pub fn build(provider: &dyn FadeTileProvider) -> Self {
        Self {
            graph: NeighborGraph::build(provider),
            resolved: HashMap::new(),
        }
    }

    /// The resolved graphic for `t` along the from -> to fade: an
    /// interpolative glyph/sprite from the loaded set when one beats both
    /// endpoints against the ideal blend, otherwise the nearer endpoint.
    /// `None` only when a graphic is unknown to the graph.
    pub fn resolve_shape_fade(&mut self, from: char, to: char, t: f32) -> Option<char> {
        let t = t.clamp(0.0, 1.0);
        let bucket = ((t * PROGRESS_BUCKETS as f32).round() as u32).min(PROGRESS_BUCKETS);
        if let Some(&resolved) = self.resolved.get(&(from, to, bucket)) {
            return Some(resolved);
        }

        let a_index = self.graph.index_of(from)?;
        let b_index = self.graph.index_of(to)?;
        let resolved = if from == to {
            from
        } else {
            let ideal = blend_alphas(self.graph.tile(a_index), self.graph.tile(b_index), t);
            self.project_onto_set(from, a_index, to, b_index, &ideal, t)
        };

        if self.resolved.len() >= RESOLVED_CACHE_CAP {
            self.resolved.clear();
        }
        self.resolved.insert((from, to, bucket), resolved);
        Some(resolved)
    }

    /// Scores the candidate pool against the ideal blend with the same L1
    /// alpha distance the graph ranks by. An intermediate wins only when it
    /// beats both endpoints; otherwise the endpoint closer to the ideal
    /// renders (ties break toward `from` early, `to` late, then by char —
    /// deterministic).
    fn project_onto_set(
        &self,
        from: char,
        from_index: usize,
        to: char,
        to_index: usize,
        ideal: &[u8; MASK_PIXELS],
        t: f32,
    ) -> char {
        let score = |glyph_index: usize| -> f32 {
            alpha_distance_to_ideal(self.graph.tile(glyph_index), ideal)
        };
        let (score_from, score_to) = (score(from_index), score(to_index));

        let mut best: Option<(f32, char)> = None;
        let mut push_candidate = |glyph: char, glyph_index: usize| {
            if glyph == from || glyph == to {
                return;
            }
            let candidate_score = score(glyph_index);
            let better = match best {
                None => true,
                Some((best_score, best_glyph)) => {
                    candidate_score < best_score
                        || (candidate_score == best_score && glyph < best_glyph)
                }
            };
            if better {
                best = Some((candidate_score, glyph));
            }
        };
        for (neighbor_index, _) in self.graph.neighbors_of(from_index) {
            push_candidate(self.graph.chars[*neighbor_index], *neighbor_index);
        }
        for (neighbor_index, _) in self.graph.neighbors_of(to_index) {
            push_candidate(self.graph.chars[*neighbor_index], *neighbor_index);
        }

        // Never-worse rule: an intermediate must beat BOTH endpoints.
        if let Some((intermediate_score, intermediate)) = best {
            if intermediate_score < score_from && intermediate_score < score_to {
                return intermediate;
            }
        }
        if score_from < score_to {
            from
        } else if score_to < score_from {
            to
        } else if t < 0.5 {
            from
        } else {
            to
        }
    }

    pub fn graph(&self) -> &NeighborGraph {
        &self.graph
    }
}

fn blend_alphas(a: &GlyphTileRaster, b: &GlyphTileRaster, t: f32) -> [u8; MASK_PIXELS] {
    let mut ideal = [0u8; MASK_PIXELS];
    for (ideal_pixel, (a_pixel, b_pixel)) in
        ideal.iter_mut().zip(a.alpha.iter().zip(b.alpha.iter()))
    {
        let blended = f32::from(*a_pixel) + (f32::from(*b_pixel) - f32::from(*a_pixel)) * t;
        *ideal_pixel = blended.round().clamp(0.0, 255.0) as u8;
    }
    ideal
}

fn alpha_distance_to_ideal(tile: &GlyphTileRaster, ideal: &[u8; MASK_PIXELS]) -> f32 {
    tile.alpha
        .iter()
        .zip(ideal.iter())
        .map(|(value, target)| (i32::from(*value) - i32::from(*target)).abs())
        .sum::<i32>() as f32
        / (255.0 * MASK_PIXELS as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape_fade::neighbor_graph::FadeTileProvider;
    use crate::{GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH};

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

    struct TwoTiles;
    impl FadeTileProvider for TwoTiles {
        fn tiles(&self) -> Vec<(char, GlyphTileRaster)> {
            vec![
                ('#', tile_rect(0, 0, 11, 15)),
                (
                    ' ',
                    GlyphTileRaster {
                        width: GLYPH_TILE_WIDTH,
                        height: GLYPH_TILE_HEIGHT,
                        alpha: [0u8; MASK_PIXELS],
                    },
                ),
            ]
        }
    }

    struct GradientSet(Vec<(char, GlyphTileRaster)>);
    impl FadeTileProvider for GradientSet {
        fn tiles(&self) -> Vec<(char, GlyphTileRaster)> {
            self.0.clone()
        }
    }

    fn gradient_set() -> GradientSet {
        GradientSet(vec![
            ('O', tile_rect(1, 1, 10, 14)),
            ('8', tile_rect(0, 0, 11, 15)),
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
        for from in ['O', '8', 'o', '.'] {
            for to in ['O', '8', 'o', '.'] {
                for step in 0..=PROGRESS_BUCKETS {
                    let t = step as f32 / PROGRESS_BUCKETS as f32;
                    let resolved = fade.resolve_shape_fade(from, to, t);
                    assert!(resolved.is_some(), "{from}->{to} at {t}");
                    if step == 0 {
                        assert_eq!(resolved, Some(from));
                    }
                    if step == PROGRESS_BUCKETS {
                        assert_eq!(resolved, Some(to));
                    }
                }
            }
        }
    }

    #[test]
    fn resolution_is_deterministic_and_cache_hits_equal_cold_paths() {
        let mut fade = ShapeFade::build(&gradient_set());
        let cold: Vec<char> = (0..=PROGRESS_BUCKETS)
            .map(|step| {
                let t = step as f32 / PROGRESS_BUCKETS as f32;
                fade.resolve_shape_fade('O', 'o', t).unwrap()
            })
            .collect();
        let warm: Vec<char> = (0..=PROGRESS_BUCKETS)
            .map(|step| {
                let t = step as f32 / PROGRESS_BUCKETS as f32;
                fade.resolve_shape_fade('O', 'o', t).unwrap()
            })
            .collect();
        assert_eq!(cold, warm);
    }

    #[test]
    fn every_intermediate_is_a_loaded_graphic() {
        let mut fade = ShapeFade::build(&gradient_set());
        for step in 0..=PROGRESS_BUCKETS {
            let t = step as f32 / PROGRESS_BUCKETS as f32;
            let resolved = fade.resolve_shape_fade('O', 'o', t).unwrap();
            assert!(
                ['O', '8', 'o', '.'].contains(&resolved),
                "intermediate {resolved} is not a loaded graphic"
            );
        }
    }

    #[test]
    fn the_walk_never_revisits_a_left_behind_endpoint_shape() {
        // Global-ideal projection: once the blend has passed the halfway
        // point, resolving back to the FROM shape would mean the walk moved
        // away from the goal. The resolved sequence may hold steady, but it
        // may never return to an earlier endpoint side after crossing.
        let mut fade = ShapeFade::build(&gradient_set());
        let walk: Vec<char> = (0..=PROGRESS_BUCKETS)
            .map(|step| {
                let t = step as f32 / PROGRESS_BUCKETS as f32;
                fade.resolve_shape_fade('8', 'o', t).unwrap()
            })
            .collect();
        let mut crossed = false;
        for &resolved in &walk {
            if resolved != '8' {
                crossed = true;
            }
            assert!(
                !(crossed && resolved == '8'),
                "walk returned to the departed shape: {walk:?}"
            );
        }
    }

    #[test]
    fn a_two_tile_fade_flips_exactly_once() {
        // With no intermediates in the pool, the walk holds one endpoint then
        // the other — never oscillating (the never-worse rule on a degenerate
        // pool).
        let mut fade = ShapeFade::build(&TwoTiles);
        let mut flips = 0;
        let mut last = fade.resolve_shape_fade('#', ' ', 0.0).unwrap();
        for step in 1..=PROGRESS_BUCKETS {
            let t = step as f32 / PROGRESS_BUCKETS as f32;
            let resolved = fade.resolve_shape_fade('#', ' ', t).unwrap();
            if resolved != last {
                flips += 1;
                last = resolved;
            }
        }
        assert_eq!(flips, 1);
    }
}
