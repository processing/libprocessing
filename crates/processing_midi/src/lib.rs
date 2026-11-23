use bevy::prelude::*;
use nannou_midi::{
    MidiError, MidiInput, MidiInputStream, MidiOutput, MidiOutputStream, MidiPort,
    MidiPortDirection,
};

use processing_core::app_mut;
use processing_core::error::{self, Result};

pub use nannou_midi::{MidiData, MidiMessage};

pub struct MidiPlugin;

pub const NOTE_ON: u8 = 0b1001_0000;
pub const NOTE_OFF: u8 = 0b1000_0000;
pub const CONTROL_CHANGE: u8 = 0b1011_0000;
pub const PITCH_BEND: u8 = 0b1110_0000;

/// What a received [`MidiMessage`] means, decoded from its status byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiMessageKind {
    NoteOn,
    /// Includes note-on messages with velocity 0, which many devices send instead.
    NoteOff,
    ControlChange,
    PitchBend,
    Other,
}

/// Decode the kind of a MIDI message.
pub fn midi_message_kind(msg: &MidiMessage) -> MidiMessageKind {
    if msg.is_note_on() {
        MidiMessageKind::NoteOn
    } else if msg.is_note_off() {
        MidiMessageKind::NoteOff
    } else {
        match msg.msg[0] & 0xF0 {
            CONTROL_CHANGE => MidiMessageKind::ControlChange,
            PITCH_BEND => MidiMessageKind::PitchBend,
            _ => MidiMessageKind::Other,
        }
    }
}

#[derive(Resource, Default)]
struct ActiveOutput(Option<Entity>);

#[derive(Resource, Default)]
struct ActiveInput(Option<Entity>);

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveOutput>();
        app.init_resource::<ActiveInput>();
        app.add_plugins(nannou_midi::MidiPlugin);
    }
}

fn sorted_ports(world: &mut World, direction: MidiPortDirection) -> Vec<(Entity, String)> {
    let mut q = world.query::<(Entity, &Name, &MidiPort)>();
    let mut ports: Vec<(Entity, String)> = q
        .iter(world)
        .filter(|(_, _, p)| p.direction == direction)
        .map(|(e, n, _)| (e, n.as_str().to_string()))
        .collect();
    ports.sort_by(|a, b| a.1.cmp(&b.1));
    ports
}

fn sorted_output_ports(world: &mut World) -> Vec<(Entity, String)> {
    sorted_ports(world, MidiPortDirection::Output)
}

fn sorted_input_ports(world: &mut World) -> Vec<(Entity, String)> {
    sorted_ports(world, MidiPortDirection::Input)
}

fn numbered(ports: Vec<(Entity, String)>) -> Vec<String> {
    ports
        .into_iter()
        .enumerate()
        .map(|(i, (_, name))| format!("{i}: {name}"))
        .collect()
}

pub fn list_ports(world: &mut World) -> Result<Vec<String>> {
    Ok(numbered(sorted_output_ports(world)))
}

pub fn list_input_ports(world: &mut World) -> Result<Vec<String>> {
    Ok(numbered(sorted_input_ports(world)))
}

/// Which port to connect to: its number in the port list, or part of its name.
#[derive(Clone, Debug)]
pub enum MidiPortSelector {
    Index(usize),
    Name(String),
}

