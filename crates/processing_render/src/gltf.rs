//! Load and query GLTF files, providing name-based lookup for meshes,
//! materials, cameras, and lights.

use bevy::{
    asset::{
        AssetPath, LoadState, handle_internal_asset_events,
        io::{AssetSourceId, embedded::GetAssetServer},
    },
    camera::visibility::RenderLayers,
    ecs::system::RunSystemOnce,
    gltf::{Gltf, GltfMaterial, GltfMeshName},
    pbr::ExtendedMaterial,
    prelude::*,
    world_serialization::WorldInstanceSpawner,
};

use crate::geometry::{BuiltinAttributes, Geometry, layout::VertexLayout};
use crate::graphics;
use crate::material::ProcessingMaterial;
use crate::render::material::{ProcessingExtendedMaterial, UntypedMaterial};
use processing_core::config::{Config, ConfigKey};
use processing_core::error::{ProcessingError, Result};

#[derive(Component)]
pub struct GltfNodeTransform(pub Transform);

fn resolve_asset_path(config: &Config, path: &str) -> AssetPath<'static> {
    let asset_path = AssetPath::parse(path).into_owned();
    match config.get(ConfigKey::AssetRootPath) {
        Some(_) => asset_path.with_source(AssetSourceId::from("assets_directory")),
        None => asset_path,
    }
}

fn block_on_load(world: &mut World, load_state: impl Fn(&World) -> LoadState) -> Result<()> {
    loop {
        match load_state(world) {
            LoadState::Loading => {
                world.run_system_once(handle_internal_asset_events).unwrap();
            }
            LoadState::Loaded => return Ok(()),
            LoadState::Failed(err) => {
                return Err(ProcessingError::GltfLoadError(format!(
                    "Asset failed to load: {err}"
                )));
            }
            LoadState::NotLoaded => {
                return Err(ProcessingError::GltfLoadError(
                    "Asset not loaded".to_string(),
                ));
            }
        }
    }
}

fn compute_global_transform(world: &World, entity: Entity) -> Transform {
    let local = world.get::<Transform>(entity).copied().unwrap_or_default();
    match world.get::<ChildOf>(entity) {
        Some(child_of) => {
            let parent_global = compute_global_transform(world, child_of.parent());
            Transform::from_matrix(parent_global.to_matrix() * local.to_matrix())
        }
        None => local,
    }
}

#[derive(Component)]
pub struct GltfHandle {
    handle: Handle<Gltf>,
    instance_id: bevy::world_serialization::InstanceId,
    graphics_entity: Entity,
}

pub fn load(
    In((graphics_entity, path)): In<(Entity, String)>,
    world: &mut World,
) -> Result<Entity> {
    let config = world.resource::<Config>().clone();
    let base_path = match path.find('#') {
        Some(idx) => &path[..idx],
        None => path.as_str(),
    };
    let asset_path = resolve_asset_path(&config, base_path);
    let handle: Handle<Gltf> = world.get_asset_server().load(asset_path);
    block_on_load(world, |w| w.get_asset_server().load_state(&handle))?;

    let scene_handle = {
        let gltf_assets = world.resource::<Assets<Gltf>>();
        let gltf = gltf_assets
            .get(&handle)
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF asset not found".into()))?;
        gltf.default_scene
            .clone()
            .or_else(|| gltf.scenes.first().cloned())
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF has no scenes".into()))?
    };

    // we spawn the scene in to the world in a blocking fashion so that bevy runs all
    // its hooks for the gltf, ex creating standard material instances
    let instance_id = world.resource_scope(|world, mut spawner: Mut<WorldInstanceSpawner>| {
        spawner
            .spawn_sync(world, &scene_handle)
            .map_err(|e| ProcessingError::GltfLoadError(format!("Scene spawn failed: {e}")))
    })?;

    // we have to remove the existing cameras from the scene -- the user can request to set *this*
    // graphics to a camera, but the scenes cameras should not exist
    {
        let spawner = world.resource::<WorldInstanceSpawner>();
        let cam_entities: Vec<Entity> = spawner
            .iter_instance_entities(instance_id)
            .filter(|&e| world.get::<Camera>(e).is_some())
            .collect();
        for e in cam_entities {
            // gltf is weird -- cameras can exist on any node. we remove just the camera component rather
            // than despawn in order to be safe
            world.entity_mut(e).remove::<Camera>();
        }
    }

    let entity = world
        .spawn(GltfHandle {
            handle,
            instance_id,
            graphics_entity,
        })
        .id();
    Ok(entity)
}

