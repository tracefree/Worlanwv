#import bevy_pbr::mesh_functions::mesh_position_clip_to_local
#import bevy_pbr::view_transformations::{direction_world_to_view, position_world_to_view, depth_ndc_to_view_z, position_clip_to_view, position_ndc_to_view}
#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}
#import bevy_pbr::mesh_view_bindings as view_bindings

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

@group(2) @binding(100) var<uniform> time: f32;
@group(2) @binding(102) var depth_sampler: sampler;

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
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
#ifndef MULTISAMPLED
    let sample_index = 0u;
#endif

    // Water material
    var normal = normalize(direction_world_to_view(vertex.world_normal));
    var view = -normalize(position_world_to_view(vertex.world_position.xyz));

    let screen_uv = vertex.position.xy * vec2<f32>(1.0 / 1280.0, 1.0 / 720.0);
    let depth = textureSample(view_bindings::depth_prepass_texture, depth_sampler, screen_uv);

   // let depth = bevy_pbr::prepass_utils::prepass_depth(vertex.position, sample_index);
    let foam_factor = smoothstep(0.5, 1.0, abs(position_world_to_view(vertex.world_position.xyz).z - depth_ndc_to_view_z(depth)));
    var color_blue = vec3(0.3, 0.3, 0.8);
    var color_foam = vec3(1.0);
    var color = mix(color_foam, color_blue, foam_factor);
    var alpha = smoothstep(0.0, 0.5, fresnel(normal, view, 1.0));

    // Feed into standard material
    // fresnel(normal, view, 1.0)
    var pbr_input = pbr_input_from_standard_material(vertex, is_front);
    pbr_input.material.base_color = vec4<f32>(color.rgb, alpha);

    var out: FragmentOutput;
    // Apply lighting and post-processing
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
  //  out.color = vec4<f32>(vec3<f32>(depth), 1.0);
  //  out.color = mix(out.color, vec4<f32>(vec3<f32>(deptha), 1.0), step(0.5, screen_uv.x));
    return out;
}
