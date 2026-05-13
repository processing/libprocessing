use bevy::prelude::*;
use nannou_midi::{MidiOutput, MidiOutputStream, MidiPort, MidiPortDirection};

use processing_core::app_mut;
use processing_core::error::{self, Result};

pub use nannou_midi::{MidiData, MidiMessage};

pub struct MidiPlugin;

pub const NOTE_ON: u8 = 0b1001_0000;
pub const NOTE_OFF: u8 = 0b1000_0000;

#[derive(Resource, Default)]
struct ActiveOutput(Option<Entity>);

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveOutput>();
        app.add_plugins(nannou_midi::MidiPlugin);
    }
}

fn sorted_output_ports(world: &mut World) -> Vec<(Entity, String)> {
    let mut q = world.query::<(Entity, &Name, &MidiPort)>();
    let mut ports: Vec<(Entity, String)> = q
        .iter(world)
        .filter(|(_, _, p)| p.direction == MidiPortDirection::Output)
        .map(|(e, n, _)| (e, n.as_str().to_string()))
        .collect();
    ports.sort_by(|a, b| a.1.cmp(&b.1));
    ports
}

pub fn list_ports(world: &mut World) -> Result<Vec<String>> {
    Ok(sorted_output_ports(world)
        .into_iter()
        .enumerate()
        .map(|(i, (_, name))| format!("{i}: {name}"))
        .collect())
}

pub fn connect(In(port): In<usize>, world: &mut World) -> Result<()> {
    let port_entity = sorted_output_ports(world)
        .get(port)
        .map(|(e, _)| *e)
        .ok_or(error::ProcessingError::MidiPortNotFound(port))?;

    let previous = world.resource::<ActiveOutput>().0;
    if let Some(e) = previous
        && let Ok(entity) = world.get_entity_mut(e)
    {
        entity.despawn();
    }

    let connection = world
        .spawn((
            Name::new("libprocessing output"),
            MidiOutput {
                port: Some(port_entity),
            },
        ))
        .id();
    world.resource_mut::<ActiveOutput>().0 = Some(connection);
    Ok(())
}

pub fn disconnect(world: &mut World) -> Result<()> {
    let entity = world.resource_mut::<ActiveOutput>().0.take();
    if let Some(e) = entity
        && let Ok(entity_mut) = world.get_entity_mut(e)
    {
        entity_mut.despawn();
    }
    Ok(())
}

pub fn send_message(In(msg): In<MidiMessage>, world: &mut World) -> Result<()> {
    let entity = world
        .resource::<ActiveOutput>()
        .0
        .ok_or(error::ProcessingError::MidiPortNotFound(usize::MAX))?;
    if let Some(mut stream) = world.get_mut::<MidiOutputStream>(entity) {
        stream.send(msg);
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn midi_refresh_ports() -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        world
            .run_system_cached(nannou_midi::native::enumerate_midi_ports)
            .unwrap();
        Ok(())
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn midi_list_ports() -> error::Result<Vec<String>> {
    app_mut(|app| {
        let world = app.world_mut();
        world.run_system_cached(list_ports).unwrap()
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn midi_connect(port: usize) -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        world.run_system_cached_with(connect, port).unwrap()?;
        // Materialize the MidiOutputStream component before the caller sends.
        world
            .run_system_cached(nannou_midi::native::open_midi_outputs)
            .unwrap();
        Ok(())
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn midi_disconnect() -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        world.run_system_cached(disconnect).unwrap()
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn midi_note_on(note: u8, velocity: u8) -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        world
            .run_system_cached_with(send_message, MidiMessage::from([NOTE_ON, note, velocity]))
            .unwrap()?;
        world
            .run_system_cached(nannou_midi::native::send_midi_messages)
            .unwrap();
        Ok(())
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn midi_note_off(note: u8) -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        world
            .run_system_cached_with(send_message, MidiMessage::from([NOTE_OFF, note, 0]))
            .unwrap()?;
        world
            .run_system_cached(nannou_midi::native::send_midi_messages)
            .unwrap();
        Ok(())
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn midi_play_notes(note: u8, duration: u64) -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        world
            .run_system_cached_with(send_message, MidiMessage::from([NOTE_ON, note, 127]))
            .unwrap()?;
        world
            .run_system_cached(nannou_midi::native::send_midi_messages)
            .unwrap();
        Ok(())
    })?;

    std::thread::sleep(std::time::Duration::from_millis(duration));

    app_mut(|app| {
        let world = app.world_mut();
        world
            .run_system_cached_with(send_message, MidiMessage::from([NOTE_OFF, note, 127]))
            .unwrap()?;
        world
            .run_system_cached(nannou_midi::native::send_midi_messages)
            .unwrap();
        Ok(())
    })
}
