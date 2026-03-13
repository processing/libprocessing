use std::any::TypeId;
use std::borrow::Cow;
use std::sync::Arc;

use bevy::platform::collections::hash_map::Entry;
use wesl::PkgResolver;
use wesl::syntax::{ModulePath, PathOrigin};

wesl::wesl_pkg!(processing);
wesl::wesl_pkg!(lygia);

use bevy::{
    asset::{AsAssetId, AssetEventSystems},
    core_pipeline::core_3d::Opaque3d,
    ecs::system::{
        SystemParamItem,
        lifetimeless::{SRes, SResMut},
    },
    material::{MaterialProperties, key::ErasedMeshPipelineKey},
    pbr::{
        DrawMaterial, EntitiesNeedingSpecialization, MainPassOpaqueDrawFunction,
        MaterialBindGroupAllocator, MaterialBindGroupAllocators, MaterialFragmentShader,
        MaterialVertexShader, MeshPipelineKey, PreparedMaterial, RenderMaterialBindings,
        RenderMaterialInstance, RenderMaterialInstances, base_specialize,
    },
    prelude::*,
    reflect::{PartialReflect, ReflectMut, ReflectRef, structs::Struct},
    render::{
        Extract, RenderApp, RenderStartup,
        camera::{DirtySpecializationSystems, DirtySpecializations},
        erased_render_asset::{ErasedRenderAsset, ErasedRenderAssetPlugin, PrepareAssetError},
        render_asset::RenderAssets,
        render_phase::DrawFunctions,
        render_resource::{BindGroupLayoutDescriptor, BindingResources, UnpreparedBindGroup},
        renderer::RenderDevice,
        sync_world::MainEntity,
        texture::GpuImage,
    },
};

use bevy_naga_reflect::dynamic_shader::DynamicShader;

use bevy::shader::Shader as ShaderAsset;

use crate::material::MaterialValue;
use crate::render::material::UntypedMaterial;
use processing_core::config::{Config, ConfigKey};
use processing_core::error::{ProcessingError, Result};

#[derive(Asset, TypePath, Clone)]
pub struct CustomMaterial {
    pub shader: DynamicShader,
    pub shader_handle: Handle<ShaderAsset>,
    pub has_vertex: bool,
    pub has_fragment: bool,
}

#[derive(Component)]
pub struct Shader {
    pub module: naga::Module,
    pub shader_handle: Handle<ShaderAsset>,
}

#[derive(Component, Clone)]
pub struct CustomMaterial3d(pub Handle<CustomMaterial>);

impl AsAssetId for CustomMaterial3d {
    type Asset = CustomMaterial;
    fn as_asset_id(&self) -> AssetId<Self::Asset> {
        self.0.id()
    }
}

struct ProcessingResolver<'a> {
    entry_source: &'a str,
    pkg_resolver: PkgResolver,
}

impl wesl::Resolver for ProcessingResolver<'_> {
    fn resolve_source<'a>(
        &'a self,
        path: &ModulePath,
    ) -> std::result::Result<Cow<'a, str>, wesl::ResolveError> {
        // Check for the entry module first (its parsed path has Package origin)
        if path.to_string() == "entry" {
            return Ok(Cow::Borrowed(self.entry_source));
        }

        match &path.origin {
            PathOrigin::Package(pkg) => {
                // Self-referential package imports: within a package, imports to
                // the same package stack the name (e.g. "lygia/lygia/lygia/...").
                // Collapse to the root package name before resolving.
                let root = pkg.split('/').next().unwrap();
                let mut fixed = path.clone();
                fixed.origin = PathOrigin::Package(root.to_string());
                self.pkg_resolver.resolve_source(&fixed)
            }
            _ => Err(wesl::ResolveError::ModuleNotFound(
                path.clone(),
                format!("unknown module: {}", path),
            )),
        }
    }
}

