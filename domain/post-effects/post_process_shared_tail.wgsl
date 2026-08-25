@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let base_color = sample_surface_color(in.uv);
    let origin_bus = textureSample(bus_tex, nearest_sampler, in.uv);
    let origin_meta = textureSample(meta_tex, nearest_sampler, in.uv);

    var color = resolve_warble_color(base_color, origin_bus, origin_meta, in.uv);
    color = resolve_texture_color(color, origin_bus, origin_meta, in.uv);
    color = resolve_indexed_color(color);
    return vec4<f32>(composite_color_over_background(color), 1.0);
}
