#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_functions,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

#ifdef HAS_COLORS
@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<storage, read> particle_colors: array<vec4<f32>>;
#endif

#ifdef HAS_EMISSIVE_COLORS
@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var<storage, read> particle_emissive_colors: array<vec4<f32>>;
#endif

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    let tag = mesh_functions::get_tag(in.instance_index);

#ifdef HAS_COLORS
    pbr_input.material.base_color = pbr_input.material.base_color * particle_colors[tag];
#endif

#ifdef HAS_EMISSIVE_COLORS
    pbr_input.material.emissive = vec4<f32>(
        pbr_input.material.emissive.rgb + particle_emissive_colors[tag].rgb,
        pbr_input.material.emissive.a
    );
#endif

    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color
    );

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}
