struct PostUniforms {
    seed_phase_uv: vec4<f32>,
    dof_fog_depth: vec4<f32>,
    background: vec4<f32>,
    flags: vec4<f32>,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var color_tex: texture_2d<f32>;
@group(0) @binding(1) var bus_tex: texture_2d<f32>;
@group(0) @binding(2) var meta_tex: texture_2d<f32>;
@group(0) @binding(3) var nearest_sampler: sampler;
@group(0) @binding(4) var linear_sampler: sampler;
@group(0) @binding(5) var<uniform> uniforms: PostUniforms;
@group(0) @binding(6) var warble_uv_tex: texture_2d<f32>;
@group(0) @binding(7) var indexed_palette_tex: texture_2d<f32>;

const POST_EFFECT_TEXEL_UV_PACK_SCALE: f32 = 64.0;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 3.0,  1.0),
    );

    let position = positions[vertex_index];
    var out: VsOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = position * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

fn decode_byte(value: f32) -> u32 {
    return u32(round(clamp(value, 0.0, 1.0) * 255.0));
}

fn decode_gate(bus: vec4<f32>, meta_sample: vec4<f32>) -> u32 {
    let hi = decode_byte(bus.a);
    let lo = decode_byte(meta_sample.a);
    return (hi << 8u) | lo;
}

fn hash21(p: vec2<f32>) -> f32 {
    let q = vec2<f32>(
        dot(p, vec2<f32>(127.1, 311.7)),
        dot(p, vec2<f32>(269.5, 183.3)),
    );
    return fract(sin(q.x + q.y) * 43758.5453123);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash21(i + vec2<f32>(0.0, 0.0));
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn layered_noise(p: vec2<f32>) -> f32 {
    let n0 = value_noise(p);
    let n1 = value_noise(p * 2.03 + vec2<f32>(17.0, 31.0));
    let n2 = value_noise(p * 4.11 + vec2<f32>(43.0, 19.0));
    return n0 * 0.6 + n1 * 0.3 + n2 * 0.1;
}

fn sample_surface_color(uv: vec2<f32>) -> vec4<f32> {
    if uniforms.flags.w >= 0.5 {
        return textureSample(color_tex, linear_sampler, uv);
    }

    return textureSample(color_tex, nearest_sampler, uv);
}

fn sample_bus_meta(uv: vec2<f32>) -> vec4<f32> {
    let bus = textureSample(bus_tex, nearest_sampler, uv);
    let meta_sample = textureSample(meta_tex, nearest_sampler, uv);
    return vec4<f32>(bus.r, bus.b, bus.a, meta_sample.a);
}

fn texture_sample_matches(origin_bus: vec4<f32>, origin_meta: vec4<f32>, sample_uv: vec2<f32>) -> bool {
    let sample_bus = textureSample(bus_tex, nearest_sampler, sample_uv);
    let sample_meta_sample = textureSample(meta_tex, nearest_sampler, sample_uv);

    return decode_byte(sample_bus.r) == decode_byte(origin_bus.r)
        && decode_byte(sample_bus.g) == decode_byte(origin_bus.g)
        && decode_byte(sample_bus.b) == decode_byte(origin_bus.b)
        && decode_gate(sample_bus, sample_meta_sample) == decode_gate(origin_bus, origin_meta);
}

fn displaced_surface_influence(base_color: vec4<f32>, displaced_color: vec4<f32>) -> f32 {
    let base_alpha = clamp(base_color.a, 0.0, 1.0);
    let displaced_alpha = clamp(displaced_color.a, 0.0, 1.0);
    let shared_alpha = min(base_alpha, displaced_alpha);
    let footprint_alpha = displaced_alpha * (1.0 - base_alpha) * 0.82;
    let usable_alpha = max(shared_alpha, footprint_alpha);
    let gated_alpha = clamp((usable_alpha - 0.02) / 0.98, 0.0, 1.0);
    return sqrt(gated_alpha);
}

fn composite_displaced_over_base(base_color: vec4<f32>, displaced_color: vec4<f32>) -> vec4<f32> {
    let base_alpha = clamp(base_color.a, 0.0, 1.0);
    let displaced_alpha = clamp(displaced_color.a, 0.0, 1.0);
    let influence = displaced_surface_influence(base_color, displaced_color);

    let base_premul = base_color.rgb * base_alpha;
    let displaced_premul = displaced_color.rgb * displaced_alpha;
    let mixed_premul = mix(base_premul, displaced_premul, influence);
    let mixed_alpha = mix(base_alpha, displaced_alpha, influence);

    if mixed_alpha <= 0.0001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    return vec4<f32>(mixed_premul / mixed_alpha, mixed_alpha);
}

fn composite_color_over_background(color: vec4<f32>) -> vec3<f32> {
    let alpha = clamp(color.a, 0.0, 1.0);
    return mix(uniforms.background.rgb, color.rgb, alpha);
}

fn squared_rgb_distance(left: vec3<f32>, right: vec3<f32>) -> f32 {
    let delta = left - right;
    return dot(delta, delta);
}

fn resolve_indexed_color(color: vec4<f32>) -> vec4<f32> {
    if uniforms.flags.y < 0.5 {
        return color;
    }

    let palette_count = u32(uniforms.flags.z);
    if palette_count == 0u {
        return color;
    }

    let visible_rgb = composite_color_over_background(color);
    var best_rgb = visible_rgb;
    var best_distance = 1e9;

    for (var index = 0u; index < palette_count; index = index + 1u) {
        let palette_sample = textureLoad(indexed_palette_tex, vec2<i32>(i32(index), 0), 0);
        if palette_sample.a <= 0.0 {
            continue;
        }

        let distance = squared_rgb_distance(visible_rgb, palette_sample.rgb);
        if distance < best_distance {
            best_distance = distance;
            best_rgb = palette_sample.rgb;
        }
    }

    return vec4<f32>(best_rgb, 1.0);
}
