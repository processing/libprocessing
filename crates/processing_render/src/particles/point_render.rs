use bevy::asset::embedded_asset;
use bevy::camera::visibility::{self, VisibilityClass};
use bevy::core_pipeline::core_3d::{
    CORE_3D_DEPTH_FORMAT, Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey, Transparent3d,
    TransparentSortingInfo3d,
};
use bevy::ecs::query::ROQueryItem;
use bevy::ecs::system::SystemParamItem;
use bevy::ecs::system::lifetimeless::{Read, SRes};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::mesh::allocator::MeshSlabs;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{
    AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, InputUniformIndex, PhaseItem,
    PhaseItemExtraIndex, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
    ViewBinnedRenderPhases, ViewSortedRenderPhases,
};
use bevy::render::render_resource::binding_types::{
    storage_buffer_read_only_sized, uniform_buffer,
};
use bevy::render::render_resource::{
    BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntries, BlendState, Buffer, Canonical, ColorTargetState, ColorWrites,
    CompareFunction, DepthStencilState, FragmentState, IndexFormat, PipelineCache, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderStages, Specializer,
    SpecializerKey, TextureFormat, Variants, VertexState,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::storage::GpuShaderBuffer;
use bevy::render::sync_world::MainEntity;
use bevy::render::view::{
    ExtractedView, RenderVisibleEntities, ViewUniform, ViewUniformOffset, ViewUniforms,
};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::shader::Shader;

use bevy::render::storage::ShaderBuffer;

use crate::geometry::Topology;

const SURFACE_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

pub struct ParticlesPointRenderPlugin;

impl Plugin for ParticlesPointRenderPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "point.wgsl");
        app.add_plugins(ExtractComponentPlugin::<ParticleRasterDraw>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<RasterBindGroups>()
            .add_render_command::<Opaque3d, DrawParticleRasterCommands>()
            .add_render_command::<Transparent3d, DrawParticleRasterCommands>()
            .add_systems(
                Render,
                prepare_raster_bind_groups.in_set(RenderSystems::PrepareBindGroups),
            )
            .add_systems(Render, queue_particle_raster.in_set(RenderSystems::Queue));
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<ParticleRasterPipeline>();
        }
    }
}

#[derive(Component, Clone, ExtractComponent)]
#[require(VisibilityClass)]
#[component(on_add = visibility::add_visibility_class::<ParticleRasterDraw>)]
pub struct ParticleRasterDraw {
    pub position: Handle<ShaderBuffer>,
    pub count: u32,
    pub topology: Topology,
    pub index: Option<Handle<ShaderBuffer>>,
    pub indirect: Option<Handle<ShaderBuffer>>,
    pub color: Option<Handle<ShaderBuffer>>,
    pub normal: Option<Handle<ShaderBuffer>>,
    /// The sketch's blend mode at draw time (`None` = opaque).
    pub blend: Option<BlendState>,
}

#[derive(Resource)]
struct ParticleRasterPipeline {
    view_layout: BindGroupLayout,
    storage_layout: BindGroupLayout,
    variants: Variants<RenderPipeline, RasterSpecializer>,
}

struct RasterSpecializer;

#[derive(Copy, Clone, PartialEq, Eq, Hash, SpecializerKey)]
struct RasterKey {
    samples: u32,
    topology: u8,
    has_color: bool,
    has_normal: bool,
    blend: Option<BlendState>,
    format: TextureFormat,
}

impl Specializer<RenderPipeline> for RasterSpecializer {
    type Key = RasterKey;
    fn specialize(
        &self,
        key: Self::Key,
        descriptor: &mut RenderPipelineDescriptor,
    ) -> Result<Canonical<Self::Key>, BevyError> {
        descriptor.multisample.count = key.samples;
        let topology = Topology::from_u8(key.topology).unwrap_or(Topology::PointList);
        descriptor.primitive.topology = topology.to_primitive_topology();

        let mut defs: Vec<&str> = Vec::new();
        if key.has_color {
            defs.push("HAS_COLORS");
        }
        if key.has_normal {
            defs.push("HAS_NORMALS");
        }
        if matches!(topology, Topology::TriangleList | Topology::TriangleStrip) {
            defs.push("SHADED");
        }
        for def in defs {
            descriptor.vertex.shader_defs.push(def.into());
            if let Some(fragment) = descriptor.fragment.as_mut() {
                fragment.shader_defs.push(def.into());
            }
        }

        if let Some(target) = descriptor
            .fragment
            .as_mut()
            .and_then(|f| f.targets.get_mut(0))
            .and_then(|t| t.as_mut())
        {
            target.format = key.format;
            target.blend = key.blend;
        }
        if key.blend.is_some() {
            if let Some(depth) = descriptor.depth_stencil.as_mut() {
                depth.depth_write_enabled = Some(false);
            }
        }
        Ok(key)
    }
}

