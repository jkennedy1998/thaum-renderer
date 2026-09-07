/// Glyph atlas section text for the monothaum atlas v3, compiled in so
/// dependents (e.g. the painter's graphic picker) can parse glyph groupings
/// without runtime asset IO and without baking machine-specific paths.
pub const MONOTHAUM_ATLAS_V3_SECTIONS_TEXT: &str =
    include_str!("../orchestration/renderer-assets/cell-sprites/monothaum-atlas-v3/sections.txt");

#[path = "atlas-intake/atlas_intake.rs"]
pub mod atlas_intake;
#[path = "camera/camera.rs"]
pub mod camera;
#[path = "modules/individuals/camera-perspective/camera_perspective_module.rs"]
pub mod camera_perspective_module;
#[path = "cell/cell.rs"]
pub mod cell;
#[path = "cell-color/cell_color.rs"]
pub mod cell_color;
#[path = "cell-graphic/cell_graphic.rs"]
pub mod cell_graphic;
#[path = "cell-group/cell_group.rs"]
pub mod cell_group;
#[path = "cell-materials/cell_materials.rs"]
pub mod cell_materials;
#[path = "cell-shader/cell_shader.rs"]
pub mod cell_shader;
#[path = "cell-texture/cell_texture.rs"]
pub mod cell_texture;
#[path = "cell-warble/cell_warble.rs"]
pub mod cell_warble;
#[path = "cell-weight/cell_weight.rs"]
pub mod cell_weight;
#[path = "modules/individuals/color-block/color_block_module.rs"]
pub mod color_block_module;
#[path = "modules/individuals/color-picker/color_picker_module.rs"]
pub mod color_picker_module;
#[path = "command-bar/command_bar.rs"]
pub mod command_bar;
#[path = "composition/composition.rs"]
pub mod composition;
#[path = "composition/composition_hash.rs"]
pub mod composition_hash;
#[path = "controls/controls.rs"]
pub mod controls;
#[path = "modules/individuals/controls-panel/controls_panel_module.rs"]
pub mod controls_panel_module;
#[path = "coordinate-space/coordinate_space.rs"]
pub mod coordinate_space;
#[path = "data-lanes/data_lanes.rs"]
pub mod data_lanes;
#[path = "debug-log/debug_log.rs"]
pub mod debug_log;
#[path = "modules/module.rs"]
pub mod module;
#[path = "modules/shared/module-gizmos/module_gizmos.rs"]
pub mod module_gizmos;
#[path = "modules/shared/panel-chrome/panel_chrome.rs"]
pub mod panel_chrome;
#[path = "post-effects/post_effects.rs"]
pub mod post_effects;
#[path = "modules/shared/property-rows/property_rows.rs"]
pub mod property_rows;
#[path = "modules/shared/scroll-state/scroll_state.rs"]
pub mod scroll_state;
#[path = "cell-graphic/shape-fade/shape_fade.rs"]
pub mod shape_fade;
#[path = "cell-graphic/sprite/sprite-color-space/sprite_color_space.rs"]
pub mod sprite_color_space;
#[path = "modules/shared/tooltip/tooltip.rs"]
pub mod tooltip;
#[path = "modules/individuals/ui-customization/ui_customization_module.rs"]
pub mod ui_customization_module;
#[path = "modules/shared/ui-palette/ui_palette.rs"]
pub mod ui_palette;
#[path = "persistence/ui-session-state/ui_session_state.rs"]
pub mod ui_session_state;