fn compile_shader(source: &str) -> Result<(String, naga::Module)> {
    let mut pkg_resolver = PkgResolver::new();
    pkg_resolver.add_package(&processing::PACKAGE);
    pkg_resolver.add_package(&lygia::PACKAGE);

    let resolver = ProcessingResolver {
        entry_source: source,
        pkg_resolver,
    };
    let module_path: ModulePath = "entry".parse().unwrap();
    let options = wesl::CompileOptions {
        imports: true,
        strip: false,
        ..Default::default()
    };
    let compiled = wesl::compile(&module_path, &resolver, &wesl::EscapeMangler, &options)
        .map_err(|e| ProcessingError::ShaderCompilationError(e.to_string()))?;
    let wgsl = compiled.to_string();
    let module = naga::front::wgsl::parse_str(&wgsl)
        .map_err(|e| ProcessingError::ShaderCompilationError(e.to_string()))?;
    Ok((wgsl, module))
}

pub fn create_shader(
    In(source): In<String>,
    mut commands: Commands,
    mut shaders: ResMut<Assets<ShaderAsset>>,
) -> Result<Entity> {
    let (compiled_wgsl, module) = compile_shader(&source)?;
    let shader_handle = shaders.add(ShaderAsset::from_wgsl(compiled_wgsl, "custom_material"));
    Ok(commands
        .spawn(Shader {
            module,
            shader_handle,
        })
        .id())
}

pub fn load_shader(In(path): In<std::path::PathBuf>, world: &mut World) -> Result<Entity> {
    use bevy::asset::{
        AssetPath, LoadState, handle_internal_asset_events,
        io::{AssetSourceId, embedded::GetAssetServer},
    };
    use bevy::ecs::system::RunSystemOnce;

    let config = world.resource::<Config>();
    let asset_path: AssetPath = match config.get(ConfigKey::AssetRootPath) {
        Some(_) => {
            AssetPath::from_path_buf(path).with_source(AssetSourceId::from("assets_directory"))
        }
        None => AssetPath::from_path_buf(path),
    };

    let handle: Handle<ShaderAsset> = world.get_asset_server().load(asset_path);

    while let LoadState::Loading = world.get_asset_server().load_state(&handle) {
        world.run_system_once(handle_internal_asset_events).unwrap();
    }

    let source = {
        let shader_assets = world.resource::<Assets<ShaderAsset>>();
        let shader = shader_assets
            .get(&handle)
            .ok_or(ProcessingError::ShaderNotFound)?;
        match &shader.source {
            bevy::shader::Source::Wesl(s) | bevy::shader::Source::Wgsl(s) => s.to_string(),
            _ => {
                return Err(ProcessingError::ShaderCompilationError(
                    "Unsupported shader source format".to_string(),
                ));
            }
        }
    };

    let (compiled_wgsl, module) = compile_shader(&source)?;

    let shader_handle = world
        .resource_mut::<Assets<ShaderAsset>>()
        .add(ShaderAsset::from_wgsl(compiled_wgsl, "custom_material"));

    Ok(world
        .spawn(Shader {
            module,
            shader_handle,
        })
        .id())
}

pub fn destroy_shader(In(entity): In<Entity>, mut commands: Commands) -> Result<()> {
    commands.entity(entity).despawn();
    Ok(())
}

pub fn create_custom(
    In(shader_entity): In<Entity>,
    mut commands: Commands,
    shader_programs: Query<&Shader>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
) -> Result<Entity> {
    let program = shader_programs
        .get(shader_entity)
        .map_err(|_| ProcessingError::ShaderNotFound)?;

    let has_vertex = program
        .module
        .entry_points
        .iter()
        .any(|ep| ep.stage == naga::ShaderStage::Vertex);
    let has_fragment = program
        .module
        .entry_points
        .iter()
        .any(|ep| ep.stage == naga::ShaderStage::Fragment);

    let mut shader = DynamicShader::new(program.module.clone())
        .map_err(|e| ProcessingError::ShaderCompilationError(e.to_string()))?;
    shader.init();

    let material = CustomMaterial {
        shader,
        shader_handle: program.shader_handle.clone(),
        has_vertex,
        has_fragment,
    };
    let handle = custom_materials.add(material);
    Ok(commands.spawn(UntypedMaterial(handle.untyped())).id())
}

