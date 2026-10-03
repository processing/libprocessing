import bevy_pbr::render::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
    mesh_functions,
};

@if(PREPASS_PIPELINE)
import bevy_pbr::{
    prepass::io::{VertexOutput, FragmentOutput},
    deferred::functions::deferred_output,
};
@else
import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
};

@if(HAS_COLORS)
@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<storage, read> particle_colors: array<vec4<f32>>;

@if(HAS_EMISSIVE_COLORS)
@group(constants::MATERIAL_BIND_GROUP) @binding(101)
var<storage, read> particle_emissive_colors: array<vec4<f32>>;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    let tag = mesh_functions::get_tag(in.instance_index);

    @if(HAS_COLORS) {
        pbr_input.material.base_color = pbr_input.material.base_color * particle_colors[tag];
    }

    @if(HAS_EMISSIVE_COLORS) {
        pbr_input.material.emissive = vec4<f32>(
            pbr_input.material.emissive.rgb + particle_emissive_colors[tag].rgb,
            pbr_input.material.emissive.a
        );
    }

    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color
    );

    var out: FragmentOutput;
@if(PREPASS_PIPELINE) {
    out = deferred_output(in, pbr_input);
} @else {
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
}

    return out;
}
