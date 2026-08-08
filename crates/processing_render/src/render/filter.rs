//! A [`Filter`] wraps a shader as a fullscreen pass.

use std::collections::{BTreeSet, HashMap};

use bevy::{
    core_pipeline::FullscreenShader,
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        render_asset::RenderAssets,
        render_resource::{
            BindGroupEntry, BindGroupLayoutDescriptor, CachedPipelineState, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, CommandEncoderDescriptor, FragmentState, LoadOp,
            Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerDescriptor, StoreOp, TextureFormat,
        },
        renderer::{RenderDevice, RenderQueue},
        storage::GpuShaderBuffer,
        sync_world::MainEntity,
        texture::GpuImage,
        view::ViewTarget,
    },
    shader::Shader as ShaderAsset,
};
use bevy_naga_reflect::{dynamic_shader::DynamicShader, reflect::ParameterCategory};

use crate::material::custom::{Shader, apply_reflect_field, find_param_containing_field};
use processing_core::error::{ProcessingError, Result};

const F_RESOLUTION: &str = "resolution";
const F_TEXEL_SIZE: &str = "texel_size";
const F_PASS_INDEX: &str = "pass_index";
const F_PASS_COUNT: &str = "pass_count";

pub mod builtin {
    pub const INVERT: &str = include_str!("filters/invert.wgsl");
    pub const GRAY: &str = include_str!("filters/gray.wgsl");
    pub const THRESHOLD: &str = include_str!("filters/threshold.wgsl");
    pub const POSTERIZE: &str = include_str!("filters/posterize.wgsl");
    pub const OPAQUE: &str = include_str!("filters/opaque.wgsl");
    pub const ERODE: &str = include_str!("filters/erode.wgsl");
    pub const DILATE: &str = include_str!("filters/dilate.wgsl");
    pub const BLUR: &str = include_str!("filters/blur.wgsl");
    pub const COMPOSITE: &str = include_str!("filters/composite.wgsl");
    pub const FEEDBACK: &str = include_str!("filters/feedback.wgsl");
}

#[derive(Component)]
pub struct Filter {
    pub shader: DynamicShader,
    pub entry_point: String,
    pub handle: Handle<ShaderAsset>,
    pub layouts: Vec<(u32, BindGroupLayoutDescriptor)>,
    pub passes: u32,
    pub pipelines: HashMap<TextureFormat, CachedRenderPipelineId>,
}

#[derive(Resource)]
struct FilterSampler(Sampler);

#[derive(Resource, Default)]
pub struct FilterRegistry {
    /// User filters, keyed by the shader entity they wrap.
    pub(crate) by_shader: HashMap<Entity, Entity>,
    /// Built-in filters, keyed by their shader source.
    pub(crate) builtins: HashMap<&'static str, Entity>,
}

pub struct FilterPlugin;

impl Plugin for FilterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FilterRegistry>();
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(RenderStartup, init_filter_sampler);
        }
    }
}

fn init_filter_sampler(mut commands: Commands, render_device: Res<RenderDevice>) {
    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    commands.insert_resource(FilterSampler(sampler));
}

pub fn create(app: &mut App, shader_entity: Entity) -> Result<Entity> {
    if let Some(&filter) = app
        .world()
        .resource::<FilterRegistry>()
        .by_shader
        .get(&shader_entity)
    {
        return Ok(filter);
    }

    let (module, handle) = {
        let program = app
            .world()
            .get::<Shader>(shader_entity)
            .ok_or(ProcessingError::ShaderNotFound)?;
        (program.module.clone(), program.shader_handle.clone())
    };

    let entry_point = module
        .entry_points
        .iter()
        .find(|e| e.stage == naga::ShaderStage::Fragment)
        .map(|e| e.name.clone())
        .ok_or_else(|| {
            ProcessingError::ShaderCompilationError("filter shader has no fragment entry".into())
        })?;

    let mut shader = DynamicShader::new(module)
        .map_err(|e| ProcessingError::ShaderCompilationError(e.to_string()))?;
    shader.init();
    let layouts = reflected_layouts(&shader);

    let filter = app
        .world_mut()
        .spawn(Filter {
            shader,
            entry_point,
            handle,
            layouts,
            passes: 1,
            pipelines: HashMap::new(),
        })
        .id();
    app.world_mut()
        .resource_mut::<FilterRegistry>()
        .by_shader
        .insert(shader_entity, filter);
    Ok(filter)
}