pub fn set_property(
    material: &mut CustomMaterial,
    name: &str,
    value: &MaterialValue,
) -> Result<()> {
    let reflect_value: Box<dyn PartialReflect> = material_value_to_reflect(value)?;

    if let Some(field) = material.shader.field_mut(name) {
        field.apply(&*reflect_value);
        return Ok(());
    }

    let param_name = find_param_containing_field(&material.shader, name);
    if let Some(param_name) = param_name
        && let Some(param) = material.shader.field_mut(&param_name)
        && let ReflectMut::Struct(s) = param.reflect_mut()
        && let Some(field) = s.field_mut(name)
    {
        field.apply(&*reflect_value);
        return Ok(());
    }

    Err(ProcessingError::UnknownMaterialProperty(name.to_string()))
}

fn material_value_to_reflect(value: &MaterialValue) -> Result<Box<dyn PartialReflect>> {
    Ok(match value {
        MaterialValue::Float(v) => Box::new(*v),
        MaterialValue::Float2(v) => Box::new(Vec2::from_array(*v)),
        MaterialValue::Float3(v) => Box::new(Vec3::from_array(*v)),
        MaterialValue::Float4(v) => Box::new(Vec4::from_array(*v)),
        MaterialValue::Int(v) => Box::new(*v),
        MaterialValue::Int2(v) => Box::new(IVec2::from_array(*v)),
        MaterialValue::Int3(v) => Box::new(IVec3::from_array(*v)),
        MaterialValue::Int4(v) => Box::new(IVec4::from_array(*v)),
        MaterialValue::UInt(v) => Box::new(*v),
        MaterialValue::Mat4(v) => Box::new(Mat4::from_cols_array(v)),
        MaterialValue::Texture(_) => {
            return Err(ProcessingError::UnknownMaterialProperty(
                "Texture properties not yet supported for custom materials".to_string(),
            ));
        }
    })
}

fn find_param_containing_field(shader: &DynamicShader, field_name: &str) -> Option<String> {
    for i in 0..shader.field_len() {
        if let Some(field) = shader.field_at(i)
            && let ReflectRef::Struct(s) = field.reflect_ref()
            && s.field(field_name).is_some()
        {
            return shader.name_at(i).map(|s: &str| s.to_string());
        }
    }
    None
}

pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<CustomMaterial>()
            .add_plugins(ErasedRenderAssetPlugin::<CustomMaterial>::default())
            .add_systems(
                PostUpdate,
                check_entities_needing_specialization.after(AssetEventSystems),
            )
            .init_resource::<EntitiesNeedingSpecialization<CustomMaterial>>();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_custom_material_resources)
            .add_systems(
                ExtractSchedule,
                (
                    extract_custom_materials,
                    extract_custom_materials_needing_specialization
                        .in_set(DirtySpecializationSystems::CheckForChanges),
                    extract_custom_materials_that_need_specializations_removed
                        .in_set(DirtySpecializationSystems::CheckForRemovals),
                ),
            );
    }
}

fn init_custom_material_resources(
    mut bind_group_allocators: ResMut<MaterialBindGroupAllocators>,
    render_device: Res<RenderDevice>,
) {
    let bind_group_layout = BindGroupLayoutDescriptor::new("custom_material_layout", &[]);

    bind_group_allocators.insert(
        TypeId::of::<CustomMaterial>(),
        MaterialBindGroupAllocator::new(
            &render_device,
            "custom_material_allocator",
            None,
            bind_group_layout,
            None,
        ),
    );
}

impl ErasedRenderAsset for CustomMaterial {
    type SourceAsset = CustomMaterial;
    type ErasedAsset = PreparedMaterial;
    type Param = (
        SRes<DrawFunctions<Opaque3d>>,
        SRes<AssetServer>,
        SRes<RenderDevice>,
        SResMut<MaterialBindGroupAllocators>,
        SResMut<RenderMaterialBindings>,
        SRes<RenderAssets<GpuImage>>,
    );

