//! Shape-fade resolution (J 2026-09-07): the consumer-facing facade. A fade
//! between two graphics walks the pair's fade path (see `neighbor_graph`);
//! the segment covering the eased progress projects onto the loaded glyph set
//! — the candidate graphics best approximating the ideal per-pixel alpha
//! blend of the segment's endpoints. The never-worse rule is enforced here:
//! an intermediate is only used when it beats BOTH segment endpoints against
//! the ideal blend; otherwise the nearer endpoint renders. Deterministic and
//! cached (paths per pair, resolved chars per pair + progress bucket).

use std::collections::HashMap;

use crate::shape_fade::mask_space::MASK_PIXELS;
use crate::shape_fade::neighbor_graph::{FadeTileProvider, NeighborGraph};

/// Progress buckets for the resolved-char cache. The eased progress is
/// continuous, but visually indistinguishable within 1/32 of the fade.
const PROGRESS_BUCKETS: u32 = 32;

const PATH_CACHE_CAP: usize = 4096;
const RESOLVED_CACHE_CAP: usize = 65_536;

pub struct ShapeFade {
    graph: NeighborGraph,
    paths: HashMap<(char, char), Vec<char>>,
    resolved: HashMap<(char, char, u32), char>,
}

impl ShapeFade {
    pub fn build(provider: &dyn FadeTileProvider) -> Self {
        Self {
            graph: NeighborGraph::build(provider),
            paths: HashMap::new(),
            resolved: HashMap::new(),
        }
    }

    /// The pair's fade path (cached). `None` when either graphic is unknown
    /// to the graph — callers fall back to their own cutoff behavior.
    pub fn fade_path(&mut self, from: char, to: char) -> Option<&[char]> {
        if !self.paths.contains_key(&(from, to)) {
            if self.paths.len() >= PATH_CACHE_CAP {
                self.paths.clear();
            }
            let path = self.graph.fade_path(from, to)?;
            self.paths.insert((from, to), path);
        }
        self.paths.get(&(from, to)).map(|path| path.as_slice())
    }

    /// The resolved graphic for `t` along the from -> to fade. The result is
    /// always one of: an interpolative glyph/sprite from the loaded set, or
    /// one of the two endpoints. `None` only when the pair is unresolvable
    /// (a graphic unknown to the graph).
    pub fn resolve_shape_fade(&mut self, from: char, to: char, t: f32) -> Option<char> {
        let t = t.clamp(0.0, 1.0);
        let bucket = ((t * PROGRESS_BUCKETS as f32).round() as u32).min(PROGRESS_BUCKETS);
        if let Some(&resolved) = self.resolved.get(&(from, to, bucket)) {
            return Some(resolved);
        }

        let path = self.fade_path(from, to)?.to_vec();
        let resolved = if path.len() == 1 {
            path[0]
        } else {
            let scaled = t * (path.len() - 1) as f32;
            let segment = (scaled.floor() as usize).min(path.len() - 2);
            let local = scaled - segment as f32;
            let a = path[segment];
            let b = path[segment + 1];
            self.resolve_segment(a, b, local)
        };

        if self.resolved.len() >= RESOLVED_CACHE_CAP {
            self.resolved.clear();
        }
        self.resolved.insert((from, to, bucket), resolved);
        Some(resolved)
    }

    /// One path segment's projection: candidates are the segment endpoints
    /// plus both endpoints' neighbor lists; the winner is the candidate whose
    /// tile alpha is closest (L1) to the ideal blend — accepted only when it
    /// beats both endpoints, else the nearer endpoint renders.
    fn resolve_segment(&self, a: char, b: char, local: f32) -> char {
        let (a_index, b_index) = (self.graph.index_of(a), self.graph.index_of(b));
        let (Some(a_index), Some(b_index)) = (a_index, b_index) else {
            return if local < 0.5 { a } else { b };
        };

        let alpha_a = self.graph.tile(a_index).alpha;
        let alpha_b = self.graph.tile(b_index).alpha;
        let mut ideal = [0u8; MASK_PIXELS];
        for index in 0..MASK_PIXELS {
            let blended =
                alpha_a[index] as f32 + (alpha_b[index] as f32 - alpha_a[index] as f32) * local;
            ideal[index] = blended.round().clamp(0.0, 255.0) as u8;
        }

        let mut candidates: Vec<char> = vec![a, b];
        for endpoint_index in [a_index, b_index] {
            for (neighbor_index, _) in self.graph.neighbors_of(endpoint_index) {
                let neighbor = self.graph.chars[*neighbor_index];
                if neighbor != a && neighbor != b && !candidates.contains(&neighbor) {
                    candidates.push(neighbor);
                }
            }
        }

        let score = |glyph: char| -> f32 {
            match self.graph.index_of(glyph) {
                Some(index) => {
                    let alpha = &self.graph.tile(index).alpha;
                    alpha
                        .iter()
                        .zip(ideal.iter())
                        .map(|(value, target)| ((*value as i32) - (*target as i32)).abs())
                        .sum::<i32>() as f32
                }
                None => f32::INFINITY,
            }
        };

        let (score_a, score_b) = (score(a), score(b));
        let mut best = (f32::INFINITY, ' ');
        for &candidate in &candidates {
            let candidate_score = score(candidate);
            let wins_tie = candidate_score == best.0 && candidate < best.1;
            if candidate_score < best.0 || wins_tie {
                best = (candidate_score, candidate);
            }
        }

        // Never-worse rule: an intermediate must beat BOTH endpoints. Without
        // a qualifying intermediate the nearer endpoint renders.
        if best.1 != a && best.1 != b && best.0 < score_a && best.0 < score_b {
            best.1
        } else if local < 0.5 {
            a
        } else {
            b
        }
    }

