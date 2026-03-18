use bevy::prelude::*;
use bevy_midi::prelude::*;

use processing_core::app_mut;
use processing_core::error::{self, Result};

pub struct MidiPlugin;

pub const NOTE_ON: u8 = 0b1001_0000;
pub const NOTE_OFF: u8 = 0b1000_0000;

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Update `bevy_midi` to treat connections as entities
        // in order to support hot-plugging
        app.insert_resource(MidiOutputSettings {
            port_name: "libprocessing output",
        });

        app.add_plugins(MidiOutputPlugin);
    }
}

pub fn connect(In(port): In<usize>, output: Res<MidiOutput>) -> Result<()> {
    match output.ports().get(port) {
        Some((_, p)) => {
            output.connect(p.clone());
            Ok(())
        }
        None => Err(error::ProcessingError::MidiPortNotFound(port)),
    }
}

pub fn disconnect(output: Res<MidiOutput>) -> Result<()> {
    output.disconnect();
    Ok(())
}

pub fn refresh_ports(output: Res<MidiOutput>) -> Result<()> {
    output.refresh_ports();
    Ok(())
}

pub fn list_ports(output: Res<MidiOutput>) -> Result<Vec<String>> {
    Ok(output
        .ports()
        .iter()
        .enumerate()
        .map(|(i, (name, _))| format!("{}: {}", i, name))
        .collect())
}

pub fn play_notes(In((note, duration)): In<(u8, u64)>, output: Res<MidiOutput>) -> Result<()> {
    output.send([NOTE_ON, note, 127].into()); // Note on, channel 1, max velocity

    std::thread::sleep(std::time::Duration::from_millis(duration));

    output.send([NOTE_OFF, note, 127].into()); // Note off, channel 1, max velocity

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn midi_refresh_ports() -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        world.run_system_cached(refresh_ports).unwrap()
    })?;
    // run the `PreUpdate` schedule to let `bevy_midi` process it's callbacks and update the ports list
    // TODO: race condition is still present here in theory
    app_mut(|app| {
        app.world_mut().run_schedule(PreUpdate);
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
        world.run_system_cached_with(connect, port).unwrap()
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
pub fn midi_play_notes(note: u8, duration: u64) -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        world
            .run_system_cached_with(play_notes, (note, duration))
            .unwrap()
    })
}
