#import bevy_pbr::mesh_functions::mesh_position_clip_to_local
#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    morph::morph,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}


@group(2) @binding(0) var<uniform> time: f32;
#ifdef WEBGL2
@group(2) @binding(1) var<uniform> _webgl2_padding: vec3<f32>;
#endif


@fragment
fn fragment(
    vertex: VertexOutput,
) -> @location(0) vec4<f32> {
    var r = length(vertex.world_position);
    var theta = acos(vertex.world_position.y / r);
    var color_day = vec3(0.2, 0.2, 0.9);
    var color_night = vec3(0.05, 0.0, 0.1);
    var color = mix(color_night, color_day * 30.0, smoothstep(0.0, 0.5, time));
    var sunset = vec3(0.4, 0.2, 0.0);
    color = mix(color, sunset, theta * 0.2 * (1.0 - time));
    if (theta > 1.58) {
        color = vec3(0.021, 0.119, 0.023) * 0.0;
    }
    return vec4(color, 1.0);
    //return vec4(0.0, 0.0, 1.0, 1.0);
}
