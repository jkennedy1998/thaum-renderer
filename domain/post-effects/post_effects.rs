#[path = "debug_bus.rs"]
pub mod debug_bus;
#[path = "index-color-clamp/index_color_clamp.rs"]
pub mod index_color_clamp;
#[path = "texture/texture.rs"]
pub mod texture;
#[path = "warble/warble.rs"]
pub mod warble;

pub use debug_bus::{
    apply_debug_depth_post_effect_to_rgba, apply_debug_texture_post_effect_to_rgba,
    apply_debug_warble_post_effect_to_rgba, encode_relative_depth_to_post_effect_bus,
    POST_EFFECT_BUS_FOCUS_DEPTH_CODE,
};
pub use index_color_clamp::{
    clamp_rgba_collection_to_index_palette, clamp_rgba_to_index_palette, IndexColorClampEffect,
};
pub const POST_EFFECTS_SHADER: &str = concat!(
    include_str!("post_process_shared_head.wgsl"),
    include_str!("warble/post_process_warble.wgsl"),
    include_str!("texture/post_process_texture.wgsl"),
    include_str!("post_process_shared_tail.wgsl"),
);
pub use texture::{
    texture_post_effect_seed_step, texture_post_effect_texel_uv_size,
    TEXTURE_POST_EFFECT_PRESET_PERLIN_DISPLACEMENT, TEXTURE_POST_EFFECT_SEED_BREATH_INTERVAL,
    TEXTURE_POST_PROCESS_SHADER, TEXTURE_POST_PROCESS_SHADER_SNIPPET,
};
pub use warble::{
    warble_post_effect_breath_phase, warble_post_effect_is_active,
    WARBLE_POST_EFFECT_BREATH_SCROLL_RATE, WARBLE_POST_EFFECT_PRESET_NONE,
    WARBLE_POST_EFFECT_PRESET_PERLIN_SWELL, WARBLE_POST_PROCESS_SHADER_SNIPPET,
};
