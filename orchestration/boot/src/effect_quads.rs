use anyhow::Result;
use thaum_renderer_domain::{CellTexture, CellWarble, SpriteTileRaster, WorldPoint};
use thaum_renderer_window_surface::{
    SurfaceQuad, SurfaceQuadPostEffectBus, SurfaceSize, SURFACE_QUAD_NO_ATLAS,
};

use crate::{CELL_HEIGHT_CLIP_SPACE, GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH};

const TEXTURED_FOOTPRINT_OUTSET_PIXELS: f32 = 1.0;
const TEXTURED_TYPEGRID_OVERLAP_SCALE: f32 = 0.06;

fn quad_frame_for_texel(
    column_index: usize,
    row_index: usize,
    width: usize,
    height: usize,
    cell_center: [f32; 2],
    cell_clip_size: [f32; 2],
    pixel_size: [f32; 2],
    textured_footprint: bool,
) -> ([f32; 2], [f32; 2]) {
    let base_center = [
        cell_center[0] + (column_index as f32 + 0.5) * pixel_size[0] - cell_clip_size[0] * 0.5,
        cell_center[1] + cell_clip_size[1] * 0.5 - (row_index as f32 + 0.5) * pixel_size[1],
    ];

    if !textured_footprint {
        return (base_center, pixel_size);
    }

    let mut left = cell_center[0] - cell_clip_size[0] * 0.5 + column_index as f32 * pixel_size[0];
    let mut right = left + pixel_size[0];
    let mut top = cell_center[1] + cell_clip_size[1] * 0.5 - row_index as f32 * pixel_size[1];
    let mut bottom = top - pixel_size[1];

    {
        let x_outset = pixel_size[0] * TEXTURED_FOOTPRINT_OUTSET_PIXELS;
        let y_outset = pixel_size[1] * TEXTURED_FOOTPRINT_OUTSET_PIXELS;

        if column_index == 0 {
            left -= x_outset;
        }
        if column_index + 1 == width {
            right += x_outset;
        }
        if row_index == 0 {
            top += y_outset;
        }
        if row_index + 1 == height {
            bottom -= y_outset;
        }

        let x_overlap = pixel_size[0] * TEXTURED_TYPEGRID_OVERLAP_SCALE;
        let y_overlap = pixel_size[1] * TEXTURED_TYPEGRID_OVERLAP_SCALE;
        left -= x_overlap * 0.5;
        right += x_overlap * 0.5;
        bottom -= y_overlap * 0.5;
        top += y_overlap * 0.5;
    }

    (
        [(left + right) * 0.5, (bottom + top) * 0.5],
        [right - left, top - bottom],
    )
}

fn quad_frame_for_texture_buffer_texel(
    column_index: i32,
    row_index: i32,
    width: usize,
    height: usize,
    cell_center: [f32; 2],
    cell_clip_size: [f32; 2],
    pixel_size: [f32; 2],
) -> ([f32; 2], [f32; 2]) {
    let mut left = cell_center[0] - cell_clip_size[0] * 0.5 + column_index as f32 * pixel_size[0];
    let mut right = left + pixel_size[0];
    let mut top = cell_center[1] + cell_clip_size[1] * 0.5 - row_index as f32 * pixel_size[1];
    let mut bottom = top - pixel_size[1];

    let x_outset = pixel_size[0] * TEXTURED_FOOTPRINT_OUTSET_PIXELS;
    let y_outset = pixel_size[1] * TEXTURED_FOOTPRINT_OUTSET_PIXELS;

    if column_index == -1 {
        left -= x_outset;
    }
    if column_index == width as i32 {
        right += x_outset;
    }
    if row_index == -1 {
        top += y_outset;
    }
    if row_index == height as i32 {
        bottom -= y_outset;
    }

    let x_overlap = pixel_size[0] * TEXTURED_TYPEGRID_OVERLAP_SCALE;
    let y_overlap = pixel_size[1] * TEXTURED_TYPEGRID_OVERLAP_SCALE;
    left -= x_overlap * 0.5;
    right += x_overlap * 0.5;
    bottom -= y_overlap * 0.5;
    top += y_overlap * 0.5;

    (
        [(left + right) * 0.5, (bottom + top) * 0.5],
        [right - left, top - bottom],
    )
}

