//! Shape-fade neighbor graph (J 2026-09-07, second pass): every loaded
//! graphic becomes a node; each node keeps its k nearest shapes by pure L1
//! alpha distance. The graph no longer routes multi-step paths — the fade
//! walk's multiple steps emerge from the per-step projection (see `fade`)
//! picking different interpolative glyphs as the ideal blend advances toward
//! the target. Routing paths through intermediates made walks drift away
//! from the endpoint shapes (J's quality feedback): the projection now always
//! targets the global blend of the two endpoints, and this graph only supplies
//! the candidate pool. Build runs once per tile-set load.

use std::collections::HashMap;

use crate::shape_fade::similarity::alpha_distance;
use crate::{CellWeight, GlyphTileRaster};

/// Neighbor list size per graphic. Small enough to keep candidate pools local
/// in image space, large enough that shape-plausible intermediates appear.
pub const FADE_NEIGHBOR_COUNT: usize = 8;

/// Supplies the loaded tile set the graph is built from. Production builds
/// this over the glyph font set (font glyphs + sprite tiles, sprite-over-font
/// precedence); tests build it over hand-made tiles.
pub trait FadeTileProvider {
    fn tiles(&self) -> Vec<(char, GlyphTileRaster)>;
}

/// Supplies one tile set for every render weight. Weight-aware shape fades use
/// the real source, target, and output-weight tiles instead of treating one
/// canonical glyph weight as every cell's shape.
pub trait WeightedFadeTileProvider {
    /// Number of native render weights represented by this provider.
    fn weight_count(&self) -> usize;
    fn tiles_at_weight(&self, weight: CellWeight) -> Vec<(char, GlyphTileRaster)>;
}

pub struct NeighborGraph {
    /// Node order is provider order deduplicated by char — stable and
    /// deterministic for a given tile set.
    pub chars: Vec<char>,
    pub tiles: Vec<GlyphTileRaster>,
    /// Per node: (neighbor index, alpha distance), sorted by (distance, char).
    neighbors: Vec<Vec<(usize, f32)>>,
}

impl NeighborGraph {
    pub fn build(provider: &dyn FadeTileProvider) -> Self {
        let mut chars: Vec<char> = Vec::new();
        let mut tiles: Vec<GlyphTileRaster> = Vec::new();
        let mut index_of: HashMap<char, usize> = HashMap::new();
        for (glyph, tile) in provider.tiles() {
            if index_of.contains_key(&glyph) {
                continue; // first occurrence wins; the tile seam already dedupes by key
            }
            index_of.insert(glyph, chars.len());
            chars.push(glyph);
            tiles.push(tile);
        }
        let count = chars.len();
        let mut neighbors: Vec<Vec<(usize, f32)>> = vec![Vec::new(); count];
        for a in 0..count {
            for b in 0..count {
                if a == b {
                    continue;
                }
                let distance = alpha_distance(&tiles[a], &tiles[b]);
                neighbors[a].push((b, distance));
            }
            neighbors[a].sort_by(|(ia, da), (ib, db)| {
                da.partial_cmp(db)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(chars[*ia].cmp(&chars[*ib]))
            });
            neighbors[a].truncate(FADE_NEIGHBOR_COUNT);
        }
        Self {
            chars,
            tiles,
            neighbors,
        }
    }

    pub fn index_of(&self, glyph: char) -> Option<usize> {
        self.chars.iter().position(|candidate| *candidate == glyph)
    }

    pub fn tile(&self, index: usize) -> &GlyphTileRaster {
        &self.tiles[index]
    }

    pub fn neighbors_of(&self, index: usize) -> &[(usize, f32)] {
        &self.neighbors[index]
    }

