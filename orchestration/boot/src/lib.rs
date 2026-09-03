use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, Result};
use thaum_renderer_breath_fallback_clock::{FallbackBreathClock, FALLBACK_BREATH_TICK_DURATION};
use thaum_renderer_domain::{
    apply_debug_depth_post_effect_to_rgba, apply_debug_texture_post_effect_to_rgba,
    apply_debug_warble_post_effect_to_rgba, encode_relative_depth_to_post_effect_bus,
    project_flat_2d_world_to_view_plane, project_rotating_3d_world_to_view_plane,
    projected_plane_is_visible, projected_plane_scale_factor, resolve_shaded_graphic,
    resolve_shaded_texture, resolve_shaded_warble, resolve_shaded_weight, Camera,
    CameraProjectedPoint, Cell, CellGroupIntakeBehavior, CellPoint, Composition, DataLanes,
    GlyphFontSet, IndexColorClampEffect, SpriteAtlasSet, WorldPoint, GLYPH_TILE_HEIGHT,
    GLYPH_TILE_WIDTH,
};
pub use thaum_renderer_window_surface::{
    run_window_surface_with_frame_provider, SurfaceQuad, SurfaceSize, WindowSurfaceConfig,
    WindowSurfaceFrameContext, WindowSurfaceInput, WindowSurfaceScene,
};

mod effect_quads;

use effect_quads::{
    cell_clip_size_for_surface, raster_to_surface_quads, sprite_raster_to_surface_quads,
};

#[cfg(test)]
use thaum_renderer_domain::glyph_font_path;

const CELL_HEIGHT_CLIP_SPACE: f32 = 0.2;
const GLYPH_BINARY_ALPHA_THRESHOLD: u8 = 0x80;
const SPRITE_BINARY_ALPHA_THRESHOLD: u8 = 0x80;

#[derive(Debug, Clone)]
pub struct BootConfig {
    pub asset_root: PathBuf,
    pub hot_reload: bool,
    pub index_color_clamp: IndexColorClampEffect,
    pub depth_of_field_post_effect: bool,
    pub motion_noise_post_effect: bool,
    pub depth_of_field_minimum_falloff_cells: f32,
    pub fog_span_cells: f32,
    pub surface_cull_bleed_cells: f32,
    pub debug_texture_post_effect: bool,
    pub debug_warble_post_effect: bool,
    pub debug_depth_post_effect: bool,
    pub window: WindowSurfaceConfig,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            asset_root: PathBuf::from("."),
            hot_reload: false,
            index_color_clamp: IndexColorClampEffect::default(),
            depth_of_field_post_effect: false,
            motion_noise_post_effect: true,
            depth_of_field_minimum_falloff_cells: 5.0,
            fog_span_cells: 5.0,
            surface_cull_bleed_cells: 2.0,
            debug_texture_post_effect: false,
            debug_warble_post_effect: false,
            debug_depth_post_effect: false,
            window: WindowSurfaceConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BootState {
    pub camera: Camera,
    pub composition: Composition,
    pub data_lanes: DataLanes,
    pub config: BootConfig,
    pub uses_fallback_breath: bool,
}

#[derive(Default)]
struct RendererAssetCache {
    glyph_fonts: RefCell<Option<GlyphFontSet>>,
    sprite_atlases: RefCell<Option<SpriteAtlasSet>>,
}

impl RendererAssetCache {
    fn ensure_glyph_fonts(&self, asset_root: &Path, hot_reload: bool) -> Result<()> {
        let already_loaded = self.glyph_fonts.borrow().is_some();
        if hot_reload || !already_loaded {
            let loaded =
                GlyphFontSet::load_from_asset_root(asset_root).map_err(anyhow::Error::msg)?;
            *self.glyph_fonts.borrow_mut() = Some(loaded);
        }
        Ok(())
    }

    fn glyph_fonts(&self) -> Ref<'_, Option<GlyphFontSet>> {
        self.glyph_fonts.borrow()
    }

    fn ensure_sprite_atlases(&self, asset_root: &Path, hot_reload: bool) {
        let already_loaded = self.sprite_atlases.borrow().is_some();
        if hot_reload || !already_loaded {
            *self.sprite_atlases.borrow_mut() =
                Some(SpriteAtlasSet::load_from_asset_root(asset_root));
        }
    }

    fn sprite_atlases(&self) -> RefMut<'_, Option<SpriteAtlasSet>> {
        self.sprite_atlases.borrow_mut()
    }
}

#[derive(Debug, Clone)]
struct ProjectedBootCell {
    world: WorldPoint,
    projected: CameraProjectedPoint,
    cell: Cell,
}

#[derive(Debug, Clone)]
struct ProjectedBootPlane {
    plane: i32,
    cells: Vec<ProjectedBootCell>,
}

pub fn boot_renderer(config: BootConfig) -> Result<BootState> {
    boot_renderer_with_data_lanes(config, DataLanes::default())
}

pub fn boot_renderer_with_data_lanes(
    config: BootConfig,
    data_lanes: DataLanes,
) -> Result<BootState> {
    let uses_fallback_breath = !data_lanes.has_breath();

    Ok(BootState {
        camera: Camera::default(),
        composition: Composition::default(),
        data_lanes: resolve_boot_data_lanes(data_lanes),
        config,
        uses_fallback_breath,
    })
}

fn resolve_boot_data_lanes(mut data_lanes: DataLanes) -> DataLanes {
    data_lanes.set_fallback_breath_if_unset(FallbackBreathClock::default().current());
    data_lanes
}

pub fn run_renderer_window(config: BootConfig) -> Result<()> {
    let state = boot_renderer(config)?;
    run_renderer_window_with_state(state)
}

pub fn run_renderer_window_with_state(state: BootState) -> Result<()> {
    run_renderer_window_with_state_frame_provider(state, |_, _| Ok(()))
}

pub fn run_renderer_window_with_state_frame_provider(
    state: BootState,
    mut frame_provider: impl FnMut(&mut BootState, &WindowSurfaceFrameContext) -> Result<()> + 'static,
) -> Result<()> {
    let mut frame_state = state;
    let mut fallback_breath =
        FallbackBreathClock::new(frame_state.data_lanes.breath().unwrap_or(0));
    let mut last_tick = Instant::now();
    let window_config = frame_state.config.window.clone();
    let asset_cache = RendererAssetCache::default();

    run_window_surface_with_frame_provider(window_config, move |frame| {
        let now = Instant::now();
        if frame_state.uses_fallback_breath {
            while now.duration_since(last_tick) >= FALLBACK_BREATH_TICK_DURATION {
                fallback_breath.tick();
                last_tick += FALLBACK_BREATH_TICK_DURATION;
            }
            frame_state.data_lanes.set_breath(fallback_breath.current());
        }

        frame_provider(&mut frame_state, &frame)?;
        build_window_surface_scene_for_surface_with_cache(
            &frame_state,
            frame.surface_size,
            &asset_cache,
        )
    })
}

pub fn build_window_surface_scene(state: &BootState) -> Result<WindowSurfaceScene> {
    build_window_surface_scene_for_surface(state, SurfaceSize::from(&state.config.window))
}

