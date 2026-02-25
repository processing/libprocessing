//! Load and query GLTF files, providing name-based lookup for meshes,
//! materials, cameras, and lights.

use bevy::{
    asset::{
        AssetPath, LoadState, handle_internal_asset_events,
        io::{AssetSourceId, embedded::GetAssetServer},
    },
    ecs::system::RunSystemOnce,
    gltf::{Gltf, GltfLoaderSettings, GltfMesh},
    prelude::*,
};

use crate::config::{Config, ConfigKey};
use crate::error::{ProcessingError, Result};
use crate::geometry::{BuiltinAttributes, Geometry, layout::VertexLayout};
use crate::graphics;
use crate::light;
use crate::render::material::UntypedMaterial;

fn resolve_asset_path(config: &Config, path: &str) -> AssetPath<'static> {
    let asset_path = AssetPath::parse(path).into_owned();
    match config.get(ConfigKey::AssetRootPath) {
        Some(_) => asset_path.with_source(AssetSourceId::from("assets_directory")),
        None => asset_path,
    }
}

/// Block until an asset handle loads, returning an error if loading fails.
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

/// Load the root GLTF asset, blocking until fully loaded.
fn load_gltf_root(world: &mut World, config: &Config, path: &str) -> Result<Handle<Gltf>> {
    let base_path = match path.find('#') {
        Some(idx) => &path[..idx],
        None => path,
    };
    let asset_path = resolve_asset_path(config, base_path);
    let handle: Handle<Gltf> =
        world
            .get_asset_server()
            .load_with_settings(asset_path, |s: &mut GltfLoaderSettings| {
                s.include_source = true;
            });
    block_on_load(world, |w| w.get_asset_server().load_state(&handle))?;
    Ok(handle)
}

#[derive(Component)]
pub struct GltfHandle {
    handle: Handle<Gltf>,
    base_path: String,
}

pub fn load(In(path): In<String>, world: &mut World) -> Result<Entity> {
    let config = world.resource::<Config>().clone();
    let handle = load_gltf_root(world, &config, &path)?;
    let base_path = match path.find('#') {
        Some(idx) => path[..idx].to_string(),
        None => path,
    };
    let entity = world.spawn(GltfHandle { handle, base_path }).id();
    Ok(entity)
}

pub fn geometry(
    In((gltf_entity, name)): In<(Entity, String)>,
    world: &mut World,
) -> Result<Entity> {
    let handle = world
        .get::<GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?;
    let gltf_handle = handle.handle.clone();

    let mesh_handle = {
        let gltf_assets = world.resource::<Assets<Gltf>>();
        let gltf = gltf_assets
            .get(&gltf_handle)
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF asset not found".into()))?;
        let gltf_mesh_handle = gltf.named_meshes.get(name.as_str()).ok_or_else(|| {
            ProcessingError::GltfLoadError(format!("Mesh '{}' not found in GLTF", name))
        })?;
        let gltf_mesh_assets = world.resource::<Assets<GltfMesh>>();
        let gltf_mesh = gltf_mesh_assets
            .get(gltf_mesh_handle)
            .ok_or_else(|| ProcessingError::GltfLoadError("GltfMesh asset not found".into()))?;
        // TODO: a mesh could have multiple primitives, but for simplicity we'll just take the
        // first one here. we could extend the API later to allow users to specify a primitive index
        // if needed, or support mesh hierachies via parent/child ptrs on the python classes.
        let prim = gltf_mesh.primitives.first().ok_or_else(|| {
            ProcessingError::GltfLoadError(format!("Mesh '{}' has no primitives", name))
        })?;
        prim.mesh.clone()
    };

    let builtins = world.resource::<BuiltinAttributes>();
    let attrs = vec![
        builtins.position,
        builtins.normal,
        builtins.color,
        builtins.uv,
    ];
    let layout_entity = world.spawn(VertexLayout::with_attributes(attrs)).id();
    let entity = world.spawn(Geometry::new(mesh_handle, layout_entity)).id();
    Ok(entity)
}

