use bevy::prelude::*;

use processing_core::error::{ProcessingError, Result};

pub fn set_position(
    In((entity, position)): In<(Entity, Vec3)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.translation = position;
    Ok(())
}

pub fn translate(
    In((entity, offset)): In<(Entity, Vec3)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.translation += offset;
    Ok(())
}

pub fn set_rotation(
    In((entity, euler)): In<(Entity, Vec3)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.rotation = Quat::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z);
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
    In((entity, angle, axis)): In<(Entity, f32, Vec3)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.rotate(Quat::from_axis_angle(axis.normalize(), angle));
    Ok(())
}

pub fn set_scale(
    In((entity, scale)): In<(Entity, Vec3)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.scale = scale;
    Ok(())
}

pub fn scale(
    In((entity, factor)): In<(Entity, Vec3)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
    transform.scale *= factor;
    Ok(())
}

pub fn look_at(
    In((entity, target)): In<(Entity, Vec3)>,
    mut transforms: Query<&mut Transform>,
) -> Result<()> {
    let mut transform = transforms
        .get_mut(entity)
        .map_err(|_| ProcessingError::TransformNotFound)?;
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
