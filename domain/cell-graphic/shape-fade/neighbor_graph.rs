//! Shape-fade neighbor graph (J 2026-09-07): every loaded graphic becomes a
//! node keyed by its packed mask; each node keeps its k nearest shapes. The
//! fade between ANY pair routes over this graph — a hop-limited Dijkstra from
//! one endpoint to the other, with a direct edge to the target always in the
//! pool so every pair has a path even across disconnected shape components.
//! Intermediates are real loaded glyphs/sprites (the graph's nodes), never
//! invented shapes. Build runs once per tile-set load; paths resolve lazily
//! per pair and cache in the facade (`fade`).

use std::collections::HashMap;

use crate::shape_fade::mask_space::PackedMask;
use crate::shape_fade::similarity::shape_distance;
use crate::GlyphTileRaster;

/// Neighbor list size per graphic. Small enough to keep candidate pools local
/// in shape space, large enough that most pairs bridge within a couple hops.
pub const FADE_NEIGHBOR_COUNT: usize = 8;

/// Maximum edges a fade path may use. Beyond this, a fade would wander — the
/// direct edge fallback still guarantees the pair resolves.
pub const MAX_FADE_HOPS: usize = 4;

/// Per-edge cost surcharge so equal-distance routes prefer fewer hops.
const HOP_PENALTY: f32 = 0.05;

/// Supplies the loaded tile set the graph is built from. Production builds
/// this over the glyph font set (font glyphs + sprite tiles, sprite-over-font
/// precedence); tests build it over hand-made masks.
pub trait FadeTileProvider {
    fn tiles(&self) -> Vec<(char, GlyphTileRaster)>;
}

pub struct NeighborGraph {
    /// Node order is provider order deduplicated by char — stable and
    /// deterministic for a given tile set.
    pub chars: Vec<char>,
    pub tiles: Vec<GlyphTileRaster>,
    pub masks: Vec<PackedMask>,
    /// Per node: (neighbor index, shape distance), sorted by (distance, char).
    neighbors: Vec<Vec<(usize, f32)>>,
}

