pub mod custom;
pub mod pbr;

use crate::compute;
use crate::render::material::UntypedMaterial;
use crate::shader_value::ShaderValue;
use bevy::material::descriptor::RenderPipelineDescriptor;
use bevy::material::specialize::SpecializedMeshPipelineError;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, BlendState, Face};
use bevy::shader::ShaderRef;
use bevy_naga_reflect::reflect::ParameterCategory;
use processing_core::error::{self, ProcessingError};

pub struct ProcessingMaterialPlugin;

impl Plugin for ProcessingMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::pbr::MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, ProcessingMaterial>,
        >::default());

        let world = app.world_mut();
        let handle = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                unlit: true,
                cull_mode: None,
                base_color: Color::WHITE,
                ..default()
            });
        let entity = world.spawn(UntypedMaterial(handle.untyped())).id();
        world.insert_resource(DefaultMaterial(entity));
    }
}

#[derive(Resource)]
pub struct DefaultMaterial(pub Entity);

pub fn create_pbr(
    mut commands: Commands,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ProcessingMaterial>>>,
) -> Entity {
    let handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            unlit: false,
            cull_mode: None,
            ..default()
        },
        extension: ProcessingMaterial {
            blend_state: None,
            depth_write: None,
        },
    });
    commands.spawn(UntypedMaterial(handle.untyped())).id()
}

pub fn set_property(
    In((entity, name, value)): In<(Entity, String, ShaderValue)>,
    material_handles: Query<&UntypedMaterial>,
    images: Query<&crate::image::Image>,
    mut extended_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ProcessingMaterial>>>,
    mut particles_materials: ResMut<Assets<crate::particles::material::ParticlesMaterial>>,
    mut custom_materials: ResMut<Assets<custom::CustomMaterial>>,
    mut p_buffers: Query<&mut compute::Buffer>,
) -> error::Result<()> {
    let texture_handle = match &value {
        ShaderValue::Texture(img_entity) => Some(
            images
                .get(*img_entity)
                .map_err(|_| ProcessingError::ImageNotFound)?
                .handle
                .clone(),
        ),
        _ => None,
    };

    let untyped = material_handles
        .get(entity)
        .map_err(|_| ProcessingError::MaterialNotFound)?;

    if let Ok(handle) = untyped
        .0
        .clone()
        .try_typed::<ExtendedMaterial<StandardMaterial, ProcessingMaterial>>()
    {
        let mut extended = extended_materials
            .get_mut(&handle)
            .ok_or(ProcessingError::MaterialNotFound)?;
        return pbr::set_property(&mut extended.base, &name, &value, texture_handle);
    }

    if let Ok(handle) = untyped
        .0
        .clone()
        .try_typed::<crate::particles::material::ParticlesMaterial>()
    {
        let mut extended = particles_materials
            .get_mut(&handle)
            .ok_or(ProcessingError::MaterialNotFound)?;
        return pbr::set_property(&mut extended.base, &name, &value, texture_handle);
    }

    if let Ok(handle) = untyped.0.clone().try_typed::<custom::CustomMaterial>() {
        let mut mat = custom_materials
            .get_mut(&handle)
            .ok_or(ProcessingError::MaterialNotFound)?;

        if let ShaderValue::Buffer(buf_entity) = &value {
            let mut buffer = p_buffers
                .get_mut(*buf_entity)
                .map_err(|_| ProcessingError::BufferNotFound)?;

            let category = mat
                .shader
                .reflection()
                .parameter(&name)
                .map(|p| p.category())
                .ok_or_else(|| ProcessingError::UnknownShaderProperty(name.clone()))?;

            let ParameterCategory::Storage { read_only } = category else {
                return Err(ProcessingError::InvalidArgument(format!(
                    "property `{name}` expects {category:?}, got Buffer"
                )));
            };
            mat.shader.insert(&name, buffer.handle.clone());
            if !read_only {
                buffer.bound_rw = true;
            }
            return Ok(());
        }

        return custom::set_property(&mut mat, &name, &value);
    }

    Err(ProcessingError::MaterialNotFound)
}

type PbrMaterial = ExtendedMaterial<StandardMaterial, ProcessingMaterial>;