impl FromWorld for ParticleRasterPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>().clone();
        let asset_server = world.resource::<AssetServer>();
        let shader: Handle<Shader> =
            asset_server.load("embedded://processing_render/particles/point.wgsl");

        let view_entries: Vec<_> = BindGroupLayoutEntries::single(
            ShaderStages::VERTEX,
            uniform_buffer::<ViewUniform>(true),
        )
        .to_vec();
        let view_layout =
            render_device.create_bind_group_layout("point_view_layout", &view_entries);
        let storage_entries: Vec<_> = BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX,
            (
                storage_buffer_read_only_sized(false, None),
                storage_buffer_read_only_sized(false, None),
                storage_buffer_read_only_sized(false, None),
            ),
        )
        .to_vec();
        let storage_layout =
            render_device.create_bind_group_layout("point_storage_layout", &storage_entries);

        let base_descriptor = RenderPipelineDescriptor {
            label: Some("particle_raster_pipeline".into()),
            layout: vec![
                BindGroupLayoutDescriptor {
                    label: "point_view_layout".into(),
                    entries: view_entries,
                },
                BindGroupLayoutDescriptor {
                    label: "point_storage_layout".into(),
                    entries: storage_entries,
                },
            ],
            vertex: VertexState {
                shader: shader.clone(),
                entry_point: Some("vertex".into()),
                buffers: vec![],
                ..default()
            },
            fragment: Some(FragmentState {
                shader,
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: SURFACE_FORMAT,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::PointList,
                ..default()
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::GreaterEqual),
                stencil: default(),
                bias: default(),
            }),
            ..default()
        };

        Self {
            view_layout,
            storage_layout,
            variants: Variants::new(RasterSpecializer, base_descriptor),
        }
    }
}

struct RasterEntry {
    storage_bg: BindGroup,
    count: u32,
    index: Option<Buffer>,
    indirect: Option<Buffer>,
}

#[derive(Resource, Default)]
struct RasterBindGroups {
    view: Option<BindGroup>,
    entries: HashMap<MainEntity, RasterEntry>,
}

fn prepare_raster_bind_groups(
    render_device: Res<RenderDevice>,
    pipeline: Res<ParticleRasterPipeline>,
    view_uniforms: Res<ViewUniforms>,
    gpu_buffers: Res<RenderAssets<GpuShaderBuffer>>,
    draws: Query<(&MainEntity, &ParticleRasterDraw)>,
    mut bind_groups: ResMut<RasterBindGroups>,
) {
    bind_groups.view = view_uniforms.uniforms.binding().map(|binding| {
        render_device.create_bind_group(
            "point_view_bind_group",
            &pipeline.view_layout,
            &BindGroupEntries::single(binding),
        )
    });

    bind_groups.entries.clear();
    for (main_entity, draw) in draws.iter() {
        let Some(position) = gpu_buffers.get(&draw.position) else {
            continue;
        };
        let (index, indirect) = match (&draw.index, &draw.indirect) {
            (Some(index_handle), Some(indirect_handle)) => {
                let (Some(index_gpu), Some(indirect_gpu)) = (
                    gpu_buffers.get(index_handle),
                    gpu_buffers.get(indirect_handle),
                ) else {
                    continue;
                };
                (
                    Some(index_gpu.buffer.clone()),
                    Some(indirect_gpu.buffer.clone()),
                )
            }
            _ => (None, None),
        };
        let color_buffer = match &draw.color {
            Some(handle) => match gpu_buffers.get(handle) {
                Some(gpu) => &gpu.buffer,
                None => continue,
            },
            None => &position.buffer,
        };
        let normal_buffer = match &draw.normal {
            Some(handle) => match gpu_buffers.get(handle) {
                Some(gpu) => &gpu.buffer,
                None => continue,
            },
            None => &position.buffer,
        };
        let storage_bg = render_device.create_bind_group(
            "point_storage_bind_group",
            &pipeline.storage_layout,
            &BindGroupEntries::sequential((
                position.buffer.as_entire_binding(),
                color_buffer.as_entire_binding(),
                normal_buffer.as_entire_binding(),
            )),
        );
        bind_groups.entries.insert(
            *main_entity,
            RasterEntry {
                storage_bg,
                count: draw.count,
                index,
                indirect,
            },
        );
    }
}

