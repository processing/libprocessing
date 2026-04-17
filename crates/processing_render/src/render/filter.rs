//! Full-screen post-process filters (Processing 4 / p5.js `filter()`).

use bevy::{
    asset::embedded_asset,
    core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
    ecs::component::Component,
    math::Vec4,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        extract_component::{DynamicUniformIndex, ExtractComponent},
        render_resource::ShaderType,
    },
    shader::ShaderRef,
};


#[derive(Debug, Clone, Copy)]
pub enum FilterKind {
    Invert,
    Gray,
    Threshold { cutoff: f32 },
    Posterize { levels: u32 },
    Opaque,
}

#[derive(Debug, Clone, Copy)]
pub struct FilterOp {
    pub kind: FilterKind,
}

impl FilterOp {
    pub fn new(kind: FilterKind) -> Self {
        Self { kind }
    }
}

#[derive(Component, Clone, Copy, Default, ExtractComponent, ShaderType)]
pub struct InvertFilter {
    _padding: Vec4,
}

impl FullscreenMaterial for InvertFilter {
    fn fragment_shader() -> ShaderRef {
        "embedded://processing_render/render/filters/invert.wgsl".into()
    }
}

#[derive(Component, Clone, Copy, Default, ExtractComponent, ShaderType)]
pub struct GrayFilter {
    _padding: Vec4,
}

impl FullscreenMaterial for GrayFilter {
    fn fragment_shader() -> ShaderRef {
        "embedded://processing_render/render/filters/gray.wgsl".into()
    }
}

#[derive(Component, Clone, Copy, Default, ExtractComponent, ShaderType)]
pub struct ThresholdFilter {
    pub cutoff: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
}

impl FullscreenMaterial for ThresholdFilter {
    fn fragment_shader() -> ShaderRef {
        "embedded://processing_render/render/filters/threshold.wgsl".into()
    }
}

#[derive(Component, Clone, Copy, Default, ExtractComponent, ShaderType)]
pub struct PosterizeFilter {
    pub levels: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}

impl FullscreenMaterial for PosterizeFilter {
    fn fragment_shader() -> ShaderRef {
        "embedded://processing_render/render/filters/posterize.wgsl".into()
    }
}

#[derive(Component, Clone, Copy, Default, ExtractComponent, ShaderType)]
pub struct OpaqueFilter {
    _padding: Vec4,
}

impl FullscreenMaterial for OpaqueFilter {
    fn fragment_shader() -> ShaderRef {
        "embedded://processing_render/render/filters/opaque.wgsl".into()
    }
}

pub struct FilterPlugin;

impl Plugin for FilterPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "filters/invert.wgsl");
        embedded_asset!(app, "filters/gray.wgsl");
        embedded_asset!(app, "filters/threshold.wgsl");
        embedded_asset!(app, "filters/posterize.wgsl");
        embedded_asset!(app, "filters/opaque.wgsl");

        app.add_plugins((
            FullscreenMaterialPlugin::<InvertFilter>::default(),
            FullscreenMaterialPlugin::<GrayFilter>::default(),
            FullscreenMaterialPlugin::<ThresholdFilter>::default(),
            FullscreenMaterialPlugin::<PosterizeFilter>::default(),
            FullscreenMaterialPlugin::<OpaqueFilter>::default(),
        ));

        // Bevy's `UniformComponentPlugin` doesn't clear `DynamicUniformIndex<T>`
        // when `T` is removed. The fullscreen-material system queries on that
        // index, so without this cleanup the filter keeps running after detach.
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(
                Render,
                (
                    clear_stale_uniform_index::<InvertFilter>,
                    clear_stale_uniform_index::<GrayFilter>,
                    clear_stale_uniform_index::<ThresholdFilter>,
                    clear_stale_uniform_index::<PosterizeFilter>,
                    clear_stale_uniform_index::<OpaqueFilter>,
                )
                    .in_set(RenderSystems::PrepareResources),
            );
        }
    }
}

fn clear_stale_uniform_index<T: Component>(
    mut commands: Commands,
    stale: Query<Entity, (With<DynamicUniformIndex<T>>, Without<T>)>,
) {
    for e in &stale {
        commands.entity(e).remove::<DynamicUniformIndex<T>>();
    }
}

pub fn attach(entity_commands: &mut EntityWorldMut<'_>, op: FilterOp) {
    match op.kind {
        FilterKind::Invert => {
            entity_commands.insert(InvertFilter::default());
        }
        FilterKind::Gray => {
            entity_commands.insert(GrayFilter::default());
        }
        FilterKind::Threshold { cutoff } => {
            entity_commands.insert(ThresholdFilter {
                cutoff,
                ..default()
            });
        }
        FilterKind::Posterize { levels } => {
            entity_commands.insert(PosterizeFilter {
                levels,
                ..default()
            });
        }
        FilterKind::Opaque => {
            entity_commands.insert(OpaqueFilter::default());
        }
    }
}

pub fn detach(entity_commands: &mut EntityWorldMut<'_>) {
    entity_commands.remove::<InvertFilter>();
    entity_commands.remove::<GrayFilter>();
    entity_commands.remove::<ThresholdFilter>();
    entity_commands.remove::<PosterizeFilter>();
    entity_commands.remove::<OpaqueFilter>();
}