impl NeighborGraph {
    pub fn build(provider: &dyn FadeTileProvider) -> Self {
        let mut chars: Vec<char> = Vec::new();
        let mut tiles: Vec<GlyphTileRaster> = Vec::new();
        let mut masks: Vec<PackedMask> = Vec::new();
        let mut index_of: HashMap<char, usize> = HashMap::new();
        for (glyph, tile) in provider.tiles() {
            if index_of.contains_key(&glyph) {
                continue; // first occurrence wins; the tile seam already dedupes by key
            }
            index_of.insert(glyph, chars.len());
            chars.push(glyph);
            tiles.push(tile);
            masks.push(PackedMask::from_tile(&tiles[masks.len()]));
        }
        let count = chars.len();
        let mut neighbors: Vec<Vec<(usize, f32)>> = vec![Vec::new(); count];
        for a in 0..count {
            for b in 0..count {
                if a == b {
                    continue;
                }
                let distance = shape_distance(&masks[a], &masks[b]);
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
            masks,
            neighbors,
        }
    }

    pub fn index_of(&self, glyph: char) -> Option<usize> {
        self.chars.iter().position(|candidate| *candidate == glyph)
    }

    pub fn mask(&self, index: usize) -> &PackedMask {
        &self.masks[index]
    }

    pub fn tile(&self, index: usize) -> &GlyphTileRaster {
        &self.tiles[index]
    }

    pub fn neighbors_of(&self, index: usize) -> &[(usize, f32)] {
        &self.neighbors[index]
    }

    /// The multi-step fade path from `from` to `to`: a hop-limited Dijkstra
    /// over the kNN edges plus a direct edge to the target (so any known pair
    /// resolves even when no chain of near shapes connects them). Returns the
    /// node path including both endpoints, or `None` when either endpoint is
    /// unknown to the graph. Deterministic: linear-scan Dijkstra over sorted
    /// adjacency, no hash iteration.
    pub fn fade_path(&self, from: char, to: char) -> Option<Vec<char>> {
        let start = self.index_of(from)?;
        let target = self.index_of(to)?;
        if start == target {
            return Some(vec![from]);
        }

        let count = self.chars.len();
        let mut dist: Vec<f32> = vec![f32::INFINITY; count];
        let mut hops: Vec<usize> = vec![usize::MAX; count];
        let mut prev: Vec<Option<usize>> = vec![None; count];
        let mut settled: Vec<bool> = vec![false; count];
        dist[start] = 0.0;
        hops[start] = 0;

        for _ in 0..count {
            let current = (0..count).filter(|index| !settled[*index]).min_by(|a, b| {
                dist[*a]
                    .total_cmp(&dist[*b])
                    .then_with(|| self.chars[*a].cmp(&self.chars[*b]))
            })?;
            if dist[current] == f32::INFINITY {
                break;
            }
            settled[current] = true;

            // The direct edge to the target rides alongside the kNN edges so a
            // bad or component-less pair still resolves (never-worse rule).
            let mut edges: Vec<(usize, f32)> = self.neighbors[current].to_vec();
            if current == start {
                edges.push((
                    target,
                    shape_distance(&self.masks[start], &self.masks[target]),
                ));
            }
            for &(next, weight) in &edges {
                if hops[current] + 1 > MAX_FADE_HOPS {
                    continue;
                }
                let candidate = dist[current] + weight + HOP_PENALTY;
                let fewer_hops = hops[current] + 1 < hops[next];
                if candidate < dist[next] || (candidate == dist[next] && fewer_hops) {
                    dist[next] = candidate;
                    hops[next] = hops[current] + 1;
                    prev[next] = Some(current);
                }
            }
        }

        if hops[target] == usize::MAX {
            return None;
        }
        let mut path = vec![target];
        let mut cursor = target;
        while let Some(previous) = prev[cursor] {
            path.push(previous);
            cursor = previous;
        }
        path.reverse();
        Some(path.into_iter().map(|index| self.chars[index]).collect())
    }

    /// Every known pair must have a path — the any-pair guarantee. Cheap to
    /// assert after mutations to the tile set in tests.
    pub fn all_pairs_have_paths(&self) -> bool {
        for a in &self.chars {
            for b in &self.chars {
                if self.fade_path(*a, *b).is_none() {
                    return false;
                }
            }
        }
        true
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

    /// A set with a shape gradient to walk: three vertical strips where the
    /// middle strip overlaps each end by more than half its pixels while the
    /// two ends are pixel-disjoint — the exact regime where a multi-hop route
    /// beats the direct edge (Dice distance is near-metric, so bridges must
    /// genuinely overlap; real font glyph neighbors do, synthetic rings don't).
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
            ('C', rect_tile(2, 0, 9, 15)),
            ('R', rect_tile(6, 0, 11, 15)),
        ])
    }

    #[test]
    fn neighbors_rank_by_shape_distance() {
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
        assert_eq!(
            neighbors.last(),
            Some(&'R'),
            "the disjoint strip ranks last (tied with space, char-ordered)"
        );
    }

    #[test]
    fn close_pairs_fade_directly() {
        let graph = NeighborGraph::build(&gradient_set());
        let path = graph.fade_path('L', 'C').unwrap();
        assert_eq!(path, vec!['L', 'C']);
    }

    #[test]
    fn far_pairs_route_through_interpolative_shapes() {
        let graph = NeighborGraph::build(&gradient_set());
        let path = graph.fade_path('L', 'R').unwrap();
        assert_eq!(
            path,
            vec!['L', 'C', 'R'],
            "disjoint strips should route through the overlapping middle strip"
        );
    }

    #[test]
    fn every_pair_has_a_path_even_disjoint_ones() {
        let graph = NeighborGraph::build(&gradient_set());
        assert!(
            graph.all_pairs_have_paths(),
            "the direct-edge fallback guarantees any-pair fades"
        );
        // Space is disconnected from everything (empty mask, similarity zero):
        // its fade still resolves, as the direct dissolve.
        assert_eq!(graph.fade_path('L', ' ').unwrap(), vec!['L', ' ']);
        assert_eq!(graph.fade_path('R', ' ').unwrap(), vec!['R', ' ']);
    }

    #[test]
    fn a_pair_of_unknowns_resolves_nothing() {
        let graph = NeighborGraph::build(&gradient_set());
        assert!(graph.fade_path('O', 'Ω').is_none());
        assert!(graph.fade_path('Ω', 'O').is_none());
    }

    #[test]
    fn identical_graphics_are_a_one_node_path() {
        let graph = NeighborGraph::build(&gradient_set());
        assert_eq!(graph.fade_path('L', 'L').unwrap(), vec!['L']);
    }

    #[test]
    fn duplicate_provider_entries_dedupe_to_one_node() {
        let mut tiles = gradient_set().0;
        let first_o = tiles[2].1.clone();
        tiles.push(('o', first_o));
        let graph = NeighborGraph::build(&TestSet(tiles));
        assert_eq!(graph.chars.iter().filter(|c| **c == 'o').count(), 1);
    }

    #[test]
    fn paths_are_deterministic_across_rebuilds() {
        let a = NeighborGraph::build(&gradient_set());
        let b = NeighborGraph::build(&gradient_set());
        for from in [' ', 'L', 'C', 'R'] {
            for to in [' ', 'L', 'C', 'R'] {
                assert_eq!(a.fade_path(from, to), b.fade_path(from, to));
            }
        }
    }
}