    pub fn graph(&self) -> &NeighborGraph {
        &self.graph
    }
}

/// Bounded-cache contract check helper: the caches clear wholesale on
/// overflow (deterministic — same query sequence, same clears).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GlyphTileRaster, GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH};

    fn solid_tile() -> GlyphTileRaster {
        GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: [255u8; MASK_PIXELS],
        }
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
            vec![('#', solid_tile()), (' ', empty_tile())]
        }
    }

    struct GradientSet(Vec<(char, GlyphTileRaster)>);
    impl FadeTileProvider for GradientSet {
        fn tiles(&self) -> Vec<(char, GlyphTileRaster)> {
            self.0.clone()
        }
    }

    fn tile_rect(x0: usize, y0: usize, x1: usize, y1: usize) -> GlyphTileRaster {
        let mut tile = empty_tile();
        for y in y0..=y1 {
            for x in x0..=x1 {
                tile.alpha[y * GLYPH_TILE_WIDTH + x] = 255;
            }
        }
        tile
    }

    fn tile_outline(x0: usize, y0: usize, x1: usize, y1: usize) -> GlyphTileRaster {
        let mut tile = empty_tile();
        for x in x0..=x1 {
            tile.alpha[y0 * GLYPH_TILE_WIDTH + x] = 255;
            tile.alpha[y1 * GLYPH_TILE_WIDTH + x] = 255;
        }
        for y in y0..=y1 {
            tile.alpha[y * GLYPH_TILE_WIDTH + x0] = 255;
            tile.alpha[y * GLYPH_TILE_WIDTH + x1] = 255;
        }
        tile
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
    fn fading_to_space_dissolves_through_the_lighter_endpoint_side() {
        let mut fade = ShapeFade::build(&TwoTiles);
        // Late in the fade the ideal blend is mostly empty: only space (or a
        // candidate closer to empty, of which there are none here) qualifies.
        let late = fade.resolve_shape_fade('#', ' ', 0.9).unwrap();
        assert_eq!(late, ' ');
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
        let mut fade = ShapeFade::build(&GradientSet(vec![
            ('O', tile_outline(1, 1, 10, 14)),
            ('o', tile_outline(4, 5, 8, 11)),
            ('.', tile_rect(5, 7, 6, 8)),
            ('#', tile_rect(0, 0, 11, 15)),
        ]));
        for from in ['O', 'o', '.', '#'] {
            for to in ['O', 'o', '.', '#'] {
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
        assert!(fade.graph().all_pairs_have_paths());
    }

    #[test]
    fn resolution_is_deterministic_and_reversal_symmetric_at_extremes() {
        let mut fade = ShapeFade::build(&GradientSet(vec![
            ('O', tile_outline(1, 1, 10, 14)),
            ('o', tile_outline(4, 5, 8, 11)),
            ('.', tile_rect(5, 7, 6, 8)),
        ]));
        let first = fade.resolve_shape_fade('O', '.', 0.5).unwrap();
        let second = fade.resolve_shape_fade('O', '.', 0.5).unwrap();
        assert_eq!(first, second, "cache hit must equal cold path");
        let mut fade = ShapeFade::build(&GradientSet(vec![
            ('O', tile_outline(1, 1, 10, 14)),
            ('o', tile_outline(4, 5, 8, 11)),
            ('.', tile_rect(5, 7, 6, 8)),
        ]));
        assert_eq!(fade.resolve_shape_fade('O', '.', 0.0), Some('O'));
        assert_eq!(fade.resolve_shape_fade('.', 'O', 1.0), Some('O'));
    }

    #[test]
    fn intermediate_graphics_come_from_the_loaded_set() {
        let mut fade = ShapeFade::build(&GradientSet(vec![
            ('O', tile_outline(1, 1, 10, 14)),
            ('o', tile_outline(4, 5, 8, 11)),
            ('.', tile_rect(5, 7, 6, 8)),
        ]));
        for step in 0..=PROGRESS_BUCKETS {
            let t = step as f32 / PROGRESS_BUCKETS as f32;
            let resolved = fade.resolve_shape_fade('O', '.', t).unwrap();
            assert!(
                ['O', 'o', '.'].contains(&resolved),
                "intermediate {resolved} is not a loaded graphic"
            );
        }
    }

    #[test]
    fn the_never_worse_rule_holds_on_a_two_tile_set() {
        // With only two tiles there are no intermediates: every t must resolve
        // to one of the endpoints, and the crossing must be a single flip.
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
        assert_eq!(flips, 1, "a two-tile fade flips once, never oscillates");
    }
}
