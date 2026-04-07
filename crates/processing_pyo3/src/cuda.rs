use processing::prelude::image::pixel_size;
use processing_cuda::{cuda_buffer, cuda_write_back, typestr_for_format};
use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyDict};

/// Implements `__cuda_array_interface__` for zero-copy interop
/// with PyTorch, CuPy, and other CUDA-aware Python libraries.
#[pyclass(unsendable)]
pub struct CudaImage {
    entity: bevy::prelude::Entity,
}

impl CudaImage {
    pub fn new(entity: bevy::prelude::Entity) -> Self {
        Self { entity }
    }
}

#[pymethods]
impl CudaImage {
    pub fn sync(&self) -> PyResult<()> {
        cuda_write_back(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn shape(&self) -> PyResult<(u32, u32, u32)> {
        let info = cuda_buffer(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok((info.height, info.width, 4))
    }

    #[getter]
    pub fn device_ptr(&self) -> PyResult<u64> {
        let info = cuda_buffer(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(info.device_ptr)
    }

    #[getter]
    pub fn __cuda_array_interface__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let info = cuda_buffer(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let typestr = typestr_for_format(info.texture_format)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let px_size =
            pixel_size(info.texture_format).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;

        let height = info.height as usize;
        let width = info.width as usize;
        let channels: usize = 4;
        let elem_size = px_size / channels;

        let dict = PyDict::new(py);
        dict.set_item("data", (info.device_ptr, false))?;
        dict.set_item("shape", (height, width, channels))?;
        dict.set_item("typestr", typestr)?;
        dict.set_item("strides", (width * px_size, px_size, elem_size))?;
        dict.set_item("version", 3)?;

        Ok(dict)
    }
}
