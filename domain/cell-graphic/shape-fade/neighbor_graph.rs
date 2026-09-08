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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH};

    fn rect_tile(x0: usize, y0: usize, x1: usize, y1: usize) -> GlyphTileRaster {
        let mut tile = GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: [0u8; GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT],
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
                    alpha: [0u8; GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT],
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
    fn graphs_are_deterministic_across_rebuilds() {
        let a = NeighborGraph::build(&gradient_set());
        let b = NeighborGraph::build(&gradient_set());
        for index in 0..a.chars.len() {
            assert_eq!(a.neighbors_of(index), b.neighbors_of(index));
        }
    }
}
