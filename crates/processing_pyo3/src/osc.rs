use processing::prelude::*;
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError},
    prelude::*,
    types::{PyBool, PyBytes, PyFloat, PyInt, PyList, PyString, PyTuple},
};
use std::net::SocketAddr;

fn runtime_err(e: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(format!("{e}"))
}

pub fn listen(port: u16) -> PyResult<u16> {
    osc_listen(port).map_err(runtime_err)
}
pub fn stop() -> PyResult<()> {
    osc_stop().map_err(runtime_err)
}
pub fn send(
    host: &str,
    port: u16,
    address: &str,
    args: &Bound<'_, PyTuple>,
) -> PyResult<PyOscMessage> {
    let args = args
        .iter()
        .map(|a| to_osc(&a))
        .collect::<PyResult<Vec<_>>>()?;
    let peer = osc_send(host, port, address, args.clone()).map_err(runtime_err)?;
    Ok(PyOscMessage {
        message: OscMessage {
            addr: address.to_string(),
            args,
        },
        peer,
    })
}
pub fn read() -> PyResult<Vec<PyOscMessage>> {
    osc_read()
        .map(|msgs| msgs.into_iter().map(PyOscMessage::from).collect())
        .map_err(runtime_err)
}

/// Python value -> OSC argument. `bool` is checked before `int` because
/// Python's `True`/`False` are ints too.
fn to_osc(value: &Bound<'_, PyAny>) -> PyResult<OscType> {
    Ok(if value.is_none() {
        OscType::Nil
    } else if value.is_instance_of::<PyBool>() {
        OscType::Bool(value.extract()?)
    } else if value.is_instance_of::<PyInt>() {
        let i: i64 = value.extract()?;
        i32::try_from(i).map_or(OscType::Long(i), OscType::Int)
    } else if value.is_instance_of::<PyFloat>() {
        // 32-bit floats ("f") are what most OSC software expects.
        OscType::Float(value.extract::<f64>()? as f32)
    } else if let Ok(s) = value.cast::<PyString>() {
        OscType::String(s.to_str()?.to_owned())
    } else if let Ok(b) = value.cast::<PyBytes>() {
        OscType::Blob(b.as_bytes().to_vec())
    } else if value.is_instance_of::<PyList>() || value.is_instance_of::<PyTuple>() {
        let content = value
            .try_iter()?
            .map(|item| to_osc(&item?))
            .collect::<PyResult<Vec<_>>>()?;
        OscType::Array(OscArray { content })
    } else {
        return Err(PyTypeError::new_err(format!(
            "can't send {} over OSC; use int, float, str, bool, bytes, None or a list of those",
            value.get_type().name()?
        )));
    })
}

/// OSC argument -> Python value.
fn to_py<'py>(py: Python<'py>, value: &OscType) -> PyResult<Bound<'py, PyAny>> {
    Ok(match value {
        OscType::Int(i) => i.into_pyobject(py)?.into_any(),
        OscType::Long(i) => i.into_pyobject(py)?.into_any(),
        OscType::Float(f) => f.into_pyobject(py)?.into_any(),
        OscType::Double(d) => d.into_pyobject(py)?.into_any(),
        OscType::String(s) => s.into_pyobject(py)?.into_any(),
        OscType::Char(c) => c.into_pyobject(py)?.into_any(),
        OscType::Blob(b) => PyBytes::new(py, b).into_any(),
        OscType::Bool(b) => b.into_pyobject(py)?.to_owned().into_any(),
        OscType::Nil | OscType::Inf => py.None().into_bound(py),
        OscType::Time(t) => (t.seconds, t.fractional).into_pyobject(py)?.into_any(),
        OscType::Color(c) => (c.red, c.green, c.blue, c.alpha)
            .into_pyobject(py)?
            .into_any(),
        OscType::Midi(m) => (m.port, m.status, m.data1, m.data2)
            .into_pyobject(py)?
            .into_any(),
        OscType::Array(a) => {
            let items = a
                .content
                .iter()
                .map(|v| to_py(py, v))
                .collect::<PyResult<Vec<_>>>()?;
            PyList::new(py, items)?.into_any()
        }
    })
}

/// The OSC type tag character for an argument, e.g. `i` for int.
fn type_tag(value: &OscType, out: &mut String) {
    match value {
        OscType::Int(_) => out.push('i'),
        OscType::Long(_) => out.push('h'),
        OscType::Float(_) => out.push('f'),
        OscType::Double(_) => out.push('d'),
        OscType::String(_) => out.push('s'),
        OscType::Char(_) => out.push('c'),
        OscType::Blob(_) => out.push('b'),
        OscType::Bool(true) => out.push('T'),
        OscType::Bool(false) => out.push('F'),
        OscType::Nil => out.push('N'),
        OscType::Inf => out.push('I'),
        OscType::Time(_) => out.push('t'),
        OscType::Color(_) => out.push('r'),
        OscType::Midi(_) => out.push('m'),
        OscType::Array(a) => {
            out.push('[');
            a.content.iter().for_each(|v| type_tag(v, out));
            out.push(']');
        }
    }
}

/// An OSC message: one returned by `osc_read()`, or the one `osc_send()` just sent.
///
/// `address` is the message's address pattern, like `"/mouse"`. `args` are
/// its arguments as Python values, and `typetag` spells out their OSC types,
/// one letter each (`"ffs"` is two floats and a string). `host` and `port`
/// are the other side: who sent it for received messages, where it went for
/// sent ones. `str(msg)` gives a short readable form like `/mouse 0.5 0.25`.
#[pyclass(name = "OscMessage", frozen)]
pub struct PyOscMessage {
    message: OscMessage,
    peer: SocketAddr,
}

impl From<ReceivedOscMessage> for PyOscMessage {
    fn from(r: ReceivedOscMessage) -> Self {
        Self {
            message: r.message,
            peer: r.from,
        }
    }
}

#[pymethods]
impl PyOscMessage {
    #[getter]
    fn address(&self) -> &str {
        &self.message.addr
    }

    #[getter]
    fn args<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let items = self
            .message
            .args
            .iter()
            .map(|v| to_py(py, v))
            .collect::<PyResult<Vec<_>>>()?;
        PyList::new(py, items)
    }

    #[getter]
    fn typetag(&self) -> String {
        let mut tag = String::new();
        self.message.args.iter().for_each(|v| type_tag(v, &mut tag));
        tag
    }

    #[getter]
    fn host(&self) -> String {
        self.peer.ip().to_string()
    }

    #[getter]
    fn port(&self) -> u16 {
        self.peer.port()
    }

    fn __str__(&self) -> String {
        osc_message_text(&self.message)
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "OscMessage(address={:?}, args={}, peer={})",
            self.message.addr,
            self.args(py)?.repr()?,
            self.peer
        ))
    }
}