pub fn geometry(
    In((gltf_entity, name)): In<(Entity, String)>,
    world: &mut World,
) -> Result<Entity> {
    let gltf_handle = world
        .get::<GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?;
    let instance_id = gltf_handle.instance_id;

    let (mesh_handle, global_transform) = {
        let spawner = world.resource::<WorldInstanceSpawner>();

        // find the mesh with the given name component that bevy added post-spawn
        // name is derived from gltf node or computed
        let mesh_entity = spawner
            .iter_instance_entities(instance_id)
            .find(|&e| {
                world
                    .get::<GltfMeshName>(e)
                    .map(|n| n.0 == name)
                    .unwrap_or(false)
            })
            .ok_or_else(|| {
                ProcessingError::GltfLoadError(format!("Mesh '{}' not found in GLTF scene", name))
            })?;

        let mesh3d = world.get::<Mesh3d>(mesh_entity).ok_or_else(|| {
            ProcessingError::GltfLoadError(format!(
                "Mesh '{}' scene entity has no Mesh3d component",
                name
            ))
        })?;
        let handle = mesh3d.0.clone();
        let transform = compute_global_transform(world, mesh_entity);
        (handle, transform)
    };

    let builtins = world.resource::<BuiltinAttributes>();
    let attrs = vec![
        builtins.position,
        builtins.normal,
        builtins.color,
        builtins.uv,
    ];
    let layout_entity = world.spawn(VertexLayout::with_attributes(attrs)).id();
    let entity = world
        .spawn((
            Geometry::new(mesh_handle, layout_entity),
            GltfNodeTransform(global_transform),
        ))
        .id();
    Ok(entity)
}

/// Translate a bevy [`GltfMaterial`] into the [`StandardMaterial`] base of a
/// [`ProcessingExtendedMaterial`]. `GltfMaterial` mirrors `StandardMaterial`'s
/// fields, so we copy across the ones that drive shading and let the rest fall
/// back to defaults.
fn gltf_material_to_standard(m: &GltfMaterial) -> StandardMaterial {
    StandardMaterial {
        base_color: m.base_color,
        base_color_channel: m.base_color_channel.clone(),
        base_color_texture: m.base_color_texture.clone(),
        emissive: m.emissive,
        emissive_channel: m.emissive_channel.clone(),
        emissive_texture: m.emissive_texture.clone(),
        perceptual_roughness: m.perceptual_roughness,
        metallic: m.metallic,
        metallic_roughness_channel: m.metallic_roughness_channel.clone(),
        metallic_roughness_texture: m.metallic_roughness_texture.clone(),
        normal_map_channel: m.normal_map_channel.clone(),
        normal_map_texture: m.normal_map_texture.clone(),
        occlusion_channel: m.occlusion_channel.clone(),
        occlusion_texture: m.occlusion_texture.clone(),
        double_sided: m.double_sided,
        cull_mode: m.cull_mode,
        unlit: m.unlit,
        alpha_mode: m.alpha_mode,
        uv_transform: m.uv_transform,
        ..default()
    }
}

pub fn material(
    In((gltf_entity, name)): In<(Entity, String)>,
    world: &mut World,
) -> Result<Entity> {
    let gltf_handle = world
        .get::<GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?
        .handle
        .clone();

    let standard = {
        let gltf_assets = world.resource::<Assets<Gltf>>();
        let gltf = gltf_assets
            .get(&gltf_handle)
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF asset not found".into()))?;
        let mat_handle = gltf.named_materials.get(name.as_str()).ok_or_else(|| {
            ProcessingError::GltfLoadError(format!("Material '{}' not found in GLTF", name))
        })?;

        let gltf_materials = world.resource::<Assets<GltfMaterial>>();
        let gltf_material = gltf_materials.get(mat_handle).ok_or_else(|| {
            ProcessingError::GltfLoadError(format!("Material '{}' asset not loaded", name))
        })?;
        gltf_material_to_standard(gltf_material)
    };

    // wrap in the extended material the processing renderer's pipeline expects, so
    // the draw systems attach a `MeshMaterial3d` and `material_set` can mutate it
    let handle = world
        .resource_mut::<Assets<ProcessingExtendedMaterial>>()
        .add(ExtendedMaterial {
            base: standard,
            extension: ProcessingMaterial { blend_state: None },
        });
    let entity = world.spawn(UntypedMaterial(handle.untyped())).id();
    Ok(entity)
}

