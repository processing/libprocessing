use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

pub use nannou_webcam::{
    Webcam, WebcamConnected, WebcamDevice, WebcamDeviceAdded, WebcamDeviceRemoved,
    WebcamDisconnected, WebcamError, WebcamFormat, WebcamPlugin, WebcamStream,
    WebcamSupportedFormat,
};

use processing_core::app_mut;
use processing_core::error::{ProcessingError, Result};
use processing_render::image;

#[derive(Component)]
pub struct ProcessingWebcam;

fn create(In(()): In<()>, mut commands: Commands) -> Entity {
    commands.spawn((ProcessingWebcam, Webcam::default())).id()
}

fn create_with_format(In(format): In<WebcamFormat>, mut commands: Commands) -> Entity {
    commands
        .spawn((
            ProcessingWebcam,
            Webcam {
                format,
                ..default()
            },
        ))
        .id()
}

fn create_image(In(entity): In<Entity>, world: &mut World) -> Result<Entity> {
    let stream = world
        .get::<WebcamStream>(entity)
        .ok_or(ProcessingError::WebcamNotConnected)?;
    let handle = stream.image.clone();

    let child = world
        .run_system_once_with(image::from_handle, handle)
        .unwrap()?;
    world.entity_mut(entity).add_child(child);
    Ok(child)
}

fn is_connected(In(entity): In<Entity>, streams: Query<&WebcamStream>) -> Result<bool> {
    Ok(streams.get(entity).is_ok())
}

fn resolution(In(entity): In<Entity>, streams: Query<&WebcamStream>) -> Result<(u32, u32)> {
    let stream = streams
        .get(entity)
        .map_err(|_| ProcessingError::WebcamNotConnected)?;
    Ok((stream.resolution.x, stream.resolution.y))
}

fn destroy(In(entity): In<Entity>, mut commands: Commands) -> Result<()> {
    commands.entity(entity).despawn();
    Ok(())
}

pub fn webcam_create() -> Result<Entity> {
    app_mut(|app| Ok(app.world_mut().run_system_cached_with(create, ()).unwrap()))
}

pub fn webcam_create_with_format(format: WebcamFormat) -> Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world_mut()
            .run_system_cached_with(create_with_format, format)
            .unwrap())
    })
}

pub fn webcam_is_connected(entity: Entity) -> Result<bool> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(is_connected, entity)
            .unwrap()
    })
}

pub fn webcam_image(entity: Entity) -> Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(create_image, entity)
            .unwrap()
    })
}

pub fn webcam_resolution(entity: Entity) -> Result<(u32, u32)> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(resolution, entity)
            .unwrap()
    })
}

pub fn webcam_destroy(entity: Entity) -> Result<()> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(destroy, entity)
            .unwrap()
    })
}
