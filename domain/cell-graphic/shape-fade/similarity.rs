//! Shape-fade similarity (J 2026-09-07, second pass): the metric is pure L1
//! alpha distance over the two 12x16 tiles — strictly image-based. No glyph
//! identity, no character semantics, no name weighting anywhere in the fade;
//! two graphics are close exactly when their ink sits in the same places.
//! Normalized to 0..=1: identical tiles cost 0, a tile and its total inverse
//! (full ink against empty) cost 1.

use crate::shape_fade::mask_space::MASK_PIXELS;
use crate::GlyphTileRaster;

/// L1 alpha distance in 0..=1: the mean absolute per-pixel alpha difference,
/// normalized by full-scale ink. The distance the neighbor graph ranks by and
/// the fade resolves against.
pub fn alpha_distance(a: &GlyphTileRaster, b: &GlyphTileRaster) -> f32 {
    let total: u64 = a
        .alpha
        .iter()
        .zip(b.alpha.iter())
        .map(|(x, y)| (i32::from(*x) - i32::from(*y)).unsigned_abs() as u64)
        .sum();
    total as f32 / (255.0 * MASK_PIXELS as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH};

    fn tile_with_coverage(indices: &[usize]) -> GlyphTileRaster {
        let mut tile = GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: [0u8; MASK_PIXELS],
        };
        for &index in indices {
            tile.alpha[index] = 255;
        }
        tile
    }

    #[test]
    fn identical_tiles_cost_nothing() {
        let tile = tile_with_coverage(&[0, 5, 30]);
        assert_eq!(alpha_distance(&tile, &tile), 0.0);
    }

    #[test]
    fn full_ink_against_empty_costs_the_maximum() {
        let full = GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: [255u8; MASK_PIXELS],
        };
        let empty = GlyphTileRaster {
            width: GLYPH_TILE_WIDTH,
            height: GLYPH_TILE_HEIGHT,
            alpha: [0u8; MASK_PIXELS],
        };
        assert_eq!(alpha_distance(&full, &empty), 1.0);
        assert_eq!(
            alpha_distance(&empty, &empty),
            0.0,
            "space to space is identity"
        );
    }

    #[test]
    fn distance_is_symmetric_and_proportional_to_disagreement() {
        let a = tile_with_coverage(&[0, 1, 2, 3]);
        let b = tile_with_coverage(&[2, 3, 4, 5]);
        let distance = alpha_distance(&a, &b);
        assert_eq!(distance, alpha_distance(&b, &a));
        // 4 differing pixels at full alpha out of 192.
        assert_eq!(distance, 4.0 / 192.0);
        let far = tile_with_coverage(&[50, 51, 52, 53]);
        assert!(alpha_distance(&a, &far) > distance);
    }
}
