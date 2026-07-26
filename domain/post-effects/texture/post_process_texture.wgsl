const TEXTURE_SCALE_A: vec2<f32> = vec2<f32>(36.0, 72.0);
const TEXTURE_SCALE_B: vec2<f32> = vec2<f32>(36.0, 72.0);
const TEXTURE_AMPLITUDE_TEXELS: f32 = 0.5;

fn texture_displacement(local_uv: vec2<f32>, seed_step: f32, texel_uv_size: f32) -> vec2<f32> {
    let seed_a = hash21(vec2<f32>(seed_step, 17.0));
    let seed_b = hash21(vec2<f32>(seed_step, 53.0));
    let a = layered_noise(local_uv * TEXTURE_SCALE_A + vec2<f32>(seed_a * 37.0, seed_b * 61.0));
    let b = layered_noise(local_uv * TEXTURE_SCALE_B + vec2<f32>(seed_b * 43.0, seed_a * 29.0));
    let vector = vec2<f32>(a, b) * 2.0 - 1.0;
    return vector * (texel_uv_size * TEXTURE_AMPLITUDE_TEXELS);
}

fn texture_displaced_sample_is_usable(base_color: vec4<f32>, displaced_color: vec4<f32>) -> bool {
    return displaced_surface_influence(base_color, displaced_color) > 0.0;
}

fn texture_visibility_from_alpha(alpha: f32) -> f32 {
    let gated_alpha = clamp((alpha - 0.14) / 0.56, 0.0, 1.0);
    return gated_alpha * gated_alpha * (3.0 - 2.0 * gated_alpha);
}

fn resolve_texture_color(base_color: vec4<f32>, origin_bus: vec4<f32>, origin_meta: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let texture_code = decode_byte(origin_bus.r);
    if uniforms.flags.x < 0.5 || texture_code != 1u {
        return base_color;
    }

    let texel_uv_size = origin_meta.b / POST_EFFECT_TEXEL_UV_PACK_SCALE;
    if texel_uv_size <= 0.0 {
        return base_color;
    }

    let local_uv = origin_meta.xy;
    let displacement = texture_displacement(local_uv, uniforms.seed_phase_uv.x, texel_uv_size);

    let outward_uv = clamp(uv + displacement, vec2<f32>(0.0), vec2<f32>(1.0));
    if texture_sample_matches(origin_bus, origin_meta, outward_uv) {
        let displaced_color = textureSample(color_tex, linear_sampler, outward_uv);
        if texture_displaced_sample_is_usable(base_color, displaced_color) {
            let textured_color = composite_displaced_over_base(base_color, displaced_color);
            let texture_visibility = max(
                texture_visibility_from_alpha(clamp(base_color.a, 0.0, 1.0)),
                displaced_surface_influence(base_color, displaced_color),
            );
            return mix(base_color, textured_color, texture_visibility);
        }
    }

    let inward_uv = clamp(uv - displacement, vec2<f32>(0.0), vec2<f32>(1.0));
    if texture_sample_matches(origin_bus, origin_meta, inward_uv) {
        let displaced_color = textureSample(color_tex, linear_sampler, inward_uv);
        if texture_displaced_sample_is_usable(base_color, displaced_color) {
            let textured_color = composite_displaced_over_base(base_color, displaced_color);
            let texture_visibility = max(
                texture_visibility_from_alpha(clamp(base_color.a, 0.0, 1.0)),
                displaced_surface_influence(base_color, displaced_color),
            );
            return mix(base_color, textured_color, texture_visibility);
        }
    }

    return base_color;
}
