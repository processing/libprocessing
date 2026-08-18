use bevy::asset::AssetPath;
use bevy::asset::io::AssetSourceId;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

pub use nannou_video::{
    HwAccelPolicy, NannouVideoPlugin, NetworkPreset, PlaybackMode, SeekTo, Video,
    VideoAssetLoaderError, VideoEnded, VideoFailed, VideoLoaded, VideoLoaderSettings, VideoLooped,
    VideoOutput, VideoPlayer, VideoResize, VideoSeeked, VideoSource,
};

mod record;
pub use record::{
    RecorderCodec, RecorderPixelFormat, VideoRecordError, VideoRecorder, VideoRecorderConfig,
};

use processing_core::app_mut;
use processing_core::error::{ProcessingError, Result};
use processing_render::image;

#[derive(Component)]
pub struct ProcessingVideo;

fn create(In(handle): In<Handle<Video>>, mut commands: Commands) -> Entity {
    commands
        .spawn((ProcessingVideo, VideoPlayer::new(handle)))
        .id()
}

fn create_with_mode(
    In((handle, mode)): In<(Handle<Video>, PlaybackMode)>,
    mut commands: Commands,
) -> Entity {
    commands
        .spawn((ProcessingVideo, VideoPlayer::new(handle).with_mode(mode)))
        .id()
}

fn create_image(In(entity): In<Entity>, world: &mut World) -> Result<Entity> {
    if let Some(linked) = world.get::<image::LinkedImage>(entity) {
        return Ok(linked.0);
    }

    let output = world
        .get::<VideoOutput>(entity)
        .ok_or(ProcessingError::VideoNotLoaded)?;
    let handle = output.image.clone();

    let child = world
        .run_system_once_with(image::from_handle, handle)
        .unwrap()?;
    world.entity_mut(entity).insert(image::LinkedImage(child));
    world.entity_mut(entity).add_child(child);
    Ok(child)
}

fn is_loaded(In(entity): In<Entity>, outputs: Query<&VideoOutput>) -> Result<bool> {
    Ok(outputs.get(entity).is_ok())
}

fn resolution(In(entity): In<Entity>, outputs: Query<&VideoOutput>) -> Result<(u32, u32)> {
    let output = outputs
        .get(entity)
        .map_err(|_| ProcessingError::VideoNotLoaded)?;
    Ok((output.size.x, output.size.y))
}

fn position(In(entity): In<Entity>, outputs: Query<&VideoOutput>) -> Result<f64> {
    let output = outputs
        .get(entity)
        .map_err(|_| ProcessingError::VideoNotLoaded)?;
    Ok(output.position_seconds)
}

fn seek(In((entity, seconds)): In<(Entity, f64)>, mut commands: Commands) -> Result<()> {
    commands.entity(entity).insert(SeekTo(seconds));
    Ok(())
}

fn set_paused(
    In((entity, paused)): In<(Entity, bool)>,
    mut players: Query<&mut VideoPlayer>,
) -> Result<()> {
    let mut player = players
        .get_mut(entity)
        .map_err(|_| ProcessingError::VideoNotLoaded)?;
    player.paused = paused;
    Ok(())
}

fn set_speed(
    In((entity, speed)): In<(Entity, f32)>,
    mut players: Query<&mut VideoPlayer>,
) -> Result<()> {
    let mut player = players
        .get_mut(entity)
        .map_err(|_| ProcessingError::VideoNotLoaded)?;
    player.speed = speed;
    Ok(())
}

fn set_mode(
    In((entity, mode)): In<(Entity, PlaybackMode)>,
    mut players: Query<&mut VideoPlayer>,
) -> Result<()> {
    let mut player = players
        .get_mut(entity)
        .map_err(|_| ProcessingError::VideoNotLoaded)?;
    player.mode = mode;
    Ok(())
}

fn destroy(In(entity): In<Entity>, mut commands: Commands) -> Result<()> {
    commands.entity(entity).despawn();
    Ok(())
}

pub fn video_load(path: &str) -> Result<Handle<Video>> {
    app_mut(|app| {
        let config = app.world().resource::<processing_core::config::Config>();
        let asset_path: AssetPath =
            match config.get(processing_core::config::ConfigKey::AssetRootPath) {
                Some(_) => AssetPath::from_path_buf(path.into())
                    .with_source(AssetSourceId::from("assets_directory")),
                None => AssetPath::from_path_buf(path.into()),
            };
        let asset_server = app.world().resource::<AssetServer>();
        Ok(asset_server.load(asset_path))
    })
}

pub fn video_create(handle: Handle<Video>) -> Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(create, handle)
            .unwrap())
    })
}

pub fn video_create_with_mode(handle: Handle<Video>, mode: PlaybackMode) -> Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(create_with_mode, (handle, mode))
            .unwrap())
    })
}

pub fn video_is_loaded(entity: Entity) -> Result<bool> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(is_loaded, entity)
            .unwrap()
    })
}

pub fn video_image(entity: Entity) -> Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(create_image, entity)
            .unwrap()
    })
}

pub fn video_resolution(entity: Entity) -> Result<(u32, u32)> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(resolution, entity)
            .unwrap()
    })
}

pub fn video_position(entity: Entity) -> Result<f64> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(position, entity)
            .unwrap()
    })
}

pub fn video_seek(entity: Entity, seconds: f64) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(seek, (entity, seconds))
            .unwrap()
    })
}

pub fn video_set_paused(entity: Entity, paused: bool) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(set_paused, (entity, paused))
            .unwrap()
    })
}

pub fn video_set_speed(entity: Entity, speed: f32) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(set_speed, (entity, speed))
            .unwrap()
    })
}

pub fn video_set_mode(entity: Entity, mode: PlaybackMode) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(set_mode, (entity, mode))
            .unwrap()
    })
}

pub fn video_destroy(entity: Entity) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(destroy, entity)
            .unwrap()
    })
}