/// Find a port by number, or by name: an exact (case-insensitive) match wins,
/// otherwise the first port whose name contains `name`.
fn find_port(
    ports: Vec<(Entity, String)>,
    selector: &MidiPortSelector,
) -> Result<(Entity, String)> {
    match selector {
        MidiPortSelector::Index(i) => ports
            .get(*i)
            .cloned()
            .ok_or(error::ProcessingError::MidiPortNotFound(*i)),
        MidiPortSelector::Name(name) => {
            let needle = name.to_lowercase();
            let exact = ports.iter().find(|(_, n)| n.to_lowercase() == needle);
            let partial = || {
                ports
                    .iter()
                    .find(|(_, n)| n.to_lowercase().contains(&needle))
            };
            exact.or_else(partial).cloned().ok_or_else(|| {
                let available = if ports.is_empty() {
                    "none".to_string()
                } else {
                    ports
                        .iter()
                        .map(|(_, n)| n.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                error::ProcessingError::MidiPortNameNotFound(name.clone(), available)
            })
        }
    }
}

pub fn connect_input(In(selector): In<MidiPortSelector>, world: &mut World) -> Result<String> {
    let (port_entity, port_name) = find_port(sorted_input_ports(world), &selector)?;

    disconnect_input(world)?;

    let connection = world
        .spawn((
            Name::new("libprocessing input"),
            MidiInput {
                port: Some(port_entity),
            },
        ))
        .id();
    world.resource_mut::<ActiveInput>().0 = Some(connection);
    Ok(port_name)
}

pub fn disconnect_input(world: &mut World) -> Result<()> {
    let entity = world.resource_mut::<ActiveInput>().0.take();
    if let Some(e) = entity
        && let Ok(entity_mut) = world.get_entity_mut(e)
    {
        entity_mut.despawn();
    }
    Ok(())
}

/// Fails if opening the input connection failed, so the error surfaces at
/// connect time instead of as silence later.
fn check_input_connected(world: &mut World) -> Result<()> {
    let entity = world
        .resource::<ActiveInput>()
        .0
        .ok_or(error::ProcessingError::MidiInputNotConnected)?;
    if let Some(err) = world.get::<MidiError>(entity) {
        return Err(error::ProcessingError::MidiConnectionFailed(
            err.message.clone(),
        ));
    }
    Ok(())
}

/// Drain every message received since the last read.
pub fn read_messages(world: &mut World) -> Result<Vec<MidiMessage>> {
    check_input_connected(world)?;
    let entity = world
        .resource::<ActiveInput>()
        .0
        .ok_or(error::ProcessingError::MidiInputNotConnected)?;
    Ok(world
        .get_mut::<MidiInputStream>(entity)
        .map(|mut stream| stream.read().map(|data| data.message).collect())
        .unwrap_or_default())
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
pub fn midi_list_input_ports() -> error::Result<Vec<String>> {
    app_mut(|app| {
        let world = app.world_mut();
        world.run_system_cached(list_input_ports).unwrap()
    })
}

/// Connect to the input port with this number in [`midi_list_input_ports`].
/// Returns the port's name.
#[cfg(not(target_arch = "wasm32"))]
pub fn midi_connect_input(port: usize) -> error::Result<String> {
    connect_input_with(MidiPortSelector::Index(port))
}

/// Connect to the first input port whose name matches `name` (case-insensitive,
/// exact match first, then partial), e.g. `"IAC"`. Returns the port's full name.
#[cfg(not(target_arch = "wasm32"))]
pub fn midi_connect_input_by_name(name: &str) -> error::Result<String> {
    connect_input_with(MidiPortSelector::Name(name.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn connect_input_with(selector: MidiPortSelector) -> error::Result<String> {
    app_mut(|app| {
        let world = app.world_mut();
        let name = world
            .run_system_cached_with(connect_input, selector)
            .unwrap()?;
        // Open the connection now so a failure is reported here, not on first read.
        world
            .run_system_cached(nannou_midi::native::open_midi_inputs)
            .unwrap();
        check_input_connected(world)?;
        Ok(name)
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn midi_disconnect_input() -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        world.run_system_cached(disconnect_input).unwrap()
    })
}

/// Every MIDI message received on the connected input since the last call.
#[cfg(not(target_arch = "wasm32"))]
pub fn midi_read() -> error::Result<Vec<MidiMessage>> {
    app_mut(|app| {
        let world = app.world_mut();
        // Pull anything still waiting in the channel from midir's callback thread.
        world
            .run_system_cached(nannou_midi::native::receive_midi_messages)
            .unwrap();
        world.run_system_cached(read_messages).unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(bytes: [u8; 3]) -> MidiMessageKind {
        midi_message_kind(&MidiMessage::from(bytes))
    }

    #[test]
    fn decodes_message_kinds() {
        assert_eq!(kind([0x90, 60, 100]), MidiMessageKind::NoteOn);
        assert_eq!(kind([0x80, 60, 0]), MidiMessageKind::NoteOff);
        // note-on with velocity 0 is a note-off
        assert_eq!(kind([0x90, 60, 0]), MidiMessageKind::NoteOff);
        assert_eq!(kind([0xB0, 1, 64]), MidiMessageKind::ControlChange);
        assert_eq!(kind([0xE0, 0, 64]), MidiMessageKind::PitchBend);
        assert_eq!(kind([0xA0, 60, 10]), MidiMessageKind::Other);
    }

    fn ports(names: &[&str]) -> Vec<(Entity, String)> {
        names
            .iter()
            .map(|n| (Entity::PLACEHOLDER, n.to_string()))
            .collect()
    }

    fn found(names: &[&str], selector: MidiPortSelector) -> Option<String> {
        find_port(ports(names), &selector).ok().map(|(_, n)| n)
    }

    #[test]
    fn finds_ports_by_index_and_name() {
        let names = ["IAC Driver Bus 1", "IAC Driver Bus 12", "nanoKONTROL2"];
        let by_name = |n: &str| found(&names, MidiPortSelector::Name(n.to_string()));

        assert_eq!(
            found(&names, MidiPortSelector::Index(2)).as_deref(),
            Some("nanoKONTROL2")
        );
        assert_eq!(found(&names, MidiPortSelector::Index(3)), None);
        // partial and case-insensitive
        assert_eq!(by_name("nano").as_deref(), Some("nanoKONTROL2"));
        assert_eq!(by_name("NANOKONTROL").as_deref(), Some("nanoKONTROL2"));
        assert_eq!(by_name("missing"), None);

        // an exact match beats an earlier partial one
        let names = ["Ableton Synth", "Synth"];
        assert_eq!(
            found(&names, MidiPortSelector::Name("synth".to_string())).as_deref(),
            Some("Synth")
        );
    }

    #[test]
    fn name_not_found_lists_available_ports() {
        let err =
            find_port(ports(&["A", "B"]), &MidiPortSelector::Name("C".to_string())).unwrap_err();
        assert_eq!(
            err.to_string(),
            r#"No MIDI port matching "C"; available ports: A, B"#
        );
        let err = find_port(ports(&[]), &MidiPortSelector::Name("C".to_string())).unwrap_err();
        assert!(err.to_string().ends_with("available ports: none"));
    }

    #[test]
    fn kind_ignores_channel() {
        assert_eq!(kind([0x9F, 60, 100]), MidiMessageKind::NoteOn);
        assert_eq!(kind([0xB5, 7, 127]), MidiMessageKind::ControlChange);
        assert_eq!(MidiMessage::from([0x9F, 60, 100]).channel(), 15);
    }
}