pub fn build_window_surface_scene_for_surface(
    state: &BootState,
    surface_size: SurfaceSize,
) -> Result<WindowSurfaceScene> {
    build_window_surface_scene_for_surface_with_cache(
        state,
        surface_size,
        &RendererAssetCache::default(),
    )
}

/// The camera-zoom-adjusted cell clip size used to render `state`'s
/// composition for a given surface size. A consumer converting a clip-space
/// cursor/click position into a world/cell coordinate (via
/// `thaum_renderer_domain::remap_surface_units_to_active_plane_world`) must
/// use this same value so hit-testing agrees with what was actually drawn.
pub fn cell_clip_size_for_state(state: &BootState, surface_size: SurfaceSize) -> [f32; 2] {
    let base = cell_clip_size_for_surface(surface_size);
    [base[0] * state.camera.zoom, base[1] * state.camera.zoom]
}

fn build_window_surface_scene_for_surface_with_cache(
    state: &BootState,
    surface_size: SurfaceSize,
    asset_cache: &RendererAssetCache,
) -> Result<WindowSurfaceScene> {
    let visible_stack = thaum_renderer_domain::visible_plane_stack_for_camera(state.camera);
    let fog_nearest_depth_code = encode_relative_depth_to_post_effect_bus(visible_stack.min_plane);
    let fog_farthest_depth_code = encode_relative_depth_to_post_effect_bus(visible_stack.max_plane);
    let mut scene = WindowSurfaceScene {
        texture_breath: state.data_lanes.breath().unwrap_or(0),
        depth_of_field_enabled: state.config.depth_of_field_post_effect,
        motion_noise_enabled: state.config.motion_noise_post_effect,
        indexed_color_enabled: state.config.index_color_clamp.is_active(),
        indexed_color_palette: state.config.index_color_clamp.palette.clone(),
        depth_of_field_minimum_falloff_cells: state.config.depth_of_field_minimum_falloff_cells,
        fog_span_cells: 0.0,
        fog_nearest_depth_code,
        fog_farthest_depth_code,
        background_color: [
            state.config.window.clear_color[0] as f32,
            state.config.window.clear_color[1] as f32,
            state.config.window.clear_color[2] as f32,
            state.config.window.clear_color[3] as f32,
        ],
        ..WindowSurfaceScene::default()
    };
    if composition_contains_visible_glyphs(&state.composition) {
        asset_cache.ensure_glyph_fonts(&state.config.asset_root, state.config.hot_reload)?;
    }
    if composition_contains_visible_sprites(&state.composition) {
        asset_cache.ensure_sprite_atlases(&state.config.asset_root, state.config.hot_reload);
    }
    let glyph_fonts_borrow = asset_cache.glyph_fonts();
    let glyph_fonts = glyph_fonts_borrow.as_ref();
    let mut sprite_atlases_borrow = asset_cache.sprite_atlases();
    let base_cell_clip_size = cell_clip_size_for_surface(surface_size);
    let cell_clip_size = [
        base_cell_clip_size[0] * state.camera.zoom,
        base_cell_clip_size[1] * state.camera.zoom,
    ];

    for plane in
        group_projected_boot_cells_by_plane(stage_projected_boot_cells(state, cell_clip_size))
    {
        let _plane_index = plane.plane;
        for projected_cell in plane.cells {
            scene.quads.extend(project_cell_to_surface_quads(
                state.camera,
                projected_cell,
                state.data_lanes,
                glyph_fonts,
                sprite_atlases_borrow.as_mut(),
                cell_clip_size,
            )?);
        }
    }

    apply_depth_edge_fade_to_scene(
        &mut scene,
        effective_depth_edge_fade_span_cells(
            state.config.fog_span_cells,
            visible_stack.planes.len(),
        ),
        fog_nearest_depth_code,
        fog_farthest_depth_code,
    );

    if state.config.debug_warble_post_effect {
        apply_debug_warble_post_effect_to_scene(&mut scene);
    }
    if state.config.debug_texture_post_effect {
        apply_debug_texture_post_effect_to_scene(&mut scene);
    }
    if state.config.debug_depth_post_effect {
        apply_debug_depth_post_effect_to_scene(&mut scene);
    }

    Ok(scene)
}

fn apply_debug_warble_post_effect_to_scene(scene: &mut WindowSurfaceScene) {
    for quad in &mut scene.quads {
        quad.color =
            apply_debug_warble_post_effect_to_rgba(quad.color, quad.post_effect_bus.warble_code);
    }
}

fn apply_debug_texture_post_effect_to_scene(scene: &mut WindowSurfaceScene) {
    for quad in &mut scene.quads {
        quad.color =
            apply_debug_texture_post_effect_to_rgba(quad.color, quad.post_effect_bus.texture_code);
    }
}

fn apply_debug_depth_post_effect_to_scene(scene: &mut WindowSurfaceScene) {
    for quad in &mut scene.quads {
        quad.color =
            apply_debug_depth_post_effect_to_rgba(quad.color, quad.post_effect_bus.depth_code);
    }
}

fn depth_edge_fade_amount_for_edge(distance_to_edge: f32, fade_span_cells: f32) -> f32 {
    if fade_span_cells <= 0.0 || distance_to_edge < 0.0 {
        return 0.0;
    }

    let span = fade_span_cells.max(1.0);
    if span <= 1.0 {
        return if distance_to_edge == 0.0 { 0.9 } else { 0.0 };
    }

    let normalized = ((span - 1.0 - distance_to_edge) / (span - 1.0)).clamp(0.0, 1.0);
    match normalized {
        n if n <= 0.0 => 0.0,
        n if n >= 1.0 => 0.9,
        n => {
            let anchors = [0.1_f32, 0.3, 0.6, 0.8, 0.9];
            let scaled = n * 4.0;
            let low_index = scaled.floor() as usize;
            let high_index = (low_index + 1).min(4);
            let local_t = scaled.fract();
            anchors[low_index] + (anchors[high_index] - anchors[low_index]) * local_t
        }
    }
}

fn effective_depth_edge_fade_span_cells(
    base_fade_span_cells: f32,
    visible_plane_count: usize,
) -> f32 {
    match visible_plane_count {
        0..=2 => 0.0,
        3 => base_fade_span_cells * (1.0 / 3.0),
        4 => base_fade_span_cells * (2.0 / 3.0),
        _ => base_fade_span_cells,
    }
}

fn apply_depth_edge_fade_to_scene(
    scene: &mut WindowSurfaceScene,
    fade_span_cells: f32,
    nearest_depth_code: u8,
    farthest_depth_code: u8,
) {
    for quad in &mut scene.quads {
        let depth_code = quad.post_effect_bus.depth_code;
        let distance_to_near_edge = depth_code as f32 - nearest_depth_code as f32;
        let distance_to_far_edge = farthest_depth_code as f32 - depth_code as f32;
        let fade_amount =
            depth_edge_fade_amount_for_edge(distance_to_near_edge, fade_span_cells).max(
                depth_edge_fade_amount_for_edge(distance_to_far_edge, fade_span_cells),
            );
        quad.color[3] *= 1.0 - fade_amount;
    }
}

fn composition_contains_visible_glyphs(composition: &Composition) -> bool {
    composition
        .groups
        .iter()
        .flat_map(|group| group.iter_cells())
        .any(|cell| cell.graphic.glyph_char().is_some())
}

