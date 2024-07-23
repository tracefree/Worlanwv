#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var<uniform> time: f32;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return vec4(0.0, 0.0, time*time, 1.0);
    //return vec4(0.0, 0.0, 1.0, 1.0);
}