enum MaterialMut<'a> {
    Pbr(&'a mut PbrMaterial),
    Particles(&'a mut crate::particles::material::ParticlesMaterial),
    Custom(&'a mut custom::CustomMaterial),
}

fn edit_material(
    entity: Entity,
    material_handles: &Query<&UntypedMaterial>,
    pbr: &mut Assets<PbrMaterial>,
    particles: &mut Assets<crate::particles::material::ParticlesMaterial>,
    custom: &mut Assets<custom::CustomMaterial>,
    f: impl FnOnce(MaterialMut) -> error::Result<()>,
) -> error::Result<()> {
    let untyped = material_handles
        .get(entity)
        .map_err(|_| ProcessingError::MaterialNotFound)?;
    if let Ok(handle) = untyped.0.clone().try_typed::<PbrMaterial>() {
        let mat = pbr
            .get_mut(&handle)
            .ok_or(ProcessingError::MaterialNotFound)?
            .into_inner();
        return f(MaterialMut::Pbr(mat));
    }
    if let Ok(handle) = untyped
        .0
        .clone()
        .try_typed::<crate::particles::material::ParticlesMaterial>()
    {
        let mat = particles
            .get_mut(&handle)
            .ok_or(ProcessingError::MaterialNotFound)?
            .into_inner();
        return f(MaterialMut::Particles(mat));
    }
    if let Ok(handle) = untyped.0.clone().try_typed::<custom::CustomMaterial>() {
        let mat = custom
            .get_mut(&handle)
            .ok_or(ProcessingError::MaterialNotFound)?
            .into_inner();
        return f(MaterialMut::Custom(mat));
    }
    Err(ProcessingError::MaterialNotFound)
}

fn resolve_alpha_mode(mode: u8, cutoff: f32) -> error::Result<AlphaMode> {
    Ok(match mode {
        0 => AlphaMode::Opaque,
        1 => AlphaMode::Mask(cutoff),
        2 => AlphaMode::Blend,
        3 => AlphaMode::Premultiplied,
        4 => AlphaMode::Add,
        5 => AlphaMode::Multiply,
        _ => {
            return Err(ProcessingError::InvalidArgument(format!(
                "unknown alpha_mode value: {mode}"
            )));
        }
    })
}

fn set_base_double_sided(base: &mut StandardMaterial, value: bool) {
    base.double_sided = value;
    base.cull_mode = if value { None } else { Some(Face::Back) };
}

pub fn set_alpha_mode(
    In((entity, mode, cutoff)): In<(Entity, u8, f32)>,
    material_handles: Query<&UntypedMaterial>,
    mut pbr: ResMut<Assets<PbrMaterial>>,
    mut particles: ResMut<Assets<crate::particles::material::ParticlesMaterial>>,
    mut custom: ResMut<Assets<custom::CustomMaterial>>,
) -> error::Result<()> {
    let alpha_mode = resolve_alpha_mode(mode, cutoff)?;
    let opaque = matches!(alpha_mode, AlphaMode::Opaque);
    edit_material(
        entity,
        &material_handles,
        &mut pbr,
        &mut particles,
        &mut custom,
        |m| {
            match m {
                MaterialMut::Pbr(mat) => {
                    mat.base.alpha_mode = alpha_mode;
                    if opaque {
                        mat.extension.blend_state = None;
                    }
                }
                MaterialMut::Particles(mat) => mat.base.alpha_mode = alpha_mode,
                MaterialMut::Custom(mat) => {
                    mat.alpha_mode = alpha_mode;
                    if opaque {
                        mat.blend_state = None;
                    }
                }
            }
            Ok(())
        },
    )
}

pub fn set_double_sided(
    In((entity, value)): In<(Entity, bool)>,
    material_handles: Query<&UntypedMaterial>,
    mut pbr: ResMut<Assets<PbrMaterial>>,
    mut particles: ResMut<Assets<crate::particles::material::ParticlesMaterial>>,
    mut custom: ResMut<Assets<custom::CustomMaterial>>,
) -> error::Result<()> {
    edit_material(
        entity,
        &material_handles,
        &mut pbr,
        &mut particles,
        &mut custom,
        |m| {
            match m {
                MaterialMut::Pbr(mat) => set_base_double_sided(&mut mat.base, value),
                MaterialMut::Particles(mat) => set_base_double_sided(&mut mat.base, value),
                MaterialMut::Custom(mat) => mat.double_sided = Some(value),
            }
            Ok(())
        },
    )
}

pub fn set_unlit(
    In((entity, value)): In<(Entity, bool)>,
    material_handles: Query<&UntypedMaterial>,
    mut pbr: ResMut<Assets<PbrMaterial>>,
    mut particles: ResMut<Assets<crate::particles::material::ParticlesMaterial>>,
    mut custom: ResMut<Assets<custom::CustomMaterial>>,
) -> error::Result<()> {
    edit_material(
        entity,
        &material_handles,
        &mut pbr,
        &mut particles,
        &mut custom,
        |m| match m {
            MaterialMut::Pbr(mat) => {
                mat.base.unlit = value;
                Ok(())
            }
            MaterialMut::Particles(mat) => {
                mat.base.unlit = value;
                Ok(())
            }
            MaterialMut::Custom(_) => Err(ProcessingError::InvalidArgument(
                "unlit is not applicable to custom-shader materials; the shader defines lighting"
                    .to_string(),
            )),
        },
    )
}

pub fn set_depth_write(
    In((entity, value)): In<(Entity, bool)>,
    material_handles: Query<&UntypedMaterial>,
    mut pbr: ResMut<Assets<PbrMaterial>>,
    mut particles: ResMut<Assets<crate::particles::material::ParticlesMaterial>>,
    mut custom: ResMut<Assets<custom::CustomMaterial>>,
) -> error::Result<()> {
    edit_material(
        entity,
        &material_handles,
        &mut pbr,
        &mut particles,
        &mut custom,
        |m| match m {
            MaterialMut::Pbr(mat) => {
                mat.extension.depth_write = Some(value);
                Ok(())
            }
            MaterialMut::Particles(mat) => {
                mat.extension.depth_write = Some(value);
                Ok(())
            }
            MaterialMut::Custom(mat) => {
                mat.depth_write = Some(value);
                Ok(())
            }
        },
    )
}

pub fn set_custom_blend(
    In((entity, blend)): In<(Entity, BlendState)>,
    material_handles: Query<&UntypedMaterial>,
    mut pbr: ResMut<Assets<PbrMaterial>>,
    mut particles: ResMut<Assets<crate::particles::material::ParticlesMaterial>>,
    mut custom: ResMut<Assets<custom::CustomMaterial>>,
) -> error::Result<()> {
    edit_material(
        entity,
        &material_handles,
        &mut pbr,
        &mut particles,
        &mut custom,
        |m| match m {
            MaterialMut::Pbr(mat) => {
                mat.extension.blend_state = Some(blend);
                mat.base.alpha_mode = AlphaMode::Blend;
                Ok(())
            }
            MaterialMut::Particles(mat) => {
                mat.extension.blend_state = Some(blend);
                mat.base.alpha_mode = AlphaMode::Blend;
                Ok(())
            }
            MaterialMut::Custom(mat) => {
                mat.blend_state = Some(blend);
                mat.alpha_mode = AlphaMode::Blend;
                Ok(())
            }
        },
    )
}

pub fn destroy(
    In(entity): In<Entity>,
    mut commands: Commands,
    material_handles: Query<&UntypedMaterial>,
    mut extended_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ProcessingMaterial>>>,
    mut custom_materials: ResMut<Assets<custom::CustomMaterial>>,
) -> error::Result<()> {
    let untyped = material_handles
        .get(entity)
        .map_err(|_| ProcessingError::MaterialNotFound)?;
    if let Ok(handle) = untyped
        .0
        .clone()
        .try_typed::<ExtendedMaterial<StandardMaterial, ProcessingMaterial>>()
    {
        extended_materials.remove(&handle);
    }
    if let Ok(handle) = untyped.0.clone().try_typed::<custom::CustomMaterial>() {
        custom_materials.remove(&handle);
    }
    commands.entity(entity).despawn();
    Ok(())
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
#[bind_group_data(ProcessingMaterialKey)]
pub struct ProcessingMaterial {
    pub blend_state: Option<BlendState>,
    pub depth_write: Option<bool>,
}

#[repr(C)]
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct ProcessingMaterialKey {
    blend_state: Option<BlendState>,
    depth_write: Option<bool>,
}

impl From<&ProcessingMaterial> for ProcessingMaterialKey {
    fn from(mat: &ProcessingMaterial) -> Self {
        ProcessingMaterialKey {
            blend_state: mat.blend_state,
            depth_write: mat.depth_write,
        }
    }
}

impl MaterialExtension for ProcessingMaterial {
    fn vertex_shader() -> ShaderRef {
        <StandardMaterial as Material>::vertex_shader()
    }

    fn fragment_shader() -> ShaderRef {
        <StandardMaterial as Material>::fragment_shader()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> std::result::Result<(), SpecializedMeshPipelineError> {
        apply_pipeline_state(
            descriptor,
            key.bind_group_data.blend_state,
            key.bind_group_data.depth_write,
        );
        Ok(())
    }
}

pub(crate) fn apply_pipeline_state(
    descriptor: &mut RenderPipelineDescriptor,
    blend_state: Option<BlendState>,
    depth_write: Option<bool>,
) {
    if let Some(blend_state) = blend_state
        && let Some(fragment_state) = &mut descriptor.fragment
    {
        for target in fragment_state.targets.iter_mut().flatten() {
            target.blend = Some(blend_state);
        }
    }
    if let Some(depth_write) = depth_write
        && let Some(depth_stencil) = &mut descriptor.depth_stencil
    {
        depth_stencil.depth_write_enabled = Some(depth_write);
    }
}