fn texel_has_visible_neighbor(
    visible_texels: &[bool],
    width: usize,
    height: usize,
    column_index: i32,
    row_index: i32,
) -> bool {
    for neighbor_row in row_index - 1..=row_index + 1 {
        for neighbor_column in column_index - 1..=column_index + 1 {
            if neighbor_column == column_index && neighbor_row == row_index {
                continue;
            }
            if neighbor_column < 0
                || neighbor_row < 0
                || neighbor_column >= width as i32
                || neighbor_row >= height as i32
            {
                continue;
            }
            if visible_texels[neighbor_row as usize * width + neighbor_column as usize] {
                return true;
            }
        }
    }

    false
}

fn local_uv_corners_for_texel(
    column_index: i32,
    row_index: i32,
    width: usize,
    height: usize,
) -> [[f32; 2]; 4] {
    let left = column_index as f32 / width as f32;
    let right = (column_index + 1) as f32 / width as f32;
    let top = row_index as f32 / height as f32;
    let bottom = (row_index + 1) as f32 / height as f32;

    [[left, bottom], [right, bottom], [right, top], [left, top]]
}

fn warble_uv_corners_for_texel(
    column_index: i32,
    row_index: i32,
    width: usize,
    height: usize,
    world: WorldPoint,
) -> [[f32; 2]; 4] {
    let left = world.x as f32 + column_index as f32 / width as f32 - 0.5;
    let right = world.x as f32 + (column_index + 1) as f32 / width as f32 - 0.5;
    let top = world.y as f32 + 0.5 - row_index as f32 / height as f32;
    let bottom = world.y as f32 + 0.5 - (row_index + 1) as f32 / height as f32;

    [[left, bottom], [right, bottom], [right, top], [left, top]]
}

fn push_texture_buffer_ring(
    quads: &mut Vec<SurfaceQuad>,
    visible_texels: &[bool],
    width: usize,
    height: usize,
    cell_center: [f32; 2],
    cell_clip_size: [f32; 2],
    pixel_size: [f32; 2],
    post_effect_bus: SurfaceQuadPostEffectBus,
    world: WorldPoint,
) {
    for row_index in -1..=height as i32 {
        for column_index in -1..=width as i32 {
            let inside_authored_bounds = column_index >= 0
                && column_index < width as i32
                && row_index >= 0
                && row_index < height as i32;
            if inside_authored_bounds
                || !texel_has_visible_neighbor(
                    visible_texels,
                    width,
                    height,
                    column_index,
                    row_index,
                )
            {
                continue;
            }

            let (center, size) = quad_frame_for_texture_buffer_texel(
                column_index,
                row_index,
                width,
                height,
                cell_center,
                cell_clip_size,
                pixel_size,
            );
            quads.push(SurfaceQuad {
                center,
                size,
                color: [0.0, 0.0, 0.0, 0.0],
                local_uv_corners: local_uv_corners_for_texel(
                    column_index,
                    row_index,
                    width,
                    height,
                ),
                warble_uv_corners: warble_uv_corners_for_texel(
                    column_index,
                    row_index,
                    width,
                    height,
                    world,
                ),
                post_effect_bus,
                atlas_uv: SURFACE_QUAD_NO_ATLAS,
            });
        }
    }
}

