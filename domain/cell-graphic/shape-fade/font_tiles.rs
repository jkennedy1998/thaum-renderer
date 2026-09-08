//! Production fade-tile provider (J 2026-09-07): builds the shape-fade graph
//! over a loaded `GlyphFontSet` at one canonical weight, with the same
//! sprite-over-font precedence `rasterize_glyph_tile` applies. The char sweep
//! comes from the set's own coverage (`glyph_chars`). Chars that rasterize to
//! an empty tile are dropped so the graph never gains phantom duplicates of
//! the empty mask — the intentional space node stays, because an empty mask
//! IS the dissolve endpoint.

use super::mask_space::PackedMask;
use super::neighbor_graph::{FadeTileProvider, WeightedFadeTileProvider};
use crate::{CellWeight, GlyphFontSet, GlyphTileRaster};

/// The canonical weight the graph builds at (see contract notes: one
/// canonical weight; per-weight graphs are a future child only if the
/// canonical graph proves too coarse).
pub const FADE_CANONICAL_WEIGHT: CellWeight = CellWeight::One;

pub struct FontSetTiles<'a> {
    pub font_set: &'a GlyphFontSet,
}

impl FadeTileProvider for FontSetTiles<'_> {
    fn tiles(&self) -> Vec<(char, GlyphTileRaster)> {
        self.tiles_at_weight(FADE_CANONICAL_WEIGHT)
    }
}

impl WeightedFadeTileProvider for FontSetTiles<'_> {
    fn tiles_at_weight(&self, weight: CellWeight) -> Vec<(char, GlyphTileRaster)> {
        self.font_set
            .glyph_chars()
            .into_iter()
            .map(|glyph| (glyph, self.font_set.rasterize_glyph_tile(glyph, weight)))
            .filter(|(glyph, tile)| {
                *glyph == ' ' || PackedMask::from_tile(tile).coverage_count() > 0
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_set_tiles_keeps_space_and_drops_empty_tiles() {
        // Skips quietly when no typeface can load from the repo asset root —
        // the assertion targets the provider's shape, not font availability.
        let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../orchestration/renderer-assets");
        let Ok(font_set) = GlyphFontSet::load_from_asset_root(&asset_root) else {
            return;
        };
        let provider = FontSetTiles {
            font_set: &font_set,
        };
        for weight in [
            CellWeight::Zero,
            CellWeight::One,
            CellWeight::Two,
            CellWeight::Three,
        ] {
            let tiles = provider.tiles_at_weight(weight);
            assert!(tiles.iter().any(|(glyph, _)| *glyph == ' '));
            assert!(tiles.iter().any(|(glyph, _)| *glyph == 'O'));
            assert!(tiles
                .iter()
                .all(|(glyph, tile)| *glyph == ' ' || tile.coverage_count() > 0));
        }
    }
}