fn queue_particle_raster(
    pipeline_cache: Res<PipelineCache>,
    mut pipeline: ResMut<ParticleRasterPipeline>,
    mut opaque_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    mut transparent_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    raster_draws: Query<&ParticleRasterDraw>,
) {
    let opaque_draw = opaque_draw_functions
        .read()
        .id::<DrawParticleRasterCommands>();
    let transparent_draw = transparent_draw_functions
        .read()
        .id::<DrawParticleRasterCommands>();

    for (view, visible, msaa) in views.iter() {
        let Some(visible_raster) = visible.get::<ParticleRasterDraw>() else {
            continue;
        };
        let mut opaque = opaque_phases.get_mut(&view.retained_view_entity);
        let mut transparent = transparent_phases.get_mut(&view.retained_view_entity);

        if let Some(op) = opaque.as_deref_mut() {
            for (_, main_entity) in &visible_raster.removed_entities {
                op.remove(*main_entity);
            }
        }
        for (render_entity, main_entity) in visible_raster.iter_visible() {
            if let Some(op) = opaque.as_deref_mut() {
                op.remove(*main_entity);
            }

            let draw = raster_draws.get(*render_entity).ok();
            let topology = draw.map(|d| d.topology).unwrap_or(Topology::PointList);
            let blended = draw.is_some_and(|d| d.blend.is_some());
            let indexed = draw.is_some_and(|d| d.index.is_some());

            let Ok(pipeline_id) = pipeline.variants.specialize(
                &pipeline_cache,
                RasterKey {
                    samples: msaa.samples(),
                    topology: topology as u8,
                    has_color: draw.is_some_and(|d| d.color.is_some()),
                    has_normal: draw.is_some_and(|d| d.normal.is_some()),
                    blend: draw.and_then(|d| d.blend),
                    format: view.target_format,
                },
            ) else {
                continue;
            };

            if blended {
                if let Some(tp) = transparent.as_deref_mut() {
                    tp.add_transient(Transparent3d {
                        // Sort by the view depth of the particle field's origin.
                        // NOT AlwaysOnTop: its -inf key sorts BEFORE the
                        // background quad (whose z-layer distance is a large
                        // negative batch offset), which painted the background
                        // over the particles.
                        sorting_info: TransparentSortingInfo3d::Sorted {
                            mesh_center: Vec3::ZERO,
                            depth_bias: 0.0,
                        },
                        distance: 0.0,
                        pipeline: pipeline_id,
                        entity: (*render_entity, *main_entity),
                        draw_function: transparent_draw,
                        batch_range: 0..1,
                        extra_index: PhaseItemExtraIndex::None,
                        indexed,
                    });
                }
            } else if let Some(op) = opaque.as_deref_mut() {
                op.add(
                    Opaque3dBatchSetKey {
                        draw_function: opaque_draw,
                        pipeline: pipeline_id,
                        material_bind_group_index: None,
                        lightmap_slab: None,
                        slabs: MeshSlabs::default(),
                    },
                    Opaque3dBinKey {
                        asset_id: AssetId::<Mesh>::invalid().untyped(),
                    },
                    (*render_entity, *main_entity),
                    InputUniformIndex::default(),
                    BinnedRenderPhaseType::NonMesh,
                );
            }
        }
    }
}

type DrawParticleRasterCommands = (
    SetItemPipeline,
    SetRasterViewBindGroup<0>,
    SetRasterStorageBindGroup<1>,
    DrawParticleRaster,
);

struct SetRasterViewBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetRasterViewBindGroup<I> {
    type Param = SRes<RasterBindGroups>;
    type ViewQuery = Read<ViewUniformOffset>;
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view_offset: ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, '_, Self::ItemQuery>>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(view_bg) = bind_groups.into_inner().view.as_ref() else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, view_bg, &[view_offset.offset]);
        RenderCommandResult::Success
    }
}

struct SetRasterStorageBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetRasterStorageBindGroup<I> {
    type Param = SRes<RasterBindGroups>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        _view: ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, '_, Self::ItemQuery>>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(entry) = bind_groups.into_inner().entries.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &entry.storage_bg, &[]);
        RenderCommandResult::Success
    }
}

struct DrawParticleRaster;
impl<P: PhaseItem> RenderCommand<P> for DrawParticleRaster {
    type Param = SRes<RasterBindGroups>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        _view: ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, '_, Self::ItemQuery>>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(entry) = bind_groups.into_inner().entries.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        match (&entry.index, &entry.indirect) {
            (Some(index), Some(indirect)) => {
                pass.set_index_buffer(index.slice(..), IndexFormat::Uint32);
                pass.draw_indexed_indirect(indirect, 0);
            }
            _ => pass.draw(0..entry.count, 0..1),
        }
        RenderCommandResult::Success
    }
}
