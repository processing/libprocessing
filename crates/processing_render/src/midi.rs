use crate::error::Result;
use bevy::prelude::*;

use bevy_midi::prelude::*;

pub struct MidiPlugin;

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Update `bevy_midi` to treat connections as entities
        // in order to support hot-plugging
        app.insert_resource(MidiOutputSettings {
            port_name: "output",
        });

        app.add_plugins(MidiOutputPlugin);
    }
}

pub fn connect(In(port): In<usize>, output: Res<MidiOutput>) -> Result<()> {
    if let Some((_, port)) = output.ports().get(port) {
        output.connect(port.clone());
    }
    Ok(())
}

pub fn disconnect() -> Result<()> {
    Ok(())
}

pub fn refresh_ports(output: Res<MidiOutput>) -> Result<()> {
    output.refresh_ports();
    Ok(())
}

pub fn play_notes(In(note): In<u8>, output: Res<MidiOutput>) -> Result<()> {
    output.send([0b1001_0000, note, 127].into()); // Note on, channel 1, max velocity

    output.send([0b1000_0000, note, 127].into()); // Note on, channel 1, max velocity

    Ok(())
}
