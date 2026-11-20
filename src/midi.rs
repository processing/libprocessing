use processing_core::error::Result;

pub fn refresh_ports() -> Result<()> {
    processing_midi::midi_refresh_ports()
}

/// Output port names, indexed as [`connect`] takes them.
pub fn ports() -> Result<Vec<String>> {
    processing_midi::midi_list_ports()
}

pub fn connect(port: usize) -> Result<()> {
    processing_midi::midi_connect(port)
}

pub fn disconnect() -> Result<()> {
    processing_midi::midi_disconnect()
}

pub fn note_on(note: u8, velocity: u8) -> Result<()> {
    processing_midi::midi_note_on(note, velocity)
}

pub fn note_off(note: u8) -> Result<()> {
    processing_midi::midi_note_off(note)
}

/// `duration` in milliseconds.
pub fn play_note(note: u8, duration: u64) -> Result<()> {
    processing_midi::midi_play_notes(note, duration)
}
