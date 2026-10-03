pub use bevy::input::keyboard::KeyCode;
pub use bevy::input::mouse::MouseButton;
pub use bevy::prelude::default;
pub use bevy::render::render_resource::TextureFormat;
pub use processing_core::{config::*, constants, error};
pub use processing_input::*;
// TEMP(bevy-020): processing_midi pins bevy 0.19; stubbed while the port is
// in progress. Restore the re-export (plus Cargo.toml deps and the
// MidiPlugin registration in lib.rs) once nannou_midi is ported.
#[cfg(not(target_arch = "wasm32"))]
mod midi_stubs {
    use processing_core::error::{ProcessingError, Result};
    fn unavailable<T>() -> Result<T> {
        Err(ProcessingError::InvalidArgument(
            "midi support is temporarily disabled in this build".to_string(),
        ))
    }
    pub fn midi_connect(_port: usize) -> Result<()> {
        unavailable()
    }
    pub fn midi_disconnect() -> Result<()> {
        unavailable()
    }
    pub fn midi_list_ports() -> Result<Vec<String>> {
        unavailable()
    }
    pub fn midi_refresh_ports() -> Result<()> {
        unavailable()
    }
    pub fn midi_play_notes(_note: u8, _duration: u64) -> Result<()> {
        unavailable()
    }
    pub fn midi_note_on(_note: u8, _velocity: u8) -> Result<()> {
        unavailable()
    }
    pub fn midi_note_off(_note: u8) -> Result<()> {
        unavailable()
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub use midi_stubs::*;
pub use processing_render::{
    render::command::{
        ArcMode, BlendMode, DrawCommand, ShapeKind, ShapeMode, StrokeCapMode, StrokeJoinMode,
        TextAlignH, TextAlignV, TextStyle, TextWrapMode, custom_blend_state,
    },
    render::filter::Filter,
    *,
};

pub use crate::{exit, init};
