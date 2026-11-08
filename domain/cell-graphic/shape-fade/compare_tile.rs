//! Fixed-bounded shape-fade comparison tiles. Render tiles may be any cell
//! size; this module reduces them once at font-set load time so the fade bake
//! stays bounded while retaining the source tile's aspect ratio.

use crate::GlyphTileRaster;

/// Largest comparison footprint. Actual comparison tiles fit inside this box
/// and preserve the render tile's aspect ratio.
pub const COMPARISON_MAX_WIDTH: usize = 12;
pub const COMPARISON_MAX_HEIGHT: usize = 16;
pub const COMPARISON_MAX_PIXELS: usize = COMPARISON_MAX_WIDTH * COMPARISON_MAX_HEIGHT;

/// The comparison-tile dimensions a render tile of `width`x`height` would
/// downsample to — pure sizing, no alpha data required. Lets a caller size
/// an editable comparison-resolution surface (project 6kbosu52's detailer
/// tool subcell canvas) before any render tile exists to downsample.
pub fn comparison_dimensions(width: usize, height: usize) -> (usize, usize) {
    assert!(
        width > 0 && height > 0,
        "comparison input must have dimensions"
    );
    let scale = (COMPARISON_MAX_WIDTH as f32 / width as f32)
        .min(COMPARISON_MAX_HEIGHT as f32 / height as f32)
        .min(1.0);
    let out_width = ((width as f32 * scale).round() as usize).max(1);
    let out_height = ((height as f32 * scale).round() as usize).max(1);
    (out_width, out_height)
}

/// Downsample `tile` into the fixed comparison bounds using area averages.
/// At least one pixel survives on each axis, so a valid render tile always
/// yields a valid comparison tile.
pub fn downsample_comparison_tile(tile: &GlyphTileRaster) -> GlyphTileRaster {
    assert!(
        tile.width > 0 && tile.height > 0,
        "comparison input must have dimensions"
    );
    assert_eq!(
        tile.alpha.len(),
        tile.width * tile.height,
        "comparison input alpha must match dimensions"
    );

    let (width, height) = comparison_dimensions(tile.width, tile.height);
    let mut alpha = vec![0; width * height];

    for output_y in 0..height {
        let y0 = output_y * tile.height / height;
        let y1 = ((output_y + 1) * tile.height).div_ceil(height);
        for output_x in 0..width {
            let x0 = output_x * tile.width / width;
            let x1 = ((output_x + 1) * tile.width).div_ceil(width);
            let mut sum = 0u64;
            let mut count = 0u64;
            for source_y in y0..y1 {
                for source_x in x0..x1 {
                    sum += u64::from(tile.alpha[source_y * tile.width + source_x]);
                    count += 1;
                }
            }
            alpha[output_y * width + output_x] = (sum / count) as u8;
        }
    }
    GlyphTileRaster {
        width,
        height,
        alpha,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_dimensions_matches_the_downsampled_tile_shape_without_needing_alpha_data() {
        assert_eq!(comparison_dimensions(120, 40), (12, 4));
        assert_eq!(comparison_dimensions(3, 2), (3, 2));
        // JetBrains Mono's 10x22 profile: bounded on height, not upscaled on width.
        assert_eq!(comparison_dimensions(10, 22), (7, 16));
    }

    #[test]
    fn bounds_large_render_tiles_while_preserving_aspect_ratio() {
        let tile = GlyphTileRaster {
            width: 120,
            height: 40,
            alpha: vec![255; 120 * 40],
        };
        let comparison = downsample_comparison_tile(&tile);
        assert_eq!((comparison.width, comparison.height), (12, 4));
        assert_eq!(comparison.alpha.len(), 48);
        assert!(comparison.alpha.iter().all(|value| *value == 255));
    }

    #[test]
    fn small_render_tiles_are_not_upscaled() {
        let tile = GlyphTileRaster {
            width: 3,
            height: 2,
            alpha: vec![255; 6],
        };
        let comparison = downsample_comparison_tile(&tile);
        assert_eq!((comparison.width, comparison.height), (3, 2));
    }
}