pub fn mesh_names(In(gltf_entity): In<Entity>, world: &mut World) -> Result<Vec<String>> {
    let handle = world
        .get::<GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?;
    let gltf_handle = handle.handle.clone();

    let gltf_assets = world.resource::<Assets<Gltf>>();
    let gltf = gltf_assets
        .get(&gltf_handle)
        .ok_or_else(|| ProcessingError::GltfLoadError("GLTF asset not found".into()))?;
    Ok(gltf.named_meshes.keys().map(|k| k.to_string()).collect())
}

pub fn material_names(In(gltf_entity): In<Entity>, world: &mut World) -> Result<Vec<String>> {
    let handle = world
        .get::<GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?;
    let gltf_handle = handle.handle.clone();

    let gltf_assets = world.resource::<Assets<Gltf>>();
    let gltf = gltf_assets
        .get(&gltf_handle)
        .ok_or_else(|| ProcessingError::GltfLoadError("GLTF asset not found".into()))?;
    Ok(gltf.named_materials.keys().map(|k| k.to_string()).collect())
}

pub fn camera(In((gltf_entity, index)): In<(Entity, usize)>, world: &mut World) -> Result<()> {
    let gltf_handle = world
        .get::<GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?;
    let instance_id = gltf_handle.instance_id;
    let graphics_entity = gltf_handle.graphics_entity;

    let (projection, node_xform) = {
        let spawner = world.resource::<WorldInstanceSpawner>();
        let camera_entity = spawner
            .iter_instance_entities(instance_id)
            .filter(|&e| world.get::<Camera3d>(e).is_some())
            .nth(index)
            .ok_or_else(|| {
                ProcessingError::GltfLoadError(format!("Camera index {} not found", index))
            })?;

        let projection = world
            .get::<Projection>(camera_entity)
            .ok_or_else(|| {
                ProcessingError::GltfLoadError("Camera entity has no Projection component".into())
            })?
            .clone();
        let transform = compute_global_transform(world, camera_entity);
        (projection, transform)
    };

    match projection {
        Projection::Perspective(p) => {
            world
                .run_system_cached_with(graphics::perspective, (graphics_entity, p))
                .unwrap()?;
        }
        Projection::Orthographic(o) => {
            world
                .run_system_cached_with(
                    graphics::ortho,
                    (
                        graphics_entity,
                        graphics::OrthoArgs {
                            left: o.area.min.x,
                            right: o.area.max.x,
                            bottom: o.area.min.y,
                            top: o.area.max.y,
                            near: o.near,
                            far: o.far,
                        },
                    ),
                )
                .unwrap()?;
        }
        Projection::Custom(_) => {
            return Err(ProcessingError::GltfLoadError(
                "Custom projections are not supported".into(),
            ));
        }
    }

    let mut transform = world
        .get_mut::<Transform>(graphics_entity)
        .ok_or(ProcessingError::GraphicsNotFound)?;
    *transform = node_xform;

    Ok(())
}

pub fn light(In((gltf_entity, index)): In<(Entity, usize)>, world: &mut World) -> Result<Entity> {
    let gltf_handle = world
        .get::<GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?;
    let instance_id = gltf_handle.instance_id;
    let graphics_entity = gltf_handle.graphics_entity;

    let light_entities: Vec<Entity> = {
        let spawner = world.resource::<WorldInstanceSpawner>();
        spawner
            .iter_instance_entities(instance_id)
            .filter(|&e| {
                world.get::<DirectionalLight>(e).is_some()
                    || world.get::<PointLight>(e).is_some()
                    || world.get::<SpotLight>(e).is_some()
            })
            .collect()
    };

    let scene_light_entity = *light_entities.get(index).ok_or_else(|| {
        ProcessingError::GltfLoadError(format!("Light index {} not found", index))
    })?;

    let render_layers = world
        .get::<RenderLayers>(graphics_entity)
        .ok_or(ProcessingError::GraphicsNotFound)?
        .clone();
    world.entity_mut(scene_light_entity).insert(render_layers);

    let global = compute_global_transform(world, scene_light_entity);
    *world
        .get_mut::<Transform>(scene_light_entity)
        .ok_or(ProcessingError::GraphicsNotFound)? = global;

    Ok(scene_light_entity)
}