pub fn set_passes(
    In((entity, passes)): In<(Entity, u32)>,
    mut filters: Query<&mut Filter>,
) -> Result<()> {
    let mut filter = filters
        .get_mut(entity)
        .map_err(|_| ProcessingError::FilterNotFound)?;
    filter.passes = passes.max(1);
    Ok(())
}

pub fn apply(app: &mut App, graphics: Entity, filter: Entity) -> Result<()> {
    let (shader, handle, entry, layouts, passes) = {
        let f = app
            .world()
            .get::<Filter>(filter)
            .ok_or(ProcessingError::FilterNotFound)?;
        (
            f.shader.clone(),
            f.handle.clone(),
            f.entry_point.clone(),
            f.layouts.clone(),
            f.passes,
        )
    };

    // flush() also extracts the shader asset to the render world.
    crate::graphics::flush(app, graphics)?;

    // The pipeline must target the view target's actual texture format, which
    // bevy may transform from the requested format (HDR, sRGB compositing).
    let (format, size) = app
        .sub_app_mut(RenderApp)
        .world_mut()
        .run_system_cached_with(target_info, graphics)
        .unwrap()?;

    let pipeline_id = match app
        .world()
        .get::<Filter>(filter)
        .and_then(|f| f.pipelines.get(&format).copied())
    {
        Some(id) => id,
        None => {
            let id = app
                .sub_app_mut(RenderApp)
                .world_mut()
                .run_system_cached_with(queue_pipeline, (handle, entry, layouts.clone(), format))
                .unwrap();
            if let Some(mut f) = app.world_mut().get_mut::<Filter>(filter) {
                f.pipelines.insert(format, id);
            }
            id
        }
    };

    const MAX_WAIT: u32 = 64;
    let mut ready = false;
    for _ in 0..MAX_WAIT {
        ready = app
            .sub_app_mut(RenderApp)
            .world_mut()
            .run_system_cached_with(pump_pipeline, pipeline_id)
            .unwrap()?;
        if ready {
            break;
        }
    }
    if !ready {
        return Err(ProcessingError::PipelineNotReady(MAX_WAIT));
    }

    app.sub_app_mut(RenderApp)
        .world_mut()
        .run_system_cached_with(
            run_pass,
            (graphics, shader, layouts, pipeline_id, size, passes),
        )
        .unwrap()
}

pub fn destroy(
    In(entity): In<Entity>,
    mut commands: Commands,
    mut registry: ResMut<FilterRegistry>,
) -> Result<()> {
    registry.by_shader.retain(|_, &mut f| f != entity);
    registry.builtins.retain(|_, &mut f| f != entity);
    commands.entity(entity).despawn();
    Ok(())
}

fn reflected_layouts(shader: &DynamicShader) -> Vec<(u32, BindGroupLayoutDescriptor)> {
    let reflection = shader.reflection();
    let groups: BTreeSet<u32> = reflection.parameters().map(|p| p.group()).collect();
    groups
        .into_iter()
        .map(|group| {
            (
                group,
                BindGroupLayoutDescriptor {
                    label: "processing_filter_layout".into(),
                    entries: reflection.bind_group_layout(group),
                },
            )
        })
        .collect()
}

fn target_info(
    In(entity): In<Entity>,
    targets: Query<(&MainEntity, &ViewTarget)>,
) -> Result<(TextureFormat, UVec2)> {
    for (main_entity, vt) in targets.iter() {
        if **main_entity == entity {
            let size = vt.main_texture().size();
            return Ok((
                vt.main_texture_format(),
                UVec2::new(size.width, size.height),
            ));
        }
    }
    Err(ProcessingError::GraphicsNotFound)
}

