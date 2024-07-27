#import bevy_pbr::mesh_functions::mesh_position_clip_to_local
#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    morph::morph,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.world_position = vec4<f32>(vertex_no_morph.position * (-100000.0), 1.0);
    out.position = position_world_to_clip(vertex_no_morph.position.xyz * (-100000.0));

    return out;
}
