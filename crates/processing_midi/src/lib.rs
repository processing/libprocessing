use bevy::prelude::*;

pub struct MidiPlugin;

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        dbg!(app);
    }
}
