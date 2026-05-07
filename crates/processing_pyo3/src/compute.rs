use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{
    exceptions::{PyIndexError, PyRuntimeError, PyTypeError, PyValueError},
    prelude::*,
    types::{PyBytes, PyList, PySlice, PySliceIndices},
};

use shader_value::ShaderValue;

use crate::material::py_to_shader_value;
use crate::shader::Shader;

#[pyclass(unsendable)]
pub struct Buffer {
    pub(crate) entity: Entity,
    element_type: Option<ShaderValue>,
    size: u64,
    /// `true` for borrowed wrappers (e.g. `Particles.buffer()`) where the
    /// underlying entity belongs elsewhere; `Drop` skips destroy in that case.
    borrowed: bool,
}

impl Buffer {
    /// Wrap an existing buffer entity without taking ownership. `Drop` will
    /// not destroy it.
    pub(crate) fn from_entity(entity: Entity, element_type: Option<ShaderValue>) -> Self {
        let size = buffer_size(entity).unwrap_or(0);
        Self {
            entity,
            element_type,
            size,
            borrowed: true,
        }
    }
}

#[pymethods]
impl Buffer {
    #[new]
    #[pyo3(signature = (size=None, data=None))]
    pub fn new(size: Option<u64>, data: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let (entity, size, element_type) = if let Some(data) = data {
            let (bytes, element_type) = shader_values_to_bytes(data)?;
            let size = bytes.len() as u64;
            let entity = buffer_create_with_data(bytes)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
            (entity, size, element_type)
        } else {
            let size = size.unwrap_or(0);
            let entity =
                buffer_create(size).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
            (entity, size, None)
        };
        Ok(Self {
            entity,
            element_type,
            size,
            borrowed: false,
        })
    }

    pub fn __len__(&self) -> usize {
        match &self.element_type {
            Some(et) => et
                .byte_size()
                .map(|s| self.size as usize / s)
                .unwrap_or(self.size as usize),
            None => self.size as usize,
        }
    }

    pub fn __getitem__(&self, py: Python<'_>, index: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let Some(ref et) = self.element_type else {
            return Err(PyTypeError::new_err("no element type; write values first"));
        };
        let elem_size = et.byte_size().unwrap() as u64;

        let read = |i: isize| -> PyResult<Bound<'_, PyAny>> {
            let bytes = buffer_read_element(self.entity, i as u64 * elem_size, elem_size)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
            let sv = et
                .read_from_bytes(&bytes)
                .ok_or_else(|| PyRuntimeError::new_err("failed to decode element"))?;
            shader_value_to_py(py, &sv)
        };

        if let Ok(i) = index.extract::<isize>() {
            Ok(read(self.normalize_index(i)? as isize)?.into())
        } else if let Ok(slice) = index.cast::<PySlice>() {
            let indices = slice.indices(self.__len__() as isize)?;
            let values = slice_positions(&indices)
                .map(read)
                .collect::<PyResult<Vec<_>>>()?;
            Ok(PyList::new(py, values)?.into())
        } else {
            Err(PyTypeError::new_err("index must be int or slice"))
        }
    }

    pub fn __setitem__(
        &mut self,
        index: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        if let Ok(i) = index.extract::<isize>() {
            let sv = py_to_shader_value(value)?;
            self.check_element_type(&sv)?;
            let bytes = sv
                .to_bytes()
                .ok_or_else(|| PyTypeError::new_err("unsupported value type for buffer"))?;
            let elem_size = bytes.len() as u64;
            let i = self.normalize_index(i)?;
            buffer_write_element(self.entity, i as u64 * elem_size, bytes)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
        } else if let Ok(slice) = index.cast::<PySlice>() {
            let (src_bytes, element_type) = shader_values_to_bytes(value)?;
            let et = element_type
                .ok_or_else(|| PyTypeError::new_err("unsupported value type for buffer"))?;
            let elem_size = et.byte_size().unwrap() as u64;
            self.check_element_type(&et)?;
            let indices = slice.indices(self.__len__() as isize)?;
            let src_elems = src_bytes.len() as u64 / elem_size;
            if indices.slicelength as u64 != src_elems {
                return Err(PyValueError::new_err(format!(
                    "slice length {} does not match value length {}",
                    indices.slicelength, src_elems
                )));
            }
            for (pos, chunk) in
                slice_positions(&indices).zip(src_bytes.chunks_exact(elem_size as usize))
            {
                buffer_write_element(self.entity, pos as u64 * elem_size, chunk.to_vec())
                    .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
            }
            Ok(())
        } else {
            Err(PyTypeError::new_err("index must be int or slice"))
        }
    }

    pub fn write(&mut self, values: &Bound<'_, PyAny>) -> PyResult<()> {
        // bytes path skips per-element conversion for large uploads
        if let Ok(b) = values.cast::<PyBytes>() {
            return buffer_write(self.entity, b.as_bytes().to_vec())
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")));
        }
        let (bytes, element_type) = shader_values_to_bytes(values)?;
        self.element_type = element_type;
        buffer_write(self.entity, bytes).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn read<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let data = buffer_read(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let Some(ref template) = self.element_type else {
            return Ok(PyBytes::new(py, &data).into_any());
        };

        let elem_size = template
            .byte_size()
            .ok_or_else(|| PyRuntimeError::new_err("unsupported element type"))?;

        let values = data
            .chunks_exact(elem_size)
            .map(|chunk| {
                let sv = template
                    .read_from_bytes(chunk)
                    .ok_or_else(|| PyRuntimeError::new_err("failed to decode bytes"))?;
                shader_value_to_py(py, &sv)
            })
            .collect::<PyResult<Vec<_>>>()?;

        Ok(PyList::new(py, values)?.into_any())
    }
}