    /// The loaded glyph nearest `tile` by alpha distance, with its distance —
    /// for resolving an arbitrary hand-edited comparison tile (not one of
    /// this graph's own nodes) to the closest real glyph, e.g. project
    /// 6kbosu52's detailer tool. Always snaps to the nearest match regardless
    /// of how far it is; `None` only for an empty graph. Deterministic
    /// tie-break: smallest distance, then lowest char.
    pub fn nearest_to(&self, tile: &GlyphTileRaster) -> Option<(char, f32)> {
        self.nearest_to_excluding(tile, None)
    }

    /// `nearest_to` with one glyph barred from winning — the detailer
    /// (project 6kbosu52) excludes a stroke's seed glyph so a hand edit
    /// always resolves to a different, visible shape instead of snapping
    /// back to what the cell already had.
    pub fn nearest_to_excluding(
        &self,
        tile: &GlyphTileRaster,
        exclude: Option<char>,
    ) -> Option<(char, f32)> {
        self.tiles
            .iter()
            .enumerate()
            .filter(|(index, _)| Some(self.chars[*index]) != exclude)
            .map(|(index, candidate)| (self.chars[index], alpha_distance(tile, candidate)))
            .min_by(|(char_a, distance_a), (char_b, distance_b)| {
                distance_a
                    .total_cmp(distance_b)
                    .then_with(|| char_a.cmp(char_b))
            })
    }

    /// The nearest candidate that actually carries ink (any nonzero alpha),
    /// skipping blank tiles — the detailer's additive resolve (project
    /// 6kbosu52): adding ink to an empty cell must be able to produce a real
    /// inked glyph, and the blank tile is every sparse mark's nearest
    /// neighbor, which resolves such strokes to the invisible space glyph.
    /// Same distance and tie-break rules as `nearest_to`.
    pub fn nearest_inked_to(&self, tile: &GlyphTileRaster) -> Option<(char, f32)> {
        self.nearest_inked_to_excluding(tile, None)
    }

