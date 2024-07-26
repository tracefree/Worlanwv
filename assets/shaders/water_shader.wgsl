#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_functions::mesh_position_clip_to_local
#import bevy_pbr::view_transformations::{direction_world_to_view, position_world_to_view, depth_ndc_to_view_z, position_clip_to_view, position_ndc_to_view}

@group(2) @binding(0) var<uniform> time: f32;

fn fresnel(normal: vec3<f32>, view: vec3<f32>, amount: f32) -> f32 {
    return pow(1.0 - clamp(dot(normal, view), 0.0, 0.9), amount);
}

fn edge(depth: f32) -> f32 {
    let depth_ndc = 2.0 * depth - 1.0;
    var near = 0.25;
    var far = 1000.0;
    return near * far / (far + depth * (near - far));
}

@fragment
fn fragment(
#ifdef MULTISAMPLED
    @builtin(sample_index): u32,
#endif
    vertex: VertexOutput,
) -> @location(0) vec4<f32> {
#ifndef MULTISAMPLED
    let sample_index = 0u;
#endif
    var normal = normalize(direction_world_to_view(vertex.world_normal));
    var view = -normalize(position_world_to_view(vertex.world_position.xyz));

    var color_blue = vec3(0.3, 0.3, 0.8);
    var color_foam = vec3(1.0);
   // var factor = sin((vertex.uv * 1000.0) + time);

    var depth = bevy_pbr::prepass_utils::prepass_depth(vertex.position, sample_index);
    let foam_factor = smoothstep(0.5, 1.0, abs(position_world_to_view(vertex.world_position.xyz).z - depth_ndc_to_view_z(depth)));
    var color = mix(color_foam, color_blue, foam_factor);
    return vec4(color, fresnel(normal, view, 1.0));
}