impl Buffer {
    fn check_element_type(&mut self, sv: &ShaderValue) -> PyResult<()> {
        match &self.element_type {
            Some(existing) if std::mem::discriminant(existing) != std::mem::discriminant(sv) => {
                Err(PyTypeError::new_err(format!(
                    "buffer element type mismatch: expected {existing:?}, got {sv:?}"
                )))
            }
            Some(_) => Ok(()),
            None => {
                self.element_type = Some(sv.clone());
                Ok(())
            }
        }
    }

    fn normalize_index(&self, i: isize) -> PyResult<usize> {
        let len = self.__len__() as isize;
        let i = if i < 0 { len + i } else { i };
        if i < 0 || i >= len {
            Err(PyIndexError::new_err("buffer index out of range"))
        } else {
            Ok(i as usize)
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if !self.borrowed {
            let _ = buffer_destroy(self.entity);
        }
    }
}

fn slice_positions(indices: &PySliceIndices) -> impl Iterator<Item = isize> + use<> {
    let PySliceIndices {
        start,
        step,
        slicelength,
        ..
    } = *indices;
    (0..slicelength as isize).map(move |i| start + i * step)
}

fn shader_values_to_bytes(values: &Bound<'_, PyAny>) -> PyResult<(Vec<u8>, Option<ShaderValue>)> {
    let mut bytes = Vec::new();
    let mut element_type: Option<ShaderValue> = None;
    for item in values.try_iter()? {
        let sv = py_to_shader_value(&item?)?;
        if let Some(ref existing) = element_type
            && std::mem::discriminant(existing) != std::mem::discriminant(&sv)
        {
            return Err(PyTypeError::new_err(format!(
                "buffer elements must all share the same type: expected {existing:?}, got {sv:?}"
            )));
        }
        let b = sv
            .to_bytes()
            .ok_or_else(|| PyTypeError::new_err("unsupported value type for buffer"))?;
        element_type.get_or_insert(sv);
        bytes.extend_from_slice(&b);
    }
    Ok((bytes, element_type))
}

fn shader_value_to_py<'py>(py: Python<'py>, sv: &ShaderValue) -> PyResult<Bound<'py, PyAny>> {
    fn list<'py, T: pyo3::IntoPyObject<'py> + Copy>(
        py: Python<'py>,
        xs: &[T],
    ) -> PyResult<Bound<'py, PyAny>> {
        Ok(PyList::new(py, xs.iter().copied())?.into_any())
    }
    match sv {
        ShaderValue::Float(v) => Ok(v.into_pyobject(py)?.into_any()),
        ShaderValue::Int(v) => Ok(v.into_pyobject(py)?.into_any()),
        ShaderValue::UInt(v) => Ok(v.into_pyobject(py)?.into_any()),
        ShaderValue::Float2(v) => list(py, v),
        ShaderValue::Float3(v) => list(py, v),
        ShaderValue::Float4(v) => list(py, v),
        ShaderValue::Int2(v) => list(py, v),
        ShaderValue::Int3(v) => list(py, v),
        ShaderValue::Int4(v) => list(py, v),
        ShaderValue::Mat4(v) => list(py, v),
        ShaderValue::Texture(_) | ShaderValue::Buffer(_) => Err(PyRuntimeError::new_err(
            "cannot convert Texture/Buffer to Python value",
        )),
    }
}

#[pyclass(unsendable)]
pub struct Compute {
    pub(crate) entity: Entity,
}

impl Compute {
    /// Wrap an existing compute entity (e.g., one created by a Rust-side
    /// factory like `field_kernel_noise`). Not exposed to Python directly.
    pub(crate) fn from_entity(entity: Entity) -> Self {
        Self { entity }
    }
}

#[pymethods]
impl Compute {
    #[new]
    pub fn new(shader: &Shader) -> PyResult<Self> {
        let entity =
            compute_create(shader.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self { entity })
    }

    #[pyo3(signature = (**kwargs))]
    pub fn set(&self, kwargs: Option<&Bound<'_, pyo3::types::PyDict>>) -> PyResult<()> {
        let Some(kwargs) = kwargs else {
            return Ok(());
        };
        for (key, value) in kwargs.iter() {
            let name: String = key.extract()?;
            let value = py_to_shader_value(&value)?;
            compute_set(self.entity, &name, value)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        }
        Ok(())
    }

    pub fn dispatch(&self, x: u32, y: u32, z: u32) -> PyResult<()> {
        compute_dispatch(self.entity, x, y, z).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }
}

impl Drop for Compute {
    fn drop(&mut self) {
        let _ = compute_destroy(self.entity);
    }
}
