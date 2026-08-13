pub const WARBLE_POST_EFFECT_PRESET_NONE: u8 = 0;
pub const WARBLE_POST_EFFECT_PRESET_PERLIN_SWELL: u8 = 1;
pub const WARBLE_POST_EFFECT_BREATH_SCROLL_RATE: f32 = 0.08;
pub const WARBLE_POST_PROCESS_SHADER_SNIPPET: &str = include_str!("post_process_warble.wgsl");

pub fn warble_post_effect_is_active(warble_code: u8) -> bool {
    warble_code != WARBLE_POST_EFFECT_PRESET_NONE
}

pub fn warble_post_effect_breath_phase(breath: i32) -> f32 {
    breath as f32 * WARBLE_POST_EFFECT_BREATH_SCROLL_RATE
}
