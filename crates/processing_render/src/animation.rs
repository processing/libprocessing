//! Animation pose pools: K paused puppet instances of a rigged glTF scene,
//! each seeked to k/K of one animation clip, giving K static skeleton poses
//! whose joint matrices bevy uploads every frame. Instanced particles sample
//! the pool by a per-particle phase attribute (see `particles/pack.rs`), so
//! thousands of instances animate at individual phases with no per-instance
//! animation state — poses are a pure function of the phase attribute, which
//! keeps frame-clock determinism.

use bevy::animation::{AnimationClip, AnimationPlayer, graph::AnimationGraphHandle};
use bevy::camera::visibility::{SetViewVisibility, ViewVisibility, VisibilitySystems};
use bevy::gltf::Gltf;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceSpawner;

use processing_core::error::{ProcessingError, Result};

pub struct AnimationPoolPlugin;

impl Plugin for AnimationPoolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            force_puppet_visibility.in_set(VisibilitySystems::CheckVisibility),
        );
    }
}

/// A pose pool: `slots[k]` is the skinned-mesh entity of the puppet posed at
/// phase k/K. The render-side pack pipeline reads each slot's live skin
/// index out of `SkinUniforms` every frame (offsets are not stable).
#[derive(Component)]
pub struct SkinPosePool {
    pub slots: Vec<Entity>,
    instances: Vec<bevy::world_serialization::InstanceId>,
}

/// Marker on puppet skinned-mesh entities. Puppets are `Visibility::Hidden`
/// (never queued for rendering by any view), but skin extraction only
/// allocates joint matrices for view-visible entities — this marker forces
/// their `ViewVisibility` through bevy's sanctioned custom-visibility hook so
/// `SkinUniforms` stays populated.
#[derive(Component)]
pub struct SkinPoolPuppet;

fn force_puppet_visibility(mut puppets: Query<&mut ViewVisibility, With<SkinPoolPuppet>>) {
    for mut view_visibility in &mut puppets {
        view_visibility.set_visible();
    }
}

/// Spawn K paused puppets of `gltf_entity`'s scene, posed along `clip_name`.
pub fn pool_create(
    In((gltf_entity, clip_name, k)): In<(Entity, String, u32)>,
    world: &mut World,
) -> Result<Entity> {
    if k == 0 {
        return Err(ProcessingError::InvalidArgument(
            "animation pool size must be at least 1".to_string(),
        ));
    }
    let handle = world
        .get::<crate::gltf::GltfHandle>(gltf_entity)
        .ok_or(ProcessingError::InvalidEntity)?
        .gltf_handle()
        .clone();

    let (scene_handle, clip_handle) = {
        let gltf_assets = world.resource::<Assets<Gltf>>();
        let gltf = gltf_assets
            .get(&handle)
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF asset not found".into()))?;
        let scene = gltf
            .default_scene
            .clone()
            .or_else(|| gltf.scenes.first().cloned())
            .ok_or_else(|| ProcessingError::GltfLoadError("GLTF has no scenes".into()))?;
        let clip = gltf.named_animations.get(clip_name.as_str()).cloned().ok_or_else(|| {
            ProcessingError::GltfLoadError(format!(
                "Animation '{clip_name}' not found in GLTF (available: {:?})",
                gltf.named_animations.keys().collect::<Vec<_>>()
            ))
        })?;
        (scene, clip)
    };

    let duration = world
        .resource::<Assets<AnimationClip>>()
        .get(&clip_handle)
        .ok_or_else(|| ProcessingError::GltfLoadError("Animation clip asset not found".into()))?
        .duration();

    let (graph, clip_node) = bevy::animation::graph::AnimationGraph::from_clip(clip_handle);
    let graph_handle = world.resource_mut::<Assets<bevy::animation::graph::AnimationGraph>>().add(graph);

    let mut slots = Vec::with_capacity(k as usize);
    let mut instances = Vec::with_capacity(k as usize);
    for pose in 0..k {
        let seek_time = duration * (pose as f32 / k as f32);

        let instance_id = world.resource_scope(|world, mut spawner: Mut<WorldInstanceSpawner>| {
            spawner
                .spawn_sync(world, &scene_handle)
                .map_err(|e| ProcessingError::GltfLoadError(format!("Puppet spawn failed: {e}")))
        })?;
        instances.push(instance_id);
        let entities: Vec<Entity> = {
            let spawner = world.resource::<WorldInstanceSpawner>();
            spawner.iter_instance_entities(instance_id).collect()
        };

        let mut slot_entity = None;
        for entity in entities {
            // Same hygiene as gltf::load: scene cameras must not fight the
            // sketch's camera.
            if world.get::<Camera>(entity).is_some() {
                world.entity_mut(entity).remove::<Camera>();
            }
            // Hidden everywhere: puppets exist only to pose joint matrices.
            if let Some(mut visibility) = world.get_mut::<Visibility>(entity) {
                *visibility = Visibility::Hidden;
            }
            if world.get::<SkinnedMesh>(entity).is_some() {
                world.entity_mut(entity).insert(SkinPoolPuppet);
                if slot_entity.is_none() {
                    slot_entity = Some(entity);
                }
            }
            if world.get::<AnimationPlayer>(entity).is_some() {
                world
                    .entity_mut(entity)
                    .insert(AnimationGraphHandle(graph_handle.clone()));
                let mut player = world.get_mut::<AnimationPlayer>(entity).unwrap();
                player.play(clip_node).seek_to(seek_time).pause();
            }
        }

        slots.push(slot_entity.ok_or_else(|| {
            ProcessingError::GltfLoadError(
                "GLTF scene has no skinned mesh; an animation pool needs a rigged mesh"
                    .to_string(),
            )
        })?);
    }

    Ok(world.spawn(SkinPosePool { slots, instances }).id())
}

/// Destroy a pose pool and all K puppet scene instances it spawned.
pub fn pool_destroy(In(pool_entity): In<Entity>, world: &mut World) -> Result<()> {
    let instances = world
        .get::<SkinPosePool>(pool_entity)
        .ok_or(ProcessingError::InvalidEntity)?
        .instances
        .clone();
    world.resource_scope(|world, mut spawner: Mut<WorldInstanceSpawner>| {
        for instance_id in &instances {
            spawner.despawn_instance_sync(world, instance_id);
        }
    });
    world.despawn(pool_entity);
    Ok(())
}