pub(crate) fn sprite_raster_to_surface_quads(
    raster: &SpriteTileRaster,
    threshold: u8,
    cell_center: [f32; 2],
    cell_clip_size: [f32; 2],
    texture: CellTexture,
    warble: CellWarble,
    depth_code: u8,
    gate_id: u16,
    world: WorldPoint,
) -> Result<Vec<SurfaceQuad>> {
    let pixel_size = [
        cell_clip_size[0] / raster.width as f32,
        cell_clip_size[1] / raster.height as f32,
    ];
    let mut quads = Vec::new();
    let post_effect_bus = SurfaceQuadPostEffectBus {
        texture_code: texture.code(),
        warble_code: warble.code(),
        depth_code,
        gate_id,
    };
    let emit_expanded_post_effect_footprint = !texture.is_none() || !warble.is_none();
    let visible_texels = raster
        .colors
        .iter()
        .map(|color| {
            color
                .map(|value| (value[3] * 255.0).round() as u8 >= threshold)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    for row_index in 0..raster.height {
        for column_index in 0..raster.width {
            let color = raster.colors[row_index * raster.width + column_index]
                .map(|mut color| {
                    let coverage = (color[3] * 255.0).round() as u8;
                    color[3] = if coverage < threshold {
                        0.0
                    } else {
                        coverage as f32 / 255.0
                    };
                    color
                })
                .unwrap_or([0.0, 0.0, 0.0, 0.0]);
            let is_visible = color[3] > 0.0;

            if !is_visible
                && (!emit_expanded_post_effect_footprint
                    || !texel_has_visible_neighbor(
                        &visible_texels,
                        raster.width,
                        raster.height,
                        column_index as i32,
                        row_index as i32,
                    ))
            {
                continue;
            }

            let (center, size) = quad_frame_for_texel(
                column_index,
                row_index,
                raster.width,
                raster.height,
                cell_center,
                cell_clip_size,
                pixel_size,
                emit_expanded_post_effect_footprint && !is_visible,
            );
            quads.push(SurfaceQuad {
                center,
                size,
                color,
                local_uv_corners: local_uv_corners_for_texel(
                    column_index as i32,
                    row_index as i32,
                    raster.width,
                    raster.height,
                ),
                warble_uv_corners: warble_uv_corners_for_texel(
                    column_index as i32,
                    row_index as i32,
                    raster.width,
                    raster.height,
                    world,
                ),
                post_effect_bus,
                atlas_uv: SURFACE_QUAD_NO_ATLAS,
            });
        }
    }

    if emit_expanded_post_effect_footprint {
        push_texture_buffer_ring(
            &mut quads,
            &visible_texels,
            raster.width,
            raster.height,
            cell_center,
            cell_clip_size,
            pixel_size,
            post_effect_bus,
            world,
        );
    }

    Ok(quads)
}

// Phase C candidate: sprite path still uses this; glyph path moved to the atlas.
#[allow(dead_code)]
pub(crate) fn raster_to_surface_quads(
    alpha: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    binary_alpha: bool,
    cell_center: [f32; 2],
    cell_clip_size: [f32; 2],
    color: [f32; 4],
    texture: CellTexture,
    warble: CellWarble,
    depth_code: u8,
    gate_id: u16,
    world: WorldPoint,
) -> Result<Vec<SurfaceQuad>> {
    let pixel_size = [
        cell_clip_size[0] / width as f32,
        cell_clip_size[1] / height as f32,
    ];
    let mut quads = Vec::new();
    let post_effect_bus = SurfaceQuadPostEffectBus {
        texture_code: texture.code(),
        warble_code: warble.code(),
        depth_code,
        gate_id,
    };
    let emit_expanded_post_effect_footprint = !texture.is_none() || !warble.is_none();
    let visible_texels = alpha
        .iter()
        .map(|coverage| *coverage >= threshold)
        .collect::<Vec<_>>();

    for row_index in 0..height {
        for column_index in 0..width {
            let coverage = alpha[row_index * width + column_index];
            let is_visible = coverage >= threshold;

            if !is_visible
                && (!emit_expanded_post_effect_footprint
                    || !texel_has_visible_neighbor(
                        &visible_texels,
                        width,
                        height,
                        column_index as i32,
                        row_index as i32,
                    ))
            {
                continue;
            }

            let mut quad_color = color;
            quad_color[3] *= if !is_visible {
                0.0
            } else if binary_alpha {
                1.0
            } else {
                coverage as f32 / 255.0
            };

            let (center, size) = quad_frame_for_texel(
                column_index,
                row_index,
                width,
                height,
                cell_center,
                cell_clip_size,
                pixel_size,
                emit_expanded_post_effect_footprint && !is_visible,
            );
            quads.push(SurfaceQuad {
                center,
                size,
                color: quad_color,
                local_uv_corners: local_uv_corners_for_texel(
                    column_index as i32,
                    row_index as i32,
                    width,
                    height,
                ),
                warble_uv_corners: warble_uv_corners_for_texel(
                    column_index as i32,
                    row_index as i32,
                    width,
                    height,
                    world,
                ),
                post_effect_bus,
                atlas_uv: SURFACE_QUAD_NO_ATLAS,
            });
        }
    }

    if emit_expanded_post_effect_footprint {
        push_texture_buffer_ring(
            &mut quads,
            &visible_texels,
            width,
            height,
            cell_center,
            cell_clip_size,
            pixel_size,
            post_effect_bus,
            world,
        );
    }

    Ok(quads)
}

pub(crate) fn cell_clip_size_for_surface(surface_size: SurfaceSize) -> [f32; 2] {
    let surface_width = surface_size.width.max(1) as f32;
    let surface_height = surface_size.height.max(1) as f32;
    let pixel_aspect_compensation = surface_height / surface_width;
    let cell_width = CELL_HEIGHT_CLIP_SPACE
        * (GLYPH_TILE_WIDTH as f32 / GLYPH_TILE_HEIGHT as f32)
        * pixel_aspect_compensation;

    [cell_width, CELL_HEIGHT_CLIP_SPACE]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_world() -> WorldPoint {
        WorldPoint::origin()
    }

    fn quad_has_any_local_uv_outside_authored_tile(quad: &SurfaceQuad) -> bool {
        quad.local_uv_corners
            .iter()
            .any(|uv| uv[0] < 0.0 || uv[0] > 1.0 || uv[1] < 0.0 || uv[1] > 1.0)
    }

    fn quad_has_all_local_uv_inside_authored_tile(quad: &SurfaceQuad) -> bool {
        quad.local_uv_corners
            .iter()
            .all(|uv| uv[0] >= 0.0 && uv[0] <= 1.0 && uv[1] >= 0.0 && uv[1] <= 1.0)
    }

    #[test]
    fn textured_glyphs_emit_a_texture_buffer_around_visible_texels() {
        let quads = raster_to_surface_quads(
            &[255, 0, 255, 0],
            2,
            2,
            128,
            true,
            [0.0, 0.0],
            [1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            CellTexture::new(1),
            CellWarble::none(),
            128,
            7,
            test_world(),
        )
        .unwrap();

        assert_eq!(quads.len(), 12);
        assert_eq!(quads.iter().filter(|quad| quad.color[3] > 0.0).count(), 2);
        assert!(quads
            .iter()
            .all(|quad| quad.post_effect_bus.texture_code == 1));
    }

    #[test]
    fn untextured_glyphs_still_only_emit_visible_pixels() {
        let quads = raster_to_surface_quads(
            &[255, 0, 255, 0],
            2,
            2,
            128,
            true,
            [0.0, 0.0],
            [1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            CellTexture::none(),
            CellWarble::none(),
            128,
            7,
            test_world(),
        )
        .unwrap();

        assert_eq!(quads.len(), 2);
        assert!(quads.iter().all(|quad| quad.color[3] > 0.0));
    }

    #[test]
    fn textured_invisible_footprint_quads_outset_without_misplacing_visible_glyph_texels() {
        let quads = raster_to_surface_quads(
            &[255, 0, 0, 255],
            2,
            2,
            128,
            true,
            [0.0, 0.0],
            [1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            CellTexture::new(1),
            CellWarble::none(),
            128,
            7,
            test_world(),
        )
        .unwrap();

        let visible_quads = quads
            .iter()
            .filter(|quad| quad.color[3] > 0.0)
            .collect::<Vec<_>>();
        let invisible_quads = quads
            .iter()
            .filter(|quad| quad.color[3] == 0.0)
            .collect::<Vec<_>>();
        let ring_quads = invisible_quads
            .iter()
            .filter(|quad| quad_has_any_local_uv_outside_authored_tile(quad))
            .collect::<Vec<_>>();

        assert!(visible_quads.iter().all(|quad| quad.size == [0.5, 0.5]));
        assert!(invisible_quads.iter().all(|quad| quad.size[0] > 0.5));
        assert!(invisible_quads.iter().all(|quad| quad.size[1] > 0.5));
        assert_eq!(ring_quads.len(), 10);
    }

    #[test]
    fn warbled_glyphs_emit_the_same_expanded_post_effect_footprint() {
        let quads = raster_to_surface_quads(
            &[255, 0, 255, 0],
            2,
            2,
            128,
            true,
            [0.0, 0.0],
            [1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            CellTexture::none(),
            CellWarble::new(1),
            128,
            7,
            test_world(),
        )
        .unwrap();

        assert_eq!(quads.len(), 12);
        assert_eq!(quads.iter().filter(|quad| quad.color[3] > 0.0).count(), 2);
        assert!(quads
            .iter()
            .all(|quad| quad.post_effect_bus.warble_code == 1));
    }

    #[test]
    fn texture_buffer_skips_far_void_texels_inside_the_tile() {
        let quads = raster_to_surface_quads(
            &[0, 0, 0, 0, 255, 0, 0, 0, 0],
            3,
            3,
            128,
            true,
            [0.0, 0.0],
            [1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            CellTexture::new(1),
            CellWarble::none(),
            128,
            7,
            test_world(),
        )
        .unwrap();

        let interior_invisible = quads
            .iter()
            .filter(|quad| quad.color[3] == 0.0 && quad_has_all_local_uv_inside_authored_tile(quad))
            .count();

        assert_eq!(quads.len(), 9);
        assert_eq!(interior_invisible, 8);
    }
}

/// Per-frame placement of the scene's glyph atlas: maps (glyph, weight) keys
/// to their whole-cell quad sampling frame. Built by the boot atlas pass.
#[derive(Debug, Default)]
pub(crate) struct GlyphAtlasPlacement {
    pub data: thaum_renderer_window_surface::GlyphAtlasSceneData,
    pub slots: std::collections::HashMap<(char, u32), usize>,
}

impl GlyphAtlasPlacement {
    fn tile_uv(&self, glyph: char, weight_index: u32) -> Option<[f32; 4]> {
        let slot = *self.slots.get(&(glyph, weight_index))?;
        let columns = self.data.columns.max(1) as usize;
        let rows = self.data.rows.max(1) as usize;
        let tile_width = self.data.tile_width.max(1) as usize;
        let tile_height = self.data.tile_height.max(1) as usize;
        let column = slot % columns;
        let row = slot / columns;
        let atlas_width = (columns * tile_width) as f32;
        let atlas_height = (rows * tile_height) as f32;
        Some([
            (column * tile_width) as f32 / atlas_width,
            (row * tile_height) as f32 / atlas_height,
            tile_width as f32 / atlas_width,
            tile_height as f32 / atlas_height,
        ])
    }
}

/// A glyph cell emits exactly one quad covering the whole 12×16 cell. The
/// quad-pass fragment shader samples the glyph atlas for coverage (v=0 at the
/// tile top, matching `local_uv` interpolation), while the bus and aux data
/// cover the entire cell rect so post-pass displaced sampling needs no ring
/// quads. See `context/glyph-atlas-geometry-reduction.md`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn glyph_cell_to_surface_quad(
    placement: &GlyphAtlasPlacement,
    glyph: char,
    weight_index: u32,
    cell_center: [f32; 2],
    cell_clip_size: [f32; 2],
    color: [f32; 4],
    texture: CellTexture,
    warble: CellWarble,
    depth_code: u8,
    gate_id: u16,
    world: WorldPoint,
) -> Result<SurfaceQuad> {
    let atlas_uv = placement
        .tile_uv(glyph, weight_index)
        .ok_or_else(|| anyhow::anyhow!("glyph '{glyph}' missing from the built glyph atlas"))?;

    Ok(SurfaceQuad {
        center: cell_center,
        size: cell_clip_size,
        color,
        local_uv_corners: [
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
        ],
        warble_uv_corners: [
            [world.x as f32 - 0.5, world.y as f32 - 0.5],
            [world.x as f32 + 0.5, world.y as f32 - 0.5],
            [world.x as f32 + 0.5, world.y as f32 + 0.5],
            [world.x as f32 - 0.5, world.y as f32 + 0.5],
        ],
        post_effect_bus: SurfaceQuadPostEffectBus {
            texture_code: texture.code(),
            warble_code: warble.code(),
            depth_code,
            gate_id,
        },
        atlas_uv,
    })
}
