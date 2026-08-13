pub const TEXTURE_POST_EFFECT_PRESET_PERLIN_DISPLACEMENT: u8 = 1;
pub const TEXTURE_POST_EFFECT_SEED_BREATH_INTERVAL: i32 = 1;
pub const TEXTURE_POST_PROCESS_SHADER_SNIPPET: &str = include_str!("post_process_texture.wgsl");

pub fn texture_post_effect_seed_step(breath: i32) -> i32 {
    if TEXTURE_POST_EFFECT_SEED_BREATH_INTERVAL <= 1 {
        return breath;
    }

    breath.div_euclid(TEXTURE_POST_EFFECT_SEED_BREATH_INTERVAL)
}

pub fn texture_post_effect_texel_uv_size(quad_size: [f32; 2]) -> f32 {
    0.5 * quad_size[0].abs().min(quad_size[1].abs())
}

pub const TEXTURE_POST_PROCESS_SHADER: &str = TEXTURE_POST_PROCESS_SHADER_SNIPPET;