pub use atlas_intake::{
    load_sprite_atlas_image, load_sprite_atlas_spec, AtlasForm, AtlasSpec, SpriteAtlasImage,
};
pub use camera::{
    active_depth_axis_for_swing, active_depth_direction_for_swing,
    build_visible_plane_stack_around_focus, camera_view_orientation_for_camera,
    camera_view_orientation_for_swing, derive_visible_plane_stack_from_world_points,
    focus_plane_for_camera, project_flat_2d_world_to_view_plane,
    project_rotating_3d_world_to_view_plane, project_world_relative_to_view,
    project_world_to_camera_units, project_world_to_view_plane,
    project_world_to_view_plane_for_intake, projected_plane_is_visible,
    projected_plane_scale_factor, remap_camera_units_to_active_plane_world,
    remap_camera_units_to_world_on_plane, remap_surface_units_to_active_plane_world,
    remap_surface_units_to_flat_2d_local, unproject_flat_2d_view_plane_to_local,
    unproject_view_plane_to_world, unproject_view_relative_to_world,
    visible_plane_stack_for_camera, Camera, CameraProjectedPoint, CameraProjectionMode, CameraRoll,
    CameraSwing, CameraViewOrientation, ParallaxProfile, PerspectiveProfile, ViewRelativePoint,
    VisiblePlaneStack,
};
pub use camera_perspective_module::{
    CameraDepthLink, CameraLayersLink, CameraPerspectiveModule, MAX_VISIBLE_PLANE_RADIUS,
};
pub use cell::Cell;
pub use cell_color::{CellColor, CellColorSlot};
pub use cell_graphic::{
    glyph_font_path, CellGraphic, GlyphFontSet, GlyphTileRaster, SpriteAtlasSet, SpriteGraphic,
    SpriteTileRaster, GLYPH_TILE_HEIGHT, GLYPH_TILE_WIDTH,
};
pub use cell_group::{CellBounds, CellClip, CellGroup, CellGroupFacing, CellGroupIntakeBehavior};
pub use cell_materials::{CellMaterialId, ColorBand};
pub use cell_shader::{
    resolve_shaded_graphic, resolve_shaded_texture, resolve_shaded_warble, resolve_shaded_weight,
    vivid_flash_is_lit, CELL_SHADER_PASS, CELL_SHADER_TEXTURE_SHIMMER, CELL_SHADER_VIVID_FLASH,
    CELL_SHADER_VIVID_FLASH_ALT, CELL_SHADER_WARBLE_DIAGONAL, CELL_SHADER_WARBLE_DISTORT_1,
    CELL_SHADER_WARBLE_DISTORT_5, CELL_SHADER_WARBLE_FUDGE_1, CELL_SHADER_WARBLE_FUDGE_5,
    CELL_SHADER_WEIGHT_SIN, VIVID_FLASH_BREATH_PERIOD,
};
pub use cell_texture::CellTexture;
pub use cell_warble::CellWarble;
pub use cell_weight::CellWeight;
pub use color_block_module::ColorBlockModule;
pub use color_picker_module::ColorPickerModule;
pub use command_bar::{
    CommandBar, CommandBarButton, CommandBarClickOutcome, CommandBarLayout,
    PersistedCommandBarState,
};
pub use composition::{compose_cells, ComposedCell, Composition};
pub use composition_hash::composition_content_hash;
pub use controls::{
    profile::{conflicting_actions, effective_bindings, format_raw_input, ControlsProfile},
    typing_mode::{TypingMode, TypingRoute},
    ActionBindingMap, ActionName, PressureSample, RawInput,
};
pub use controls_panel_module::{ControlActionRow, ControlsPanelModule};
pub use coordinate_space::{AxisSign, CellPoint, GlobalDirection, WorldAxis, WorldPoint};
pub use data_lanes::DataLanes;
pub use module::{
    BlankPanelModule, Module, ModulePointerButton, ModulePointerEvent, ModuleRect, ModuleRegistry,
};
pub use module_gizmos::{GizmoBar, GizmoClickOutcome, GizmoKind, GizmoState, ResizeEdge};
pub use panel_chrome::{PanelBorderEdge, PanelBorderStyle, PanelChrome};
pub use post_effects::{
    apply_debug_depth_post_effect_to_rgba, apply_debug_texture_post_effect_to_rgba,
    apply_debug_warble_post_effect_to_rgba, clamp_rgba_collection_to_index_palette,
    clamp_rgba_to_index_palette, encode_relative_depth_to_post_effect_bus,
    texture_post_effect_seed_step, texture_post_effect_texel_uv_size,
    warble_post_effect_breath_phase, warble_post_effect_is_active, IndexColorClampEffect,
    POST_EFFECTS_SHADER, POST_EFFECT_BUS_FOCUS_DEPTH_CODE,
    TEXTURE_POST_EFFECT_PRESET_PERLIN_DISPLACEMENT, TEXTURE_POST_EFFECT_SEED_BREATH_INTERVAL,
    TEXTURE_POST_PROCESS_SHADER, TEXTURE_POST_PROCESS_SHADER_SNIPPET,
    WARBLE_POST_EFFECT_BREATH_SCROLL_RATE, WARBLE_POST_EFFECT_PRESET_NONE,
    WARBLE_POST_EFFECT_PRESET_PERLIN_SWELL, WARBLE_POST_PROCESS_SHADER_SNIPPET,
};
pub use property_rows::{
    NumberFieldEdit, PropertyHit, PropertyMatrixColumn, PropertyMatrixSide, PropertyRow,
    PropertyRows,
};
pub use scroll_state::ScrollState;
pub use sprite_color_space::{
    canonical_sprite_palette, decode_sprite_pixel, decode_sprite_rgba, DecodedSpritePixel,
    SpriteColorChannel,
};
pub use tooltip::{tooltip_card_group, Hotspot, TooltipState, DWELL, TEXT_WRAP_COLUMNS};
pub use ui_customization_module::UiCustomizationModule;
pub use ui_palette::{UiColorRole, UiPalette};
pub use ui_session_state::{
    PersistedCameraUiState, PersistedModuleRect, PersistedModuleUiState,
    PersistedRendererUiSessionState, PersistedUiPaletteColor, PersistedUiPaletteState,
};
