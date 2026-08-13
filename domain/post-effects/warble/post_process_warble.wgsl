const WARBLE_SCALE_A: vec2<f32> = vec2<f32>(0.25, 0.25);
const WARBLE_SCALE_B: vec2<f32> = vec2<f32>(0.25, 0.25);
const WARBLE_AMPLITUDE_TEXELS: f32 = 5.8;

fn sample_warble_uv(uv: vec2<f32>) -> vec2<f32> {
    return textureSample(warble_uv_tex, nearest_sampler, uv).xy;
}

fn warble_displacement(global_uv: vec2<f32>, breath_phase: f32, texel_uv_size: f32) -> vec2<f32> {
    let a = layered_noise(global_uv * WARBLE_SCALE_A + vec2<f32>(breath_phase, breath_phase * 0.6));
    let b = layered_noise(global_uv * WARBLE_SCALE_B + vec2<f32>(-breath_phase * 0.4, breath_phase * 0.9));
    let vector = vec2<f32>(a, b) * 2.0 - 1.0;
    return vector * (texel_uv_size * WARBLE_AMPLITUDE_TEXELS);
}

fn warble_sample_matches(origin_bus: vec4<f32>, sample_uv: vec2<f32>) -> bool {
    let sample_bus = textureSample(bus_tex, nearest_sampler, sample_uv);
    let sample_color = textureSample(color_tex, linear_sampler, sample_uv);

    return decode_byte(sample_bus.g) == decode_byte(origin_bus.g)
        && decode_byte(sample_bus.b) == decode_byte(origin_bus.b)
        && sample_color.a > 0.05;
}

fn warble_displaced_sample_is_usable(base_color: vec4<f32>, displaced_color: vec4<f32>) -> bool {
    let base_alpha = clamp(base_color.a, 0.0, 1.0);
    let displaced_alpha = clamp(displaced_color.a, 0.0, 1.0);
    let minimum_alpha = max(0.03, base_alpha * 0.15);
    return displaced_alpha >= minimum_alpha;
}

fn resolve_warble_color(base_color: vec4<f32>, origin_bus: vec4<f32>, origin_meta: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let warble_code = decode_byte(origin_bus.g);
    if uniforms.flags.x < 0.5 || warble_code != 1u {
        return base_color;
    }

    let texel_uv_size = origin_meta.b / POST_EFFECT_TEXEL_UV_PACK_SCALE;
    if texel_uv_size <= 0.0 {
        return base_color;
    }

    let warble_uv = sample_warble_uv(uv);
    let displacement = warble_displacement(warble_uv, uniforms.seed_phase_uv.y, texel_uv_size);

    let outward_uv = clamp(uv + displacement, vec2<f32>(0.0), vec2<f32>(1.0));
    if warble_sample_matches(origin_bus, outward_uv) {
        let displaced_color = textureSample(color_tex, linear_sampler, outward_uv);
        if warble_displaced_sample_is_usable(base_color, displaced_color) {
            return composite_displaced_over_base(base_color, displaced_color);
        }
    }

    let inward_uv = clamp(uv - displacement, vec2<f32>(0.0), vec2<f32>(1.0));
    if warble_sample_matches(origin_bus, inward_uv) {
        let displaced_color = textureSample(color_tex, linear_sampler, inward_uv);
        if warble_displaced_sample_is_usable(base_color, displaced_color) {
            return composite_displaced_over_base(base_color, displaced_color);
        }
    }

    return base_color;
}