    /// `nearest_inked_to` with one glyph barred from winning (see
    /// `nearest_to_excluding`).
    pub fn nearest_inked_to_excluding(
        &self,
        tile: &GlyphTileRaster,
        exclude: Option<char>,
    ) -> Option<(char, f32)> {
        self.tiles
            .iter()
            .enumerate()
            .filter(|(index, _)| Some(self.chars[*index]) != exclude)
            .filter(|(_, candidate)| candidate.alpha.iter().any(|&alpha| alpha > 0))
            .map(|(index, candidate)| (self.chars[index], alpha_distance(tile, candidate)))
            .min_by(|(char_a, distance_a), (char_b, distance_b)| {
                distance_a
                    .total_cmp(distance_b)
                    .then_with(|| char_a.cmp(char_b))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH};

    fn rect_tile(x0: usize, y0: usize, x1: usize, y1: usize) -> GlyphTileRaster {
        let mut tile = GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: vec![0u8; GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT],
        };
        for y in y0..=y1 {
            for x in x0..=x1 {
                tile.alpha[y * GLYPH_TILE_WIDTH + x] = 255;
            }
        }
        tile
    }

    struct TestSet(Vec<(char, GlyphTileRaster)>);
    impl FadeTileProvider for TestSet {
        fn tiles(&self) -> Vec<(char, GlyphTileRaster)> {
            self.0.clone()
        }
    }

    fn gradient_set() -> TestSet {
        TestSet(vec![
            (
                ' ',
                GlyphTileRaster {
                    width: GLYPH_TILE_WIDTH,
                    height: GLYPH_TILE_HEIGHT,
                    alpha: vec![0u8; GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT],
                },
            ),
            ('L', rect_tile(0, 0, 5, 15)),
            ('C', rect_tile(2, 0, 8, 15)),
            ('R', rect_tile(6, 0, 11, 15)),
        ])
    }

    #[test]
    fn neighbors_rank_by_image_distance() {
        let graph = NeighborGraph::build(&gradient_set());
        let left = graph.index_of('L').unwrap();
        let neighbors: Vec<char> = graph
            .neighbors_of(left)
            .iter()
            .map(|(index, _)| graph.chars[*index])
            .collect();
        assert_eq!(
            neighbors.first(),
            Some(&'C'),
            "the overlapping middle strip is the nearest shape"
        );
        // Space (96 px of L disagree with it) beats R (160 px disagree).
        assert_eq!(neighbors[1], ' ');
        assert_eq!(neighbors[2], 'R');
    }

    #[test]
    fn duplicate_provider_entries_dedupe_to_one_node() {
        let mut tiles = gradient_set().0;
        let first_c = tiles[2].1.clone();
        tiles.push(('C', first_c));
        let graph = NeighborGraph::build(&TestSet(tiles));
        assert_eq!(graph.chars.iter().filter(|c| **c == 'C').count(), 1);
    }

    #[test]
    fn a_pair_of_unknowns_has_no_pool_entry() {
        let graph = NeighborGraph::build(&gradient_set());
        assert!(graph.index_of('Ω').is_none());
    }

    #[test]
    fn nearest_to_finds_the_exact_match_at_zero_distance() {
        let graph = NeighborGraph::build(&gradient_set());
        let left_tile = rect_tile(0, 0, 5, 15);
        assert_eq!(graph.nearest_to(&left_tile), Some(('L', 0.0)));
    }

    #[test]
    fn nearest_to_finds_the_closest_candidate_for_a_novel_buffer() {
        // A hand-drawn buffer that is not any loaded glyph: slightly
        // narrower than 'L' but still much closer to it than to 'C' or 'R'.
        let graph = NeighborGraph::build(&gradient_set());
        let drawn = rect_tile(0, 0, 4, 15);
        let (nearest, distance) = graph.nearest_to(&drawn).unwrap();
        assert_eq!(nearest, 'L');
        assert!(distance > 0.0, "a novel buffer is not an exact match");
    }

    #[test]
    fn nearest_to_ties_break_on_the_lowest_char() {
        // 'R' and 'C' are equal-size blocks, so both sit at the same L1
        // distance from the empty probe; the tie-break picks 'C' purely by
        // char ordering (it was registered second, after 'R').
        let set = TestSet(vec![
            ('R', rect_tile(0, 0, 5, 15)),
            ('C', rect_tile(6, 0, 11, 15)),
        ]);
        let graph = NeighborGraph::build(&set);
        let probe = GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: vec![0u8; GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT],
        };
        let (nearest, _) = graph.nearest_to(&probe).unwrap();
        assert_eq!(nearest, 'C');
    }

    #[test]
    fn nearest_inked_to_skips_the_blank_candidate() {
        // The blank probe is the exact match of the blank tile, but inked
        // candidates are the only ones allowed to win: a sparse hand mark
        // resolves to a real glyph, never to the invisible space glyph.
        let graph = NeighborGraph::build(&gradient_set());
        let probe = GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: vec![0u8; GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT],
        };
        let (nearest, _) = graph.nearest_inked_to(&probe).unwrap();
        assert_eq!(
            nearest, 'L',
            "blank would win nearest_to, but the smallest inked shape wins nearest_inked_to"
        );
    }

    #[test]
    fn nearest_inked_to_still_ranks_inked_candidates_by_distance() {
        let graph = NeighborGraph::build(&gradient_set());
        let drawn = rect_tile(0, 0, 4, 15);
        let (nearest, distance) = graph.nearest_inked_to(&drawn).unwrap();
        assert_eq!(nearest, 'L');
        assert!(distance > 0.0, "a novel buffer is not an exact match");
    }

    #[test]
    fn graphs_are_deterministic_across_rebuilds() {
        let a = NeighborGraph::build(&gradient_set());
        let b = NeighborGraph::build(&gradient_set());
        for index in 0..a.chars.len() {
            assert_eq!(a.neighbors_of(index), b.neighbors_of(index));
        }
    }
}