pub fn material(
    In((gltf_entity, name)): In<(Entity, String)>,
    world: &mut World,
) -> Result<Entity> {
    let handle = world
        .get::<GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?;
    let gltf_handle = handle.handle.clone();
    let base_path = handle.base_path.clone();

    let material_index = {
        let gltf_assets = world.resource::<Assets<Gltf>>();
        let gltf = gltf_assets
            .get(&gltf_handle)
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF asset not found".into()))?;
        let named_handle = gltf.named_materials.get(name.as_str()).ok_or_else(|| {
            ProcessingError::GltfLoadError(format!("Material '{}' not found in GLTF", name))
        })?;
        gltf.materials
            .iter()
            .position(|h| h.id() == named_handle.id())
            .ok_or_else(|| {
                ProcessingError::GltfLoadError(format!(
                    "Material '{}' not found in materials list",
                    name
                ))
            })?
    };

    let config = world.resource::<Config>().clone();
    // this is a bit hacky but we can leverage the fact that the GLTF loader creates standard
    // material assets with predictable labels based on the GLTF material index. we just need to
    // construct the correct path to look up the asset handle, then we can spawn an entity with an
    // UntypedMaterial component referencing that handle.
    let std_path = format!("{}#Material{}/std", base_path, material_index);
    let asset_path = resolve_asset_path(&config, &std_path);
    let handle: Handle<StandardMaterial> = world.get_asset_server().load(asset_path);
    block_on_load(world, |w| w.get_asset_server().load_state(&handle))?;
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

fn node_transform(node: &gltf::Node) -> Transform {
    let (translation, rotation, scale) = node.transform().decomposed();
    Transform {
        translation: Vec3::from(translation),
        rotation: Quat::from_array(rotation),
        scale: Vec3::from(scale),
    }
}

fn find_node_transform(
    source: &gltf::Gltf,
    predicate: impl Fn(&gltf::Node) -> bool,
) -> Option<Transform> {
    fn walk(node: gltf::Node, predicate: &impl Fn(&gltf::Node) -> bool) -> Option<Transform> {
        if predicate(&node) {
            return Some(node_transform(&node));
        }
        for child in node.children() {
            if let Some(t) = walk(child, predicate) {
                return Some(t);
            }
        }
        None
    }

    for scene in source.scenes() {
        for node in scene.nodes() {
            if let Some(t) = walk(node, &predicate) {
                return Some(t);
            }
        }
    }
    None
}

enum CameraProjection {
    Perspective {
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        xmag: f32,
        ymag: f32,
        near: f32,
        far: f32,
    },
}

pub fn camera(
    In((gltf_entity, graphics_entity, index)): In<(Entity, Entity, usize)>,
    world: &mut World,
) -> Result<()> {
    let handle = world
        .get::<GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?;
    let gltf_handle = handle.handle.clone();

    let (projection, node_xform) = {
        let gltf_assets = world.resource::<Assets<Gltf>>();
        let gltf = gltf_assets
            .get(&gltf_handle)
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF asset not found".into()))?;
        let source = gltf
            .source
            .as_ref()
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF source not loaded".into()))?;

        let gltf_camera = source.cameras().nth(index).ok_or_else(|| {
            ProcessingError::GltfLoadError(format!("Camera index {} not found", index))
        })?;

        let projection = match gltf_camera.projection() {
            gltf::camera::Projection::Perspective(p) => CameraProjection::Perspective {
                fov: p.yfov(),
                aspect_ratio: p.aspect_ratio().unwrap_or(1.0),
                near: p.znear(),
                far: p.zfar().unwrap_or(10_000.0),
            },
            gltf::camera::Projection::Orthographic(o) => CameraProjection::Orthographic {
                xmag: o.xmag(),
                ymag: o.ymag(),
                near: o.znear(),
                far: o.zfar(),
            },
        };

        let node_xform =
            find_node_transform(source, |n| n.camera().map(|c| c.index()) == Some(index));

        (projection, node_xform)
    };

    match projection {
        CameraProjection::Perspective {
            fov,
            aspect_ratio,
            near,
            far,
        } => {
            world
                .run_system_cached_with(
                    graphics::perspective,
                    (
                        graphics_entity,
                        PerspectiveProjection {
                            fov,
                            aspect_ratio,
                            near,
                            far,
                            near_clip_plane: Vec4::new(0.0, 0.0, -1.0, -near),
                        },
                    ),
                )
                .unwrap()?;
        }
        CameraProjection::Orthographic {
            xmag,
            ymag,
            near,
            far,
        } => {
            world
                .run_system_cached_with(
                    graphics::ortho,
                    (
                        graphics_entity,
                        graphics::OrthoArgs {
                            left: -xmag,
                            right: xmag,
                            bottom: -ymag,
                            top: ymag,
                            near,
                            far,
                        },
                    ),
                )
                .unwrap()?;
        }
    }

    if let Some(t) = node_xform {
        let mut transform = world
            .get_mut::<Transform>(graphics_entity)
            .ok_or(ProcessingError::GraphicsNotFound)?;
        *transform = t;
    }

    Ok(())
}

pub fn light(
    In((gltf_entity, graphics_entity, index)): In<(Entity, Entity, usize)>,
    world: &mut World,
) -> Result<Entity> {
    let handle = world
        .get::<GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?;
    let gltf_handle = handle.handle.clone();

    let (light_entity, node_xform) = {
        let gltf_assets = world.resource::<Assets<Gltf>>();
        let gltf = gltf_assets
            .get(&gltf_handle)
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF asset not found".into()))?;
        let source = gltf
            .source
            .as_ref()
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF source not loaded".into()))?;

        let gltf_light = source
            .lights()
            .and_then(|mut lights| lights.nth(index))
            .ok_or_else(|| {
                ProcessingError::GltfLoadError(format!("Light index {} not found", index))
            })?;

        let color = Color::srgb_from_array(gltf_light.color());
        let node_xform =
            find_node_transform(source, |n| n.light().map(|l| l.index()) == Some(index));

        let light_entity = match gltf_light.kind() {
            gltf::khr_lights_punctual::Kind::Directional => world
                .run_system_cached_with(
                    light::create_directional,
                    (graphics_entity, color, gltf_light.intensity()),
                )
                .unwrap()?,
            gltf::khr_lights_punctual::Kind::Point => world
                .run_system_cached_with(
                    light::create_point,
                    (
                        graphics_entity,
                        color,
                        gltf_light.intensity() * core::f32::consts::PI * 4.0,
                        gltf_light.range().unwrap_or(20.0),
                        0.0,
                    ),
                )
                .unwrap()?,
            gltf::khr_lights_punctual::Kind::Spot {
                inner_cone_angle,
                outer_cone_angle,
            } => world
                .run_system_cached_with(
                    light::create_spot,
                    (
                        graphics_entity,
                        color,
                        gltf_light.intensity() * core::f32::consts::PI * 4.0,
                        gltf_light.range().unwrap_or(20.0),
                        0.0,
                        inner_cone_angle,
                        outer_cone_angle,
                    ),
                )
                .unwrap()?,
        };

        (light_entity, node_xform)
    };

    if let Some(t) = node_xform {
        let mut transform = world
            .get_mut::<Transform>(light_entity)
            .ok_or(ProcessingError::TransformNotFound)?;
        *transform = t;
    }

    Ok(light_entity)
}
