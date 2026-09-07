//! Shape-fade mask space (J 2026-09-07): packs a 12x16 graphic tile's alpha
//! into a binary coverage mask — `[u64; 3]`, one bit per pixel — so shape
//! similarity is popcount arithmetic. The empty mask (the space glyph's tile)
//! is the dissolve state: it has similarity zero to every non-empty shape and
//! one to itself. Owned by `cell-graphic/shape-fade`; see the contract for the
//! ownership split (tile production stays in `glyph/` / `sprite/`).

use crate::{GlyphTileRaster, GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH};

pub const MASK_PIXELS: usize = GLYPH_TILE_WIDTH * GLYPH_TILE_HEIGHT;
const MASK_WORDS: usize = MASK_PIXELS.div_ceil(64);

/// Binary coverage mask over one 12x16 tile. Bit `i` is pixel `i`
/// (row-major); a pixel counts as covered when its alpha is at least half
/// coverage — both font and sprite tiles are anti-aliased u8 alpha, so the
/// midpoint threshold keeps faint edge halo out of the shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedMask([u64; MASK_WORDS]);

impl PackedMask {
    pub fn from_alpha(alpha: &[u8]) -> Self {
        debug_assert_eq!(
            alpha.len(),
            MASK_PIXELS,
            "mask input must be one 12x16 tile"
        );
        let mut words = [0u64; MASK_WORDS];
        for (index, &value) in alpha.iter().enumerate() {
            if value >= 128 {
                words[index / 64] |= 1u64 << (index % 64);
            }
        }
        Self(words)
    }

    pub fn from_tile(tile: &GlyphTileRaster) -> Self {
        Self::from_alpha(&tile.alpha)
    }

    pub fn empty() -> Self {
        Self([0u64; MASK_WORDS])
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|word| *word == 0)
    }

    pub fn coverage_count(&self) -> u32 {
        self.0.iter().map(|word| word.count_ones()).sum()
    }

    pub fn intersection_count(&self, other: &Self) -> u32 {
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| (a & b).count_ones())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn packing_sets_one_bit_per_half_covered_pixel() {
        let mut alpha = [0u8; MASK_PIXELS];
        alpha[0] = 255;
        alpha[1] = 127;
        alpha[2] = 128;
        alpha[191] = 200;
        let mask = PackedMask::from_alpha(&alpha);
        assert_eq!(mask.coverage_count(), 3, "sub-threshold halo must stay out");
        assert!(!mask.is_empty());
    }

    #[test]
    fn empty_alpha_packs_to_the_dissolve_mask() {
        let mask = PackedMask::from_alpha(&[0u8; MASK_PIXELS]);
        assert!(mask.is_empty());
        assert_eq!(mask.coverage_count(), 0);
    }

    #[test]
    fn from_tile_preserves_the_tile_shape() {
        let tile = tile_with_coverage(&[0, 1, 2, 3]);
        let mask = PackedMask::from_tile(&tile);
        assert_eq!(mask.coverage_count(), 4);
    }

    #[test]
    fn intersection_counts_only_shared_pixels() {
        let a = PackedMask::from_alpha(&{
            let mut alpha = [0u8; MASK_PIXELS];
            alpha[0] = 255;
            alpha[1] = 255;
            alpha[2] = 255;
            alpha
        });
        let b = PackedMask::from_alpha(&{
            let mut alpha = [0u8; MASK_PIXELS];
            alpha[1] = 255;
            alpha[2] = 255;
            alpha[3] = 255;
            alpha
        });
        assert_eq!(a.intersection_count(&b), 2);
        assert_eq!(b.intersection_count(&a), 2, "intersection is symmetric");
    }
}
