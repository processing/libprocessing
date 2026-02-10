use bevy::prelude::*;

use crate::error::{ProcessingError, Result};

pub fn set_position(
    In((entity, x, y, z)): In<(Entity, f32, f32, f32)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.translation = Vec3::new(x, y, z);
    Ok(())
}

pub fn translate(
    In((entity, x, y, z)): In<(Entity, f32, f32, f32)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.translation += Vec3::new(x, y, z);
    Ok(())
}

pub fn set_rotation(
    In((entity, x, y, z)): In<(Entity, f32, f32, f32)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.rotation = Quat::from_euler(EulerRot::XYZ, x, y, z);
    Ok(())
}

pub fn rotate_x(
    In((entity, angle)): In<(Entity, f32)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.rotate_x(angle);
    Ok(())
}

pub fn rotate_y(
    In((entity, angle)): In<(Entity, f32)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.rotate_y(angle);
    Ok(())
}

pub fn rotate_z(
    In((entity, angle)): In<(Entity, f32)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.rotate_z(angle);
    Ok(())
}

pub fn rotate_axis(
    In((entity, angle, axis_x, axis_y, axis_z)): In<(Entity, f32, f32, f32, f32)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    let axis = Vec3::new(axis_x, axis_y, axis_z).normalize();
    transform.rotate(Quat::from_axis_angle(axis, angle));
    Ok(())
}

pub fn set_scale(
    In((entity, x, y, z)): In<(Entity, f32, f32, f32)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.scale = Vec3::new(x, y, z);
    Ok(())
}

pub fn scale(
    In((entity, x, y, z)): In<(Entity, f32, f32, f32)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.scale *= Vec3::new(x, y, z);
    Ok(())
}

pub fn look_at(
    In((entity, target_x, target_y, target_z)): In<(Entity, f32, f32, f32)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    let target = Vec3::new(target_x, target_y, target_z);
    transform.look_at(target, Vec3::Y);
    Ok(())
}

pub fn reset(In(entity): In<Entity>, mut transforms: Query<&mut Transform>) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    *transform = Transform::IDENTITY;
    Ok(())
}
