use bevy::prelude::*;
use bevy::window::{Monitor, PrimaryMonitor};

pub fn list(query: Query<Entity, With<Monitor>>) -> Vec<Entity> {
    query.iter().collect()
}

pub fn primary(query: Query<Entity, With<PrimaryMonitor>>) -> Option<Entity> {
    query.iter().next()
}

pub fn width(In(entity): In<Entity>, query: Query<&Monitor>) -> u32 {
    query.get(entity).map(|m| m.physical_width).unwrap_or(0)
}

pub fn height(In(entity): In<Entity>, query: Query<&Monitor>) -> u32 {
    query.get(entity).map(|m| m.physical_height).unwrap_or(0)
}

pub fn scale_factor(In(entity): In<Entity>, query: Query<&Monitor>) -> f64 {
    query.get(entity).map(|m| m.scale_factor).unwrap_or(1.0)
}

pub fn refresh_rate_millihertz(In(entity): In<Entity>, query: Query<&Monitor>) -> Option<u32> {
    query
        .get(entity)
        .ok()
        .and_then(|m| m.refresh_rate_millihertz)
}

pub fn name(In(entity): In<Entity>, query: Query<&Monitor>) -> Option<String> {
    query.get(entity).ok().and_then(|m| m.name.clone())
}
