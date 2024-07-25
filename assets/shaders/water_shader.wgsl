#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_functions::mesh_position_clip_to_local
#import bevy_pbr::view_transformations::{direction_world_to_view, position_world_to_view}

@group(2) @binding(0) var<uniform> time: f32;

fn fresnel(normal: vec3<f32>, view: vec3<f32>, amount: f32) -> f32 {
    return pow(1.0 - clamp(dot(normal, view), 0.0, 0.9), amount);
}

@fragment
fn fragment(
    vertex: VertexOutput,
) -> @location(0) vec4<f32> {
    var normal = normalize(direction_world_to_view(vertex.world_normal));
    var view = -normalize(position_world_to_view(vertex.world_position.xyz));
    return vec4(0.3, 0.3, 0.8, fresnel(normal, view, 0.5));
}