fn composition_contains_visible_sprites(composition: &Composition) -> bool {
    composition
        .groups
        .iter()
        .flat_map(|group| group.iter_cells())
        .any(|cell| cell.graphic.sprite().is_some())
}

fn project_cell_to_surface_quads(
    camera: Camera,
    projected_cell: ProjectedBootCell,
    data_lanes: DataLanes,
    glyph_fonts: Option<&GlyphFontSet>,
    sprite_atlases: Option<&mut SpriteAtlasSet>,
    cell_clip_size: [f32; 2],
) -> Result<Vec<SurfaceQuad>> {
    if !projected_cell.cell.graphic.is_visible() {
        return Ok(Vec::new());
    }

    let projected_cell_clip_size =
        projected_cell_clip_size_for_surface(camera, projected_cell.projected, cell_clip_size);
    let cell_center = projected_cell_center_for_surface(projected_cell.projected, cell_clip_size);
    let shaded_graphic = resolve_shaded_graphic(
        projected_cell.cell.graphic.clone(),
        &projected_cell.cell.shader_stack,
        projected_cell.world,
        data_lanes,
    );
    if !shaded_graphic.is_visible() {
        return Ok(Vec::new());
    }
    let shaded_weight = resolve_shaded_weight(
        projected_cell.cell.weight,
        &projected_cell.cell.shader_stack,
        projected_cell.world,
        data_lanes,
    );
    let shaded_texture = resolve_shaded_texture(
        projected_cell.cell.texture,
        &projected_cell.cell.shader_stack,
        projected_cell.world,
        data_lanes,
    );
    let shaded_warble = resolve_shaded_warble(
        projected_cell.cell.warble,
        &projected_cell.cell.shader_stack,
        projected_cell.world,
        data_lanes,
    );
    let depth_code = encode_relative_depth_to_post_effect_bus(projected_cell.projected.plane);
    let gate_id = post_effect_gate_id_for_world_point(projected_cell.world);

    if let Some(glyph) = shaded_graphic.glyph_char() {
        let glyph_fonts = glyph_fonts.context("visible glyph cells require loaded glyph fonts")?;
        let raster = glyph_fonts.rasterize_glyph_tile(glyph, shaded_weight);
        return raster_to_surface_quads(
            &raster.alpha,
            raster.width,
            raster.height,
            GLYPH_BINARY_ALPHA_THRESHOLD,
            true,
            cell_center,
            projected_cell_clip_size,
            projected_cell.cell.color.resolve_glyph(),
            shaded_texture,
            shaded_warble,
            depth_code,
            gate_id,
            projected_cell.world,
        );
    }

    if let Some(sprite) = shaded_graphic.sprite() {
        let sprite_atlases =
            sprite_atlases.context("visible sprite cells require loaded sprite atlases")?;
        let raster = sprite_atlases
            .rasterize_single_sprite_tile(
                sprite.atlas_relative_path(),
                shaded_weight,
                projected_cell.cell.color,
            )
            .map_err(anyhow::Error::msg)?;
        return sprite_raster_to_surface_quads(
            &raster,
            SPRITE_BINARY_ALPHA_THRESHOLD,
            cell_center,
            projected_cell_clip_size,
            shaded_texture,
            shaded_warble,
            depth_code,
            gate_id,
            projected_cell.world,
        );
    }

    Ok(Vec::new())
}

fn projected_cell_intersects_surface(
    camera: Camera,
    projected: CameraProjectedPoint,
    cell_clip_size: [f32; 2],
    bleed_cells: f32,
) -> bool {
    let projected_cell_clip_size =
        projected_cell_clip_size_for_surface(camera, projected, cell_clip_size);
    let cell_center = projected_cell_center_for_surface(projected, cell_clip_size);
    let half_width = projected_cell_clip_size[0] * 0.5;
    let half_height = projected_cell_clip_size[1] * 0.5;
    let bleed_width = projected_cell_clip_size[0] * bleed_cells.max(0.0);
    let bleed_height = projected_cell_clip_size[1] * bleed_cells.max(0.0);

    cell_center[0] + half_width >= -1.0 - bleed_width
        && cell_center[0] - half_width <= 1.0 + bleed_width
        && cell_center[1] + half_height >= -1.0 - bleed_height
        && cell_center[1] - half_height <= 1.0 + bleed_height
}

fn stage_projected_boot_cells(
    state: &BootState,
    cell_clip_size: [f32; 2],
) -> Vec<ProjectedBootCell> {
    let mut staged = Vec::new();
    let mut projected_to_index = HashMap::<(i32, u32, u32), usize>::new();

    for group_index in state.composition.pass_order.iter().copied() {
        let group = state
            .composition
            .groups
            .get(group_index)
            .unwrap_or_else(|| {
                panic!("composition pass_order references missing group index {group_index}")
            });

        for cell in group.iter_cells() {
            let world = group.world_point_for(cell.position);
            let projected = match group.intake_behavior {
                CellGroupIntakeBehavior::Rotating3d => {
                    project_rotating_3d_world_to_view_plane(state.camera, world)
                }
                CellGroupIntakeBehavior::Flat2d => {
                    // Flat2d content is a screen-locked 2D layer: its group
                    // origin is a screen-space offset, not a world anchor, so
                    // it always projects relative to the camera's own focus
                    // target (the active render depth) instead of drifting
                    // as the camera pans between unrelated world anchors.
                    // `hud_pan_offset` is the one thing allowed to move it,
                    // so the 2D layer can be panned on its own.
                    let local = CellPoint {
                        x: group.origin.x + cell.position.x + state.camera.hud_pan_offset.x,
                        y: group.origin.y + cell.position.y + state.camera.hud_pan_offset.y,
                        z: group.origin.z + cell.position.z,
                    };
                    project_flat_2d_world_to_view_plane(
                        state.camera,
                        state.camera.focus_target,
                        local,
                    )
                }
            };

            if !projected_plane_is_visible(state.camera, projected.plane)
                || !projected_cell_intersects_surface(
                    state.camera,
                    projected,
                    cell_clip_size,
                    state.config.surface_cull_bleed_cells,
                )
            {
                continue;
            }

            let key = (
                projected.plane,
                projected.u.to_bits(),
                projected.v.to_bits(),
            );
            let next = ProjectedBootCell {
                world,
                projected,
                cell: cell.clone(),
            };
            if let Some(&index) = projected_to_index.get(&key) {
                // One visible cell per projected position: later pass-order
                // cells win, but a cell whose shader hides it this frame (a
                // flashing overlay in its off half) yields to what is staged
                // beneath it instead of blanking the position.
                if staged_cell_renders_this_frame(&next, state.data_lanes) {
                    staged[index] = next;
                }
            } else {
                let index = staged.len();
                staged.push(next);
                projected_to_index.insert(key, index);
            }
        }
    }

    staged
}

/// Whether a cell would render a visible graphic this frame: its base
/// graphic is visible and no shader in its stack hides it (the vivid flash
/// pair hides its cell during the opposite half of the breath cycle).
fn staged_cell_renders_this_frame(projected_cell: &ProjectedBootCell, data_lanes: DataLanes) -> bool {
    projected_cell.cell.graphic.is_visible()
        && resolve_shaded_graphic(
            projected_cell.cell.graphic.clone(),
            &projected_cell.cell.shader_stack,
            projected_cell.world,
            data_lanes,
        )
        .is_visible()
}

