use processing::prelude::*;
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError},
    prelude::*,
};

pub fn connect(port: usize) -> PyResult<()> {
    midi_connect(port).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn disconnect() -> PyResult<()> {
    midi_disconnect().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn refresh_ports() -> PyResult<()> {
    midi_refresh_ports().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn list_ports() -> PyResult<Vec<String>> {
    midi_list_ports().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn play_notes(note: u8, duration: u64) -> PyResult<()> {
    midi_play_notes(note, duration).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn note_on(note: u8, velocity: u8) -> PyResult<()> {
    midi_note_on(note, velocity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn note_off(note: u8) -> PyResult<()> {
    midi_note_off(note).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn list_input_ports() -> PyResult<Vec<String>> {
    midi_list_input_ports().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
/// `port` is a number from `midi_list_input_ports()` or part of a port name.
pub fn connect_input(port: Option<&Bound<'_, PyAny>>) -> PyResult<String> {
    let result = match port {
        None => midi_connect_input(0),
        Some(p) => {
            if let Ok(index) = p.extract::<usize>() {
                midi_connect_input(index)
            } else if let Ok(name) = p.extract::<String>() {
                midi_connect_input_by_name(&name)
            } else {
                return Err(PyTypeError::new_err(
                    "port must be a port number or part of a port name",
                ));
            }
        }
    };
    result.map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn disconnect_input() -> PyResult<()> {
    midi_disconnect_input().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}
pub fn read() -> PyResult<Vec<PyMidiMessage>> {
    midi_read()
        .map(|msgs| msgs.into_iter().map(PyMidiMessage::from).collect())
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

/// A MIDI message received on the connected input.
///
/// `kind` is one of `"note_on"`, `"note_off"`, `"control_change"`,
/// `"pitch_bend"` or `"other"`. `channel` is 0-15.
///
/// The two data bytes have names for the common kinds: `note` and `velocity`
/// for notes, `control` and `value` for control changes. For pitch bend,
/// `bend` combines them into one value from 0 to 16383 (8192 is centered).
#[pyclass(name = "MidiMessage", frozen)]
pub struct PyMidiMessage {
    status: u8,
    data1: u8,
    data2: u8,
}

impl From<MidiMessage> for PyMidiMessage {
    fn from(msg: MidiMessage) -> Self {
        let [status, data1, data2] = msg.msg;
        Self {
            status,
            data1,
            data2,
        }
    }
}

impl PyMidiMessage {
    fn message(&self) -> MidiMessage {
        MidiMessage::from([self.status, self.data1, self.data2])
    }
}

#[pymethods]
impl PyMidiMessage {
    #[getter]
    fn kind(&self) -> &'static str {
        match midi_message_kind(&self.message()) {
            MidiMessageKind::NoteOn => "note_on",
            MidiMessageKind::NoteOff => "note_off",
            MidiMessageKind::ControlChange => "control_change",
            MidiMessageKind::PitchBend => "pitch_bend",
            MidiMessageKind::Other => "other",
        }
    }

    #[getter]
    fn channel(&self) -> u8 {
        self.message().channel()
    }

    #[getter]
    fn status(&self) -> u8 {
        self.status
    }

    #[getter]
    fn data1(&self) -> u8 {
        self.data1
    }

    #[getter]
    fn data2(&self) -> u8 {
        self.data2
    }

    #[getter]
    fn note(&self) -> u8 {
        self.data1
    }

    #[getter]
    fn velocity(&self) -> u8 {
        self.data2
    }

    #[getter]
    fn control(&self) -> u8 {
        self.data1
    }

    #[getter]
    fn value(&self) -> u8 {
        self.data2
    }

    #[getter]
    fn bend(&self) -> u16 {
        (self.data2 as u16) << 7 | self.data1 as u16
    }

    fn __repr__(&self) -> String {
        format!(
            "MidiMessage(kind={:?}, channel={}, data1={}, data2={})",
            self.kind(),
            self.channel(),
            self.data1,
            self.data2
        )
    }
}