    fn prepare_asset(
        source_asset: Self::SourceAsset,
        asset_id: AssetId<Self::SourceAsset>,
        (
            opaque_draw_functions,
            _asset_server,
            render_device,
            bind_group_allocators,
            render_material_bindings,
            gpu_images,
        ): &mut SystemParamItem<Self::Param>,
    ) -> std::result::Result<Self::ErasedAsset, PrepareAssetError<Self::SourceAsset>> {
        let reflection = source_asset.shader.reflection();

        let layout_entries = reflection.bind_group_layout(3);
        let bind_group_layout =
            BindGroupLayoutDescriptor::new("custom_material_bind_group", &layout_entries);

        let bindings =
            reflection.create_bindings(3, &source_asset.shader, render_device, gpu_images);

        let unprepared = UnpreparedBindGroup {
            bindings: BindingResources(bindings),
        };

        let bind_group_allocator = bind_group_allocators
            .get_mut(&TypeId::of::<CustomMaterial>())
            .unwrap();

        let binding = match render_material_bindings.entry(asset_id.into()) {
            Entry::Occupied(mut occupied_entry) => {
                bind_group_allocator.free(*occupied_entry.get());
                let new_binding =
                    bind_group_allocator.allocate_unprepared(unprepared, &bind_group_layout);
                *occupied_entry.get_mut() = new_binding;
                new_binding
            }
            Entry::Vacant(vacant_entry) => *vacant_entry
                .insert(bind_group_allocator.allocate_unprepared(unprepared, &bind_group_layout)),
        };

        let draw_function = opaque_draw_functions.read().id::<DrawMaterial>();

        let mut properties = MaterialProperties {
            mesh_pipeline_key_bits: ErasedMeshPipelineKey::new(MeshPipelineKey::empty()),
            base_specialize: Some(base_specialize),
            material_layout: Some(bind_group_layout),
            ..Default::default()
        };
        properties.add_draw_function(MainPassOpaqueDrawFunction, draw_function);
        if source_asset.has_vertex {
            properties.add_shader(MaterialVertexShader, source_asset.shader_handle.clone());
        }
        if source_asset.has_fragment {
            properties.add_shader(MaterialFragmentShader, source_asset.shader_handle.clone());
        }

        Ok(PreparedMaterial {
            binding,
            properties: Arc::new(properties),
        })
    }
}

fn extract_custom_materials(
    mut material_instances: ResMut<RenderMaterialInstances>,
    changed_query: Extract<
        Query<
            (Entity, &ViewVisibility, &CustomMaterial3d),
            Or<(Changed<ViewVisibility>, Changed<CustomMaterial3d>)>,
        >,
    >,
) {
    let last_change_tick = material_instances.current_change_tick;
    for (entity, view_visibility, material) in &changed_query {
        let vis = view_visibility.get();
        if vis {
            material_instances.instances.insert(
                entity.into(),
                RenderMaterialInstance {
                    asset_id: material.0.id().untyped(),
                    last_change_tick,
                },
            );
        } else {
            material_instances
                .instances
                .remove(&MainEntity::from(entity));
        }
    }
}

fn extract_custom_materials_needing_specialization(
    entities: Extract<Res<EntitiesNeedingSpecialization<CustomMaterial>>>,
    mut dirty: ResMut<DirtySpecializations>,
) {
    for entity in entities.changed.iter() {
        dirty.changed_renderables.insert(MainEntity::from(*entity));
    }
}

fn extract_custom_materials_that_need_specializations_removed(
    entities: Extract<Res<EntitiesNeedingSpecialization<CustomMaterial>>>,
    mut dirty: ResMut<DirtySpecializations>,
) {
    for entity in entities.removed.iter() {
        dirty.removed_renderables.insert(MainEntity::from(*entity));
    }
}

fn check_entities_needing_specialization(
    needs_specialization: Query<
        Entity,
        (
            Or<(
                Changed<Mesh3d>,
                AssetChanged<Mesh3d>,
                Changed<CustomMaterial3d>,
                AssetChanged<CustomMaterial3d>,
            )>,
            With<CustomMaterial3d>,
        ),
    >,
    mut entities: ResMut<EntitiesNeedingSpecialization<CustomMaterial>>,
    mut removed_mesh: RemovedComponents<Mesh3d>,
    mut removed_material: RemovedComponents<CustomMaterial3d>,
) {
    entities.changed.clear();
    entities.removed.clear();

    for entity in &needs_specialization {
        entities.changed.push(entity);
    }

    for entity in removed_mesh.read() {
        entities.removed.push(entity);
    }
    for entity in removed_material.read() {
        entities.removed.push(entity);
    }
}