fn queue_pipeline(
    In((handle, entry, layouts, format)): In<(
        Handle<ShaderAsset>,
        String,
        Vec<(u32, BindGroupLayoutDescriptor)>,
        TextureFormat,
    )>,
    pipeline_cache: Res<PipelineCache>,
    fullscreen: Res<FullscreenShader>,
) -> CachedRenderPipelineId {
    let max_group = layouts.iter().map(|(g, _)| *g).max().map_or(0, |g| g + 1);
    let mut layout = vec![BindGroupLayoutDescriptor::default(); max_group as usize];
    for (group, desc) in &layouts {
        layout[*group as usize] = desc.clone();
    }

    let descriptor = RenderPipelineDescriptor {
        label: Some("processing_filter_pipeline".into()),
        layout,
        vertex: fullscreen.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: handle,
            entry_point: Some(entry.into()),
            targets: vec![Some(ColorTargetState {
                format,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    };
    pipeline_cache.queue_render_pipeline(descriptor)
}

fn pump_pipeline(
    In(id): In<CachedRenderPipelineId>,
    mut pipeline_cache: ResMut<PipelineCache>,
) -> Result<bool> {
    pipeline_cache.process_queue();
    match pipeline_cache.get_render_pipeline_state(id) {
        CachedPipelineState::Ok(_) => Ok(true),
        CachedPipelineState::Err(e) => Err(ProcessingError::PipelineCompileError(format!("{e}"))),
        _ => Ok(false),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_pass(
    In((entity, mut shader, layouts, pipeline_id, size, passes)): In<(
        Entity,
        DynamicShader,
        Vec<(u32, BindGroupLayoutDescriptor)>,
        CachedRenderPipelineId,
        UVec2,
        u32,
    )>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    gpu_buffers: Res<RenderAssets<GpuShaderBuffer>>,
    filter_sampler: Res<FilterSampler>,
    targets: Query<(&MainEntity, &ViewTarget)>,
) -> Result<()> {
    let pipeline = pipeline_cache
        .get_render_pipeline(pipeline_id)
        .ok_or(ProcessingError::PipelineNotReady(0))?;

    let mut view_target = None;
    for (main_entity, vt) in targets.iter() {
        if **main_entity == entity {
            view_target = Some(vt);
            break;
        }
    }
    let view_target = view_target.ok_or(ProcessingError::GraphicsNotFound)?;

    // Fill the engine-owned `filter` uniform (only the fields the shader uses).
    let resolution = Vec2::new(size.x as f32, size.y as f32);
    let texel = Vec2::new(1.0 / resolution.x, 1.0 / resolution.y);
    fill_system(&mut shader, F_RESOLUTION, &resolution);
    fill_system(&mut shader, F_TEXEL_SIZE, &texel);
    fill_system(&mut shader, F_PASS_COUNT, &passes);
    let has_pass_index = find_param_containing_field(&shader, F_PASS_INDEX).is_some();

    // wesl mangles the imported `processing::filter` bindings, so the screen
    // texture and sampler are bound by their reflected (mangled) names, found by
    // category in group 0.
    let (input_texture, input_sampler) = {
        let reflection = shader.reflection();
        let mut texture = None;
        let mut sampler = None;
        for param in reflection.parameters().filter(|p| p.group() == 0) {
            match param.category() {
                ParameterCategory::Texture => texture = param.name().map(String::from),
                ParameterCategory::Sampler => sampler = param.name().map(String::from),
                _ => {}
            }
        }
        (texture, sampler)
    };

    for pass in 0..passes {
        if has_pass_index {
            let _ = apply_reflect_field(&mut shader, F_PASS_INDEX, &pass);
        }

        let post_process = view_target.post_process_write();
        if let Some(name) = &input_texture {
            shader.insert_texture_view(name, post_process.source.clone());
        }
        if let Some(name) = &input_sampler {
            shader.insert_sampler(name, filter_sampler.0.clone());
        }

        let reflection = shader.reflection();
        let mut bind_groups = Vec::new();
        for (group, desc) in &layouts {
            let layout = pipeline_cache.get_bind_group_layout(desc);
            let bindings = reflection.create_bindings(
                *group,
                &shader,
                &render_device,
                &gpu_images,
                &gpu_buffers,
            );
            let entries: Vec<BindGroupEntry> = bindings
                .iter()
                .map(|(binding, resource)| BindGroupEntry {
                    binding: *binding,
                    resource: resource.get_binding(),
                })
                .collect();
            bind_groups.push((
                *group,
                render_device.create_bind_group(
                    Some("processing_filter_bind_group"),
                    &layout,
                    &entries,
                ),
            ));
        }

        let mut encoder =
            render_device.create_command_encoder(&CommandEncoderDescriptor::default());
        {
            let mut render = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("processing_filter_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: post_process.destination,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render.set_pipeline(pipeline);
            for (group, bind_group) in &bind_groups {
                render.set_bind_group(*group, bind_group, &[]);
            }
            render.draw(0..3, 0..1);
        }
        render_queue.submit(std::iter::once(encoder.finish()));
    }

    Ok(())
}

fn fill_system(shader: &mut DynamicShader, name: &str, value: &dyn PartialReflect) {
    if find_param_containing_field(shader, name).is_some() {
        let _ = apply_reflect_field(shader, name, value);
    }
}
