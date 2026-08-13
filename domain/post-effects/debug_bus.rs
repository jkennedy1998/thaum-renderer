pub const POST_EFFECT_BUS_FOCUS_DEPTH_CODE: u8 = 128;

pub fn encode_relative_depth_to_post_effect_bus(relative_depth: i32) -> u8 {
    (POST_EFFECT_BUS_FOCUS_DEPTH_CODE as i32 + relative_depth).clamp(0, 255) as u8
}

pub fn apply_debug_texture_post_effect_to_rgba(mut rgba: [f32; 4], texture_code: u8) -> [f32; 4] {
    if texture_code != 0 {
        rgba = [0.0, 0.0, 1.0, rgba[3].max(1.0)];
    }
    rgba
}

pub fn apply_debug_warble_post_effect_to_rgba(mut rgba: [f32; 4], warble_code: u8) -> [f32; 4] {
    if warble_code != 0 {
        rgba = [1.0, 0.0, 0.0, rgba[3].max(1.0)];
    }
    rgba
}

pub fn apply_debug_depth_post_effect_to_rgba(_rgba: [f32; 4], depth_code: u8) -> [f32; 4] {
    let value = depth_code as f32 / 255.0;
    [value, value, value, 1.0]
}
