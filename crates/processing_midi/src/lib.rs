use bevy::prelude::*;
use bevy_midi::prelude::*;

pub struct MidiPlugin;

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MidiOutputSettings {
            port_name: "output",
        })
        .add_plugins(MidiOutputPlugin);
    }
}

enum MidiCommand {}

pub fn connect(port: usize) {
    // we need to work with the ECS
    // do we pass a MidiCommand to Bevy?
}

pub fn disconnect() {}
pub fn refresh_ports() {}

pub fn play_notes() {}