fn group_projected_boot_cells_by_plane(
    projected_cells: Vec<ProjectedBootCell>,
) -> Vec<ProjectedBootPlane> {
    let mut grouped = BTreeMap::<i32, Vec<ProjectedBootCell>>::new();

    for projected_cell in projected_cells {
        grouped
            .entry(projected_cell.projected.plane)
            .or_default()
            .push(projected_cell);
    }

    grouped
        .into_iter()
        .rev()
        .map(|(plane, cells)| ProjectedBootPlane { plane, cells })
        .collect()
}

fn projected_cell_clip_size_for_surface(
    camera: Camera,
    projected: CameraProjectedPoint,
    base_cell_clip_size: [f32; 2],
) -> [f32; 2] {
    let scale = projected_plane_scale_factor(projected.plane, camera.projection_mode);

    [
        base_cell_clip_size[0] * scale,
        base_cell_clip_size[1] * scale,
    ]
}

fn projected_cell_center_for_surface(
    projected: CameraProjectedPoint,
    cell_clip_size: [f32; 2],
) -> [f32; 2] {
    [
        projected.u * cell_clip_size[0],
        projected.v * cell_clip_size[1],
    ]
}

fn post_effect_gate_id_for_world_point(world: WorldPoint) -> u16 {
    let mut hash = 0x811c9dc5u32;

    for coordinate in [world.x, world.y, world.z] {
        for byte in coordinate.to_le_bytes() {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
    }

    ((hash >> 16) as u16) ^ (hash as u16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use thaum_renderer_domain::{
        project_world_to_view_plane, CameraSwing, Cell, CellColor, CellGraphic, CellGroup,
        CellGroupFacing, CellMaterialId, CellPoint, CellWeight, DataLanes,
        CELL_SHADER_TEXTURE_SHIMMER, CELL_SHADER_VIVID_FLASH, CELL_SHADER_VIVID_FLASH_ALT,
        CELL_SHADER_WARBLE_DIAGONAL, CELL_SHADER_WEIGHT_SIN, VIVID_FLASH_BREATH_PERIOD,
    };

    fn staged_asset_root() -> PathBuf {
        if let Ok(path) = std::env::var("THAUM_RENDERER_ASSET_ROOT") {
            return PathBuf::from(path);
        }

        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../renderer-assets")
    }

    #[test]
    fn boot_renderer_seeds_fallback_breath_when_unset() {
        let state = boot_renderer(BootConfig::default()).unwrap();
        assert_eq!(state.data_lanes.breath(), Some(0));
        assert!(state.uses_fallback_breath);
    }

    #[test]
    fn stage_projected_boot_cells_keeps_flat_2d_modules_screen_locked_across_camera_pan_and_swing()
    {
        let module_group = CellGroup::from_cells(
            WorldPoint { x: 5, y: 2, z: 0 },
            [Cell {
                position: CellPoint { x: 1, y: 1, z: 0 },
                graphic: CellGraphic::Glyph('A'),
                ..Cell::default()
            }],
        )
        .with_intake_behavior(CellGroupIntakeBehavior::Flat2d);

        let build_state = |camera: Camera| BootState {
            camera,
            composition: Composition {
                groups: vec![module_group.clone()],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig::default(),
            uses_fallback_breath: false,
        };
        let cell_clip_size =
            cell_clip_size_for_surface(SurfaceSize::from(&BootConfig::default().window));

        let at_origin = build_state(Camera::default());
        let panned = build_state(Camera {
            focus_target: WorldPoint {
                x: 40,
                y: -17,
                z: 6,
            },
            ..Camera::default()
        });
        let swung = build_state(Camera {
            focus_target: WorldPoint {
                x: 40,
                y: -17,
                z: 6,
            },
            swing: CameraSwing::PosX,
            ..Camera::default()
        });

        let origin_staged = stage_projected_boot_cells(&at_origin, cell_clip_size);
        let panned_staged = stage_projected_boot_cells(&panned, cell_clip_size);
        let swung_staged = stage_projected_boot_cells(&swung, cell_clip_size);

        assert_eq!(origin_staged.len(), 1);
        assert_eq!(origin_staged[0].projected, panned_staged[0].projected);
        assert_eq!(origin_staged[0].projected, swung_staged[0].projected);
        assert_eq!(swung_staged[0].projected.plane, 0);
    }

    #[test]
    fn stage_projected_boot_cells_hud_pan_offset_moves_flat_2d_but_not_rotating_3d_cells() {
        let module_group = CellGroup::from_cells(
            WorldPoint { x: 5, y: 2, z: 0 },
            [Cell {
                position: CellPoint { x: 1, y: 1, z: 0 },
                graphic: CellGraphic::Glyph('A'),
                ..Cell::default()
            }],
        )
        .with_intake_behavior(CellGroupIntakeBehavior::Flat2d);
        let scene_group = CellGroup::from_cells(
            WorldPoint { x: 1, y: 1, z: 0 },
            [Cell {
                position: CellPoint { x: 1, y: 1, z: 0 },
                graphic: CellGraphic::Glyph('B'),
                ..Cell::default()
            }],
        );

        let build_state = |camera: Camera| BootState {
            camera,
            composition: Composition {
                groups: vec![module_group.clone(), scene_group.clone()],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig::default(),
            uses_fallback_breath: false,
        };
        let cell_clip_size =
            cell_clip_size_for_surface(SurfaceSize::from(&BootConfig::default().window));

        let at_rest = stage_projected_boot_cells(&build_state(Camera::default()), cell_clip_size);
        let panned = stage_projected_boot_cells(
            &build_state(Camera {
                hud_pan_offset: CellPoint { x: 3, y: -2, z: 0 },
                ..Camera::default()
            }),
            cell_clip_size,
        );

        let flat_at_rest = at_rest
            .iter()
            .find(|c| c.cell.graphic == CellGraphic::Glyph('A'))
            .unwrap();
        let flat_panned = panned
            .iter()
            .find(|c| c.cell.graphic == CellGraphic::Glyph('A'))
            .unwrap();
        let scene_at_rest = at_rest
            .iter()
            .find(|c| c.cell.graphic == CellGraphic::Glyph('B'))
            .unwrap();
        let scene_panned = panned
            .iter()
            .find(|c| c.cell.graphic == CellGraphic::Glyph('B'))
            .unwrap();

        assert_eq!(flat_panned.projected.u, flat_at_rest.projected.u + 3.0);
        assert_eq!(flat_panned.projected.v, flat_at_rest.projected.v - 2.0);
        assert_eq!(scene_panned.projected, scene_at_rest.projected);
    }

    #[test]
    fn stage_projected_boot_cells_flash_pair_yields_to_beneath_cells_instead_of_blank()
    {
        // Document cell, then the two flash halves stacked above it. Each
        // phase must surface the half that renders, and when an overlay half
        // is empty the document cell beneath shows through — no blank state.
        let scene_group = CellGroup::from_cells(
            WorldPoint::origin(),
            [Cell {
                position: CellPoint::origin(),
                graphic: CellGraphic::Glyph('S'),
                ..Cell::default()
            }],
        );
        let alt_group = CellGroup::from_cells(
            WorldPoint::origin(),
            [Cell {
                position: CellPoint::origin(),
                graphic: CellGraphic::Glyph('V'),
                shader_stack: vec![CELL_SHADER_VIVID_FLASH_ALT],
                ..Cell::default()
            }],
        );
        let flash_group = CellGroup::from_cells(
            WorldPoint::origin(),
            [Cell {
                position: CellPoint::origin(),
                graphic: CellGraphic::Glyph('F'),
                shader_stack: vec![CELL_SHADER_VIVID_FLASH],
                ..Cell::default()
            }],
        );
        let build_state = |data_lanes: DataLanes| BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![scene_group.clone(), alt_group.clone(), flash_group.clone()],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes,
            config: BootConfig::default(),
            uses_fallback_breath: false,
        };
        let cell_clip_size =
            cell_clip_size_for_surface(SurfaceSize::from(&BootConfig::default().window));

        // Lit half: the FLASH overlay wins over everything beneath.
        let lit = stage_projected_boot_cells(
            &build_state(DataLanes::with_breath(VIVID_FLASH_BREATH_PERIOD)),
            cell_clip_size,
        );
        assert_eq!(lit.len(), 1);
        assert_eq!(lit[0].cell.graphic, CellGraphic::Glyph('F'));

        // Off half: the FLASH overlay yields to the ALT half beneath it.
        let off =
            stage_projected_boot_cells(&build_state(DataLanes::with_breath(0)), cell_clip_size);
        assert_eq!(off.len(), 1);
        assert_eq!(off[0].cell.graphic, CellGraphic::Glyph('V'));

        // Off half with an empty ALT half: the document cell shows through.
        let empty_alt_group = CellGroup::from_cells(
            WorldPoint::origin(),
            [Cell {
                position: CellPoint::origin(),
                graphic: CellGraphic::None,
                shader_stack: vec![CELL_SHADER_VIVID_FLASH_ALT],
                ..Cell::default()
            }],
        );
        let reveal_state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![scene_group.clone(), empty_alt_group, flash_group.clone()],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::with_breath(0),
            config: BootConfig::default(),
            uses_fallback_breath: false,
        };
        let revealed = stage_projected_boot_cells(&reveal_state, cell_clip_size);
        assert_eq!(revealed.len(), 1);
        assert_eq!(revealed[0].cell.graphic, CellGraphic::Glyph('S'));
    }

    #[test]
    fn stage_projected_boot_cells_reveals_rotating_3d_cells_once_flat_2d_moves_off_them() {
        let scene_group = CellGroup::from_cells(
            WorldPoint::origin(),
            [Cell {
                position: CellPoint::origin(),
                graphic: CellGraphic::Glyph('S'),
                ..Cell::default()
            }],
        );
        let module_group = CellGroup::from_cells(
            WorldPoint::origin(),
            [Cell {
                position: CellPoint::origin(),
                graphic: CellGraphic::Glyph('M'),
                ..Cell::default()
            }],
        )
        .with_intake_behavior(CellGroupIntakeBehavior::Flat2d);
        let build_state = |camera: Camera| BootState {
            camera,
            composition: Composition {
                groups: vec![scene_group.clone(), module_group.clone()],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig::default(),
            uses_fallback_breath: false,
        };
        let cell_clip_size =
            cell_clip_size_for_surface(SurfaceSize::from(&BootConfig::default().window));

        let overlapping =
            stage_projected_boot_cells(&build_state(Camera::default()), cell_clip_size);
        let separated = stage_projected_boot_cells(
            &build_state(Camera {
                hud_pan_offset: CellPoint { x: 2, y: 0, z: 0 },
                ..Camera::default()
            }),
            cell_clip_size,
        );

        assert_eq!(overlapping.len(), 1);
        assert_eq!(overlapping[0].cell.graphic, CellGraphic::Glyph('M'));
        assert_eq!(separated.len(), 2);
        assert!(separated
            .iter()
            .any(|cell| cell.cell.graphic == CellGraphic::Glyph('M')));
        assert!(separated
            .iter()
            .any(|cell| cell.cell.graphic == CellGraphic::Glyph('S')));
    }

    #[test]
    fn boot_renderer_preserves_app_provided_breath_on_boot() {
        let state =
            boot_renderer_with_data_lanes(BootConfig::default(), DataLanes::with_breath(37))
                .unwrap();

        assert_eq!(state.data_lanes.breath(), Some(37));
        assert!(!state.uses_fallback_breath);
    }

    #[test]
    fn stage_projected_boot_cells_keeps_projected_plane_truth() {
        let state = BootState {
            camera: Camera {
                focus_target: WorldPoint { x: 1, y: 0, z: 0 },
                swing: CameraSwing::PosX,
                ..Camera::default()
            },
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [
                        Cell {
                            position: CellPoint { x: 0, y: 0, z: 0 },
                            graphic: CellGraphic::Glyph('A'),
                            ..Cell::default()
                        },
                        Cell {
                            position: CellPoint { x: 3, y: 0, z: 0 },
                            graphic: CellGraphic::Glyph('B'),
                            ..Cell::default()
                        },
                    ],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig::default(),
            uses_fallback_breath: false,
        };

        let staged = stage_projected_boot_cells(
            &state,
            cell_clip_size_for_surface(SurfaceSize::from(&state.config.window)),
        );
        assert_eq!(staged.len(), 2);
        assert_eq!(staged[0].projected.plane, -1);
        assert_eq!(staged[1].projected.plane, 2);
    }

    #[test]
    fn stage_projected_boot_cells_filters_planes_outside_camera_window() {
        let state = BootState {
            camera: Camera {
                visible_plane_radius: 1,
                ..Camera::default()
            },
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [
                        Cell {
                            position: CellPoint { x: 0, y: 0, z: -2 },
                            graphic: CellGraphic::Glyph('A'),
                            ..Cell::default()
                        },
                        Cell {
                            position: CellPoint { x: 0, y: 0, z: 0 },
                            graphic: CellGraphic::Glyph('B'),
                            ..Cell::default()
                        },
                        Cell {
                            position: CellPoint { x: 0, y: 0, z: 2 },
                            graphic: CellGraphic::Glyph('C'),
                            ..Cell::default()
                        },
                    ],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig::default(),
            uses_fallback_breath: false,
        };

        let staged = stage_projected_boot_cells(
            &state,
            cell_clip_size_for_surface(SurfaceSize::from(&state.config.window)),
        );
        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].projected.plane, 0);
    }

    #[test]
    fn stage_projected_boot_cells_filters_cells_outside_surface_xy_bounds() {
        let mut config = BootConfig::default();
        config.window.width = 1280;
        config.window.height = 720;
        let state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [
                        Cell {
                            position: CellPoint { x: 0, y: 0, z: 0 },
                            graphic: CellGraphic::Glyph('A'),
                            ..Cell::default()
                        },
                        Cell {
                            position: CellPoint { x: 30, y: 0, z: 0 },
                            graphic: CellGraphic::Glyph('B'),
                            ..Cell::default()
                        },
                    ],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config,
            uses_fallback_breath: false,
        };

        let staged = stage_projected_boot_cells(
            &state,
            cell_clip_size_for_surface(SurfaceSize::from(&state.config.window)),
        );
        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].world, WorldPoint::origin());
    }

    #[test]
    fn build_window_surface_scene_for_surface_uses_live_surface_size_for_xy_culling() {
        let mut config = BootConfig::default();
        config.window.width = 1280;
        config.window.height = 720;
        let state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [
                        Cell {
                            position: CellPoint { x: 0, y: 0, z: 0 },
                            graphic: CellGraphic::Glyph('A'),
                            ..Cell::default()
                        },
                        Cell {
                            position: CellPoint { x: 10, y: 0, z: 0 },
                            graphic: CellGraphic::Glyph('B'),
                            ..Cell::default()
                        },
                    ],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config,
            uses_fallback_breath: false,
        };

        let wide_staged = stage_projected_boot_cells(
            &state,
            cell_clip_size_for_surface(SurfaceSize {
                width: 1280,
                height: 720,
            }),
        );
        let narrow_staged = stage_projected_boot_cells(
            &state,
            cell_clip_size_for_surface(SurfaceSize {
                width: 640,
                height: 720,
            }),
        );

        assert!(wide_staged.len() > narrow_staged.len());
    }

    #[test]
    fn effective_depth_edge_fade_turns_off_for_tiny_plane_counts() {
        assert_eq!(effective_depth_edge_fade_span_cells(5.0, 1), 0.0);
        assert_eq!(effective_depth_edge_fade_span_cells(5.0, 2), 0.0);
        assert!((effective_depth_edge_fade_span_cells(5.0, 3) - (5.0 / 3.0)).abs() < 0.0001);
        assert!((effective_depth_edge_fade_span_cells(5.0, 4) - (10.0 / 3.0)).abs() < 0.0001);
        assert_eq!(effective_depth_edge_fade_span_cells(5.0, 5), 5.0);
    }

    #[test]
    fn group_projected_boot_cells_by_plane_orders_far_planes_before_near_planes() {
        let grouped = group_projected_boot_cells_by_plane(vec![
            ProjectedBootCell {
                world: WorldPoint { x: 3, y: 0, z: 0 },
                projected: CameraProjectedPoint {
                    u: 3.5,
                    v: -0.5,
                    plane: 1,
                },
                cell: Cell {
                    graphic: CellGraphic::Glyph('C'),
                    ..Cell::default()
                },
            },
            ProjectedBootCell {
                world: WorldPoint { x: 1, y: 0, z: 0 },
                projected: CameraProjectedPoint {
                    u: -1.0,
                    v: 0.0,
                    plane: -2,
                },
                cell: Cell {
                    graphic: CellGraphic::Glyph('A'),
                    ..Cell::default()
                },
            },
            ProjectedBootCell {
                world: WorldPoint { x: 4, y: 1, z: 0 },
                projected: CameraProjectedPoint {
                    u: 4.5,
                    v: 0.5,
                    plane: 1,
                },
                cell: Cell {
                    graphic: CellGraphic::Glyph('D'),
                    ..Cell::default()
                },
            },
        ]);

        assert_eq!(
            grouped.iter().map(|plane| plane.plane).collect::<Vec<_>>(),
            vec![1, -2]
        );
        assert_eq!(grouped[0].cells.len(), 2);
        assert_eq!(grouped[1].cells.len(), 1);
    }

    #[test]
    fn build_window_surface_scene_skips_cells_without_graphics() {
        let state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        graphic: CellGraphic::None,
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig::default(),
            uses_fallback_breath: false,
        };

        let scene = build_window_surface_scene(&state).unwrap();
        assert!(scene.quads.is_empty());
    }

    #[test]
    fn glyph_font_weight_files_map_from_cell_weight() {
        let asset_root = staged_asset_root();

        assert_eq!(
            glyph_font_path(&asset_root, CellWeight::Zero),
            asset_root.join("glyph-fonts/thaum-mono/ThaumMono-W80.ttf")
        );
        assert_eq!(
            glyph_font_path(&asset_root, CellWeight::One),
            asset_root.join("glyph-fonts/thaum-mono/ThaumMono-W160.ttf")
        );
        assert_eq!(
            glyph_font_path(&asset_root, CellWeight::Two),
            asset_root.join("glyph-fonts/thaum-mono/ThaumMono-W320.ttf")
        );
        assert_eq!(
            glyph_font_path(&asset_root, CellWeight::Three),
            asset_root.join("glyph-fonts/thaum-mono/ThaumMono-W640.ttf")
        );
    }

    #[test]
    fn staged_thaum_mono_font_rasterizes_into_a_non_empty_12x16_tile() {
        let fonts = GlyphFontSet::load_from_asset_root(&staged_asset_root()).unwrap();
        let raster = fonts.rasterize_glyph_tile('A', CellWeight::Two);

        assert_eq!(raster.width, GLYPH_TILE_WIDTH);
        assert_eq!(raster.height, GLYPH_TILE_HEIGHT);
        assert!(raster.coverage_count() > 0);
    }

    #[test]
    fn staged_thaum_mono_tester_glyphs_rasterize_for_all_weights() {
        let fonts = GlyphFontSet::load_from_asset_root(&staged_asset_root()).unwrap();

        for glyph in ['█', '▓', '▒', '░'] {
            for weight in [
                CellWeight::Zero,
                CellWeight::One,
                CellWeight::Two,
                CellWeight::Three,
            ] {
                let raster = fonts.rasterize_glyph_tile(glyph, weight);
                assert_eq!(raster.width, GLYPH_TILE_WIDTH);
                assert_eq!(raster.height, GLYPH_TILE_HEIGHT);
                assert!(
                    raster.coverage_count() > 0,
                    "expected non-empty raster for glyph {glyph} at weight {:?}",
                    weight
                );
            }
        }
    }

    #[test]
    fn staged_thaum_mono_weight_variants_change_glyph_coverage() {
        let fonts = GlyphFontSet::load_from_asset_root(&staged_asset_root()).unwrap();
        let coverages = [
            fonts
                .rasterize_glyph_tile('A', CellWeight::Zero)
                .coverage_count(),
            fonts
                .rasterize_glyph_tile('A', CellWeight::One)
                .coverage_count(),
            fonts
                .rasterize_glyph_tile('A', CellWeight::Two)
                .coverage_count(),
            fonts
                .rasterize_glyph_tile('A', CellWeight::Three)
                .coverage_count(),
        ];

        assert!(
            coverages.windows(2).any(|pair| pair[0] != pair[1]),
            "expected at least one Thaum Mono weight change to alter glyph coverage; got {coverages:?}"
        );
    }

    #[test]
    fn staged_thaum_mono_block_glyph_uses_most_of_the_12x16_tile() {
        let fonts = GlyphFontSet::load_from_asset_root(&staged_asset_root()).unwrap();
        let raster = fonts.rasterize_glyph_tile('█', CellWeight::Two);
        let (min_x, min_y, max_x, max_y) = raster.coverage_bounds().unwrap();

        assert!(
            min_x <= 1,
            "expected block glyph to reach near the left edge; got {min_x}"
        );
        assert!(
            min_y <= 1,
            "expected block glyph to reach near the top edge; got {min_y}"
        );
        assert!(
            max_x >= 10,
            "expected block glyph to reach near the right edge; got {max_x}"
        );
        assert!(
            max_y >= 14,
            "expected block glyph to reach near the bottom edge; got {max_y}"
        );
    }

    #[test]
    fn build_window_surface_scene_projects_real_glyph_pixels_relative_to_camera_focus() {
        let state = BootState {
            camera: Camera {
                position: WorldPoint::origin(),
                focus_target: WorldPoint { x: 1, y: 2, z: 0 },
                swing: CameraSwing::PosZ,
                ..Camera::default()
            },
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint { x: 2, y: 4, z: 0 },
                    [Cell {
                        position: CellPoint { x: 1, y: -1, z: 0 },
                        graphic: CellGraphic::Glyph('A'),
                        weight: CellWeight::Three,
                        color: CellColor::Material(CellMaterialId::GrayScale),
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };

        let scene = build_window_surface_scene(&state).unwrap();
        assert!(!scene.quads.is_empty());
        let cell_clip_size = cell_clip_size_for_surface(SurfaceSize::from(&state.config.window));
        assert!(scene
            .quads
            .iter()
            .all(|quad| { quad.size == [cell_clip_size[0] / 12.0, cell_clip_size[1] / 16.0] }));
        assert!(scene
            .quads
            .iter()
            .all(|quad| quad.center[0] >= 0.12656249 && quad.center[0] <= 0.21093749));
        assert!(scene
            .quads
            .iter()
            .all(|quad| quad.center[1] >= 0.1 && quad.center[1] <= 0.3));
    }

    #[test]
    fn build_window_surface_scene_uses_flat_color_directly_for_real_glyphs() {
        let state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('@'),
                        color: CellColor::Flat([0.2, 0.4, 0.8, 1.0]),
                        weight: CellWeight::One,
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };

        let scene = build_window_surface_scene(&state).unwrap();
        assert!(!scene.quads.is_empty());
        assert!(scene
            .quads
            .iter()
            .all(|quad| quad.color[0] == 0.2 && quad.color[1] == 0.4 && quad.color[2] == 0.8));
        assert!(scene.quads.iter().all(|quad| quad.color[3] == 1.0));
    }

    #[test]
    fn build_window_surface_scene_uses_medium_light_band_for_material_glyphs() {
        let state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('#'),
                        color: CellColor::Material(CellMaterialId::GrayScale),
                        weight: CellWeight::Zero,
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };

        let scene = build_window_surface_scene(&state).unwrap();
        assert!(!scene.quads.is_empty());
        assert!(scene.quads.iter().all(|quad| {
            quad.color[0] == 0xa8 as f32 / 255.0
                && quad.color[1] == 0xa8 as f32 / 255.0
                && quad.color[2] == 0xa8 as f32 / 255.0
                && quad.color[3] == 1.0
        }));
    }

    #[test]
    fn real_glyph_scene_quads_keep_the_12x16_cell_aspect_ratio() {
        let state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('█'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::Two,
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };

        let scene = build_window_surface_scene(&state).unwrap();
        let first_quad = scene.quads.first().unwrap();
        let cell_clip_size = cell_clip_size_for_surface(SurfaceSize::from(&state.config.window));
        assert_eq!(
            first_quad.size,
            [cell_clip_size[0] / 12.0, cell_clip_size[1] / 16.0]
        );
        let screen_pixel_width = first_quad.size[0] * state.config.window.width as f32;
        let screen_pixel_height = first_quad.size[1] * state.config.window.height as f32;
        assert!((screen_pixel_width - screen_pixel_height).abs() < 0.0001);
    }

    #[test]
    fn build_window_surface_scene_projects_a_real_cell_through_group_facing_and_composition() {
        let baseline = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::Two,
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };
        let faced = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint { x: 0, y: 0, z: 1 },
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::Two,
                        ..Cell::default()
                    }],
                )
                .with_facing(CellGroupFacing::PosX)],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };

        let baseline_scene = build_window_surface_scene(&baseline).unwrap();
        let faced_scene = build_window_surface_scene(&faced).unwrap();

        assert_eq!(baseline_scene.quads.len(), faced_scene.quads.len());
        assert!(baseline_scene
            .quads
            .iter()
            .zip(faced_scene.quads.iter())
            .all(|(baseline, faced)| faced.center[0] > baseline.center[0]));
    }

    #[test]
    fn projected_cell_center_for_surface_uses_projection_u_and_v() {
        let camera = Camera {
            focus_target: WorldPoint { x: 1, y: -2, z: 3 },
            swing: CameraSwing::PosX,
            ..Camera::default()
        };
        let world = WorldPoint { x: 4, y: 5, z: -1 };
        let projected = project_world_to_view_plane(camera, world);
        let center = projected_cell_center_for_surface(projected, [0.1, 0.2]);

        assert_eq!(projected.plane, 3);
        assert!(center[0] > 0.05 && center[0] < 0.35);
        assert!(center[1] > 0.3 && center[1] < 1.6);
    }

    #[test]
    fn build_window_surface_scene_projects_world_z_into_visible_screen_offset() {
        let state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint { x: 0, y: 0, z: 2 },
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('█'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };

        let scene = build_window_surface_scene(&state).unwrap();
        assert!(!scene.quads.is_empty());
        assert!(scene
            .quads
            .iter()
            .any(|quad| quad.center[0] > 0.0 && quad.center[1] < 0.0));
    }

    #[test]
    fn build_window_surface_scene_resolves_exact_world_xyz_overlap_before_projection() {
        let state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![
                    CellGroup::from_cells(
                        WorldPoint::origin(),
                        [Cell {
                            position: CellPoint::origin(),
                            graphic: CellGraphic::Glyph('█'),
                            color: CellColor::Flat([1.0, 0.0, 0.0, 1.0]),
                            ..Cell::default()
                        }],
                    ),
                    CellGroup::from_cells(
                        WorldPoint { x: -1, y: 0, z: 0 },
                        [Cell {
                            position: CellPoint { x: 1, y: 0, z: 0 },
                            graphic: CellGraphic::Glyph('█'),
                            color: CellColor::Flat([0.0, 1.0, 0.0, 1.0]),
                            ..Cell::default()
                        }],
                    ),
                ],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::default(),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };

        let scene = build_window_surface_scene(&state).unwrap();
        let fonts = GlyphFontSet::load_from_asset_root(&staged_asset_root()).unwrap();
        let expected_quad_count = fonts
            .rasterize_glyph_tile('█', CellWeight::Zero)
            .alpha
            .into_iter()
            .filter(|alpha| *alpha >= GLYPH_BINARY_ALPHA_THRESHOLD)
            .count();

        assert_eq!(scene.quads.len(), expected_quad_count);
        assert!(scene
            .quads
            .iter()
            .all(|quad| quad.color == [0.0, 1.0, 0.0, 1.0]));
    }

    #[test]
    fn build_window_surface_scene_applies_weight_sin_to_real_glyph_tiles() {
        let state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::One,
                        shader_stack: vec![CELL_SHADER_WEIGHT_SIN],
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::with_breath(2),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };

        let scene = build_window_surface_scene(&state).unwrap();
        let fonts = GlyphFontSet::load_from_asset_root(&staged_asset_root()).unwrap();
        let unshaded = fonts
            .rasterize_glyph_tile('A', CellWeight::One)
            .alpha
            .into_iter()
            .filter(|alpha| *alpha >= GLYPH_BINARY_ALPHA_THRESHOLD)
            .count();
        let shaded_expected = fonts
            .rasterize_glyph_tile('A', CellWeight::Two)
            .alpha
            .into_iter()
            .filter(|alpha| *alpha >= GLYPH_BINARY_ALPHA_THRESHOLD)
            .count();
        let shaded = scene.quads.len();

        assert_eq!(shaded, shaded_expected);
        assert_ne!(shaded, unshaded);
    }

    #[test]
    fn build_window_surface_scene_applies_texture_debug_post_effect_to_real_glyph_tiles() {
        let baseline_state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::Two,
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::with_breath(2),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };
        let textured_state = BootState {
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::Two,
                        shader_stack: vec![CELL_SHADER_TEXTURE_SHIMMER],
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            config: BootConfig {
                debug_texture_post_effect: true,
                ..baseline_state.config.clone()
            },
            ..baseline_state.clone()
        };

        let baseline_scene = build_window_surface_scene(&baseline_state).unwrap();
        let textured_scene = build_window_surface_scene(&textured_state).unwrap();

        assert!(textured_scene.quads.len() >= baseline_scene.quads.len());
        assert!(textured_scene
            .quads
            .iter()
            .filter(|quad| quad.color[3] > 0.0)
            .all(|quad| quad.color[0] == 0.0 && quad.color[1] == 0.0 && quad.color[2] == 1.0));
        assert!(textured_scene
            .quads
            .iter()
            .all(|quad| quad.post_effect_bus.texture_code != 0));
    }

    #[test]
    fn build_window_surface_scene_emits_texture_footprint_quads_for_textured_glyph_tiles() {
        let baseline_state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::Two,
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::with_breath(2),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };
        let textured_state = BootState {
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::Two,
                        shader_stack: vec![CELL_SHADER_TEXTURE_SHIMMER],
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            ..baseline_state.clone()
        };

        let baseline_scene = build_window_surface_scene(&baseline_state).unwrap();
        let textured_scene = build_window_surface_scene(&textured_state).unwrap();

        assert!(textured_scene.quads.len() > baseline_scene.quads.len());
        assert!(textured_scene
            .quads
            .iter()
            .any(|quad| quad.color[3] == 0.0 && quad.post_effect_bus.texture_code != 0));
    }

    #[test]
    fn build_window_surface_scene_applies_warble_debug_post_effect_to_real_glyph_tiles() {
        let baseline_state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::Two,
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::with_breath(2),
            config: BootConfig {
                asset_root: staged_asset_root(),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };
        let warbled_state = BootState {
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::Two,
                        shader_stack: vec![CELL_SHADER_WARBLE_DIAGONAL],
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            config: BootConfig {
                debug_warble_post_effect: true,
                ..baseline_state.config.clone()
            },
            ..baseline_state.clone()
        };

        let baseline_scene = build_window_surface_scene(&baseline_state).unwrap();
        let warbled_scene = build_window_surface_scene(&warbled_state).unwrap();

        assert!(warbled_scene.quads.len() > baseline_scene.quads.len());
        assert!(warbled_scene
            .quads
            .iter()
            .all(|quad| quad.color[0] == 1.0 && quad.color[1] == 0.0 && quad.color[2] == 0.0));
        assert!(warbled_scene
            .quads
            .iter()
            .all(|quad| quad.post_effect_bus.warble_code != 0));
    }

    #[test]
    fn build_window_surface_scene_applies_index_color_clamp_after_other_post_effects() {
        let palette = vec![[0x20, 0x40, 0x80, 0xff]];
        let state = BootState {
            camera: Camera::default(),
            composition: Composition {
                groups: vec![CellGroup::from_cells(
                    WorldPoint::origin(),
                    [Cell {
                        position: CellPoint::origin(),
                        graphic: CellGraphic::Glyph('A'),
                        color: CellColor::Flat([1.0, 1.0, 1.0, 1.0]),
                        weight: CellWeight::Two,
                        shader_stack: vec![CELL_SHADER_TEXTURE_SHIMMER],
                        ..Cell::default()
                    }],
                )],
                pass_order: Vec::new(),
            }
            .with_natural_pass_order(),
            data_lanes: DataLanes::with_breath(2),
            config: BootConfig {
                asset_root: staged_asset_root(),
                debug_texture_post_effect: true,
                index_color_clamp: IndexColorClampEffect::new(palette.clone()),
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };

        let scene = build_window_surface_scene(&state).unwrap();

        assert!(!scene.quads.is_empty());
        assert!(scene.indexed_color_enabled);
        assert_eq!(scene.indexed_color_palette, palette);
        assert!(scene
            .quads
            .iter()
            .any(|quad| quad.post_effect_bus.texture_code != 0));
    }

    #[test]
    fn build_window_surface_scene_carries_depth_of_field_toggle() {
        let disabled_state = BootState {
            camera: Camera::default(),
            composition: Composition::default(),
            data_lanes: DataLanes::with_breath(0),
            config: BootConfig {
                asset_root: staged_asset_root(),
                depth_of_field_post_effect: false,
                depth_of_field_minimum_falloff_cells: 5.0,
                ..BootConfig::default()
            },
            uses_fallback_breath: false,
        };
        let enabled_state = BootState {
            config: BootConfig {
                depth_of_field_post_effect: true,
                depth_of_field_minimum_falloff_cells: 7.0,
                ..disabled_state.config.clone()
            },
            ..disabled_state.clone()
        };

        assert!(
            !build_window_surface_scene(&disabled_state)
                .unwrap()
                .depth_of_field_enabled
        );
        assert_eq!(
            build_window_surface_scene(&disabled_state)
                .unwrap()
                .depth_of_field_minimum_falloff_cells,
            5.0
        );
        assert!(
            build_window_surface_scene(&enabled_state)
                .unwrap()
                .depth_of_field_enabled
        );
        assert_eq!(
            build_window_surface_scene(&enabled_state)
                .unwrap()
                .depth_of_field_minimum_falloff_cells,
            7.0
        );
    }

    #[test]
    fn cell_clip_size_compensates_for_wide_surface_aspect_ratio() {
        let clip_size = cell_clip_size_for_surface(SurfaceSize {
            width: 1280,
            height: 720,
        });

        assert_eq!(clip_size[1], CELL_HEIGHT_CLIP_SPACE);
        assert!((clip_size[0] - 0.084375).abs() < 0.0001);
    }

    #[test]
    fn cell_clip_size_for_state_scales_with_camera_zoom() {
        let surface_size = SurfaceSize {
            width: 1280,
            height: 720,
        };
        let mut state = BootState {
            camera: Camera::default(),
            composition: Composition::default(),
            data_lanes: DataLanes::default(),
            config: BootConfig::default(),
            uses_fallback_breath: false,
        };

        let base = cell_clip_size_for_state(&state, surface_size);
        state.camera.zoom *= 2.0;
        let zoomed = cell_clip_size_for_state(&state, surface_size);

        assert_eq!(zoomed[0], base[0] * 2.0);
        assert_eq!(zoomed[1], base[1] * 2.0);
    }
}
