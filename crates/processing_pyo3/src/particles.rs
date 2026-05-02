use bevy::prelude::Entity;
use processing::prelude::*;
use processing_render::geometry as geometry;
use pyo3::types::PyDict;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use std::collections::HashMap;

use crate::compute::{Buffer, Compute};
use crate::graphics::Geometry;

/// Per-element format for an attribute.
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AttributeFormat {
    Float = 1,
    Float2 = 2,
    Float3 = 3,
    Float4 = 4,
}

impl AttributeFormat {
    pub(crate) fn to_inner(self) -> geometry::AttributeFormat {
        match self {
            Self::Float => geometry::AttributeFormat::Float,
            Self::Float2 => geometry::AttributeFormat::Float2,
            Self::Float3 => geometry::AttributeFormat::Float3,
            Self::Float4 => geometry::AttributeFormat::Float4,
        }
    }

    pub(crate) fn from_inner(inner: geometry::AttributeFormat) -> Self {
        match inner {
            geometry::AttributeFormat::Float => Self::Float,
            geometry::AttributeFormat::Float2 => Self::Float2,
            geometry::AttributeFormat::Float3 => Self::Float3,
            geometry::AttributeFormat::Float4 => Self::Float4,
        }
    }

    pub(crate) fn float_count(self) -> usize {
        match self {
            Self::Float => 1,
            Self::Float2 => 2,
            Self::Float3 => 3,
            Self::Float4 => 4,
        }
    }
}

/// Named typed attribute. Use the `position()`/`color()`/etc. classmethods for
/// builtins or `Attribute(name, format)` for custom ones.
#[pyclass(unsendable, frozen, hash, eq)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Attribute {
    pub(crate) entity: Entity,
}

#[pymethods]
impl Attribute {
    #[new]
    pub fn new(name: &str, format: AttributeFormat) -> PyResult<Self> {
        let entity = geometry_attribute_create(name, format.to_inner())
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Self { entity })
    }

    #[staticmethod]
    pub fn position() -> Self { Self { entity: geometry_attribute_position() } }
    #[staticmethod]
    pub fn normal() -> Self { Self { entity: geometry_attribute_normal() } }
    #[staticmethod]
    pub fn color() -> Self { Self { entity: geometry_attribute_color() } }
    #[staticmethod]
    pub fn uv() -> Self { Self { entity: geometry_attribute_uv() } }
    #[staticmethod]
    pub fn rotation() -> Self { Self { entity: geometry_attribute_rotation() } }
    #[staticmethod]
    pub fn scale() -> Self { Self { entity: geometry_attribute_scale() } }
    #[staticmethod]
    pub fn dead() -> Self { Self { entity: geometry_attribute_dead() } }

    #[getter]
    pub fn name(&self) -> PyResult<String> {
        let (name, _) = geometry_attribute_info(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(name)
    }

    #[getter]
    pub fn format(&self) -> PyResult<AttributeFormat> {
        let (_, fmt) = geometry_attribute_info(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(AttributeFormat::from_inner(fmt))
    }
}

#[pyclass(unsendable)]
pub struct Particles {
    pub(crate) entity: Entity,
    /// Name → (entity, format) so `emit(**kwargs)` can route kwargs to the
    /// right attribute and pack them into bytes.
    name_to_attr: HashMap<String, (Entity, AttributeFormat)>,
}

impl Particles {
    fn build_name_index(attrs: &[Attribute]) -> PyResult<HashMap<String, (Entity, AttributeFormat)>> {
        let mut map = HashMap::with_capacity(attrs.len());
        for attr in attrs {
            let (name, fmt) = geometry_attribute_info(attr.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
            map.insert(name, (attr.entity, AttributeFormat::from_inner(fmt)));
        }
        Ok(map)
    }
}

#[pymethods]
impl Particles {
    /// Pass `capacity` for empty buffers, or `geometry` to seed positions
    /// (and matching attributes) from a source mesh. Exactly one is required.
    #[new]
    #[pyo3(signature = (capacity=None, attributes=None, geometry=None))]
    pub fn new(
        capacity: Option<u32>,
        attributes: Option<Vec<PyRef<Attribute>>>,
        geometry: Option<&Geometry>,
    ) -> PyResult<Self> {
        let attrs: Vec<Attribute> = attributes
            .unwrap_or_default()
            .iter()
            .map(|a| (**a).clone())
            .collect();
        let attr_entities: Vec<Entity> = attrs.iter().map(|a| a.entity).collect();

        let entity = match (capacity, geometry) {
            (Some(cap), None) => particles_create(cap, attr_entities)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
            (None, Some(g)) => particles_create_from_geometry(g.entity, attr_entities)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
            (None, None) => {
                return Err(PyRuntimeError::new_err(
                    "Particles requires either capacity or geometry",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(PyRuntimeError::new_err(
                    "Particles accepts capacity or geometry, not both",
                ));
            }
        };

        Ok(Self {
            entity,
            name_to_attr: Particles::build_name_index(&attrs)?,
        })
    }

    #[getter]
    pub fn capacity(&self) -> PyResult<u32> {
        particles_capacity(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Backing `Buffer` for a registered attribute, or `None` if not registered.
    /// The element type matches the attribute's format so `read()` returns
    /// typed values.
    pub fn buffer(&self, attribute: &Attribute) -> PyResult<Option<Buffer>> {
        let buf = particles_buffer(self.entity, attribute.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let (_, fmt) = geometry_attribute_info(attribute.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let element_type = match AttributeFormat::from_inner(fmt) {
            AttributeFormat::Float => shader_value::ShaderValue::Float(0.0),
            AttributeFormat::Float2 => shader_value::ShaderValue::Float2([0.0; 2]),
            AttributeFormat::Float3 => shader_value::ShaderValue::Float3([0.0; 3]),
            AttributeFormat::Float4 => shader_value::ShaderValue::Float4([0.0; 4]),
        };
        Ok(buf.map(|e| Buffer::from_entity(e, Some(element_type))))
    }

    /// Dispatch a compute kernel against these particles' buffers. Buffers
    /// are auto-bound by attribute name; kwargs are forwarded to
    /// `compute.set(...)`. For example:
    ///
    /// ```python
    /// p.apply(noise, scale=0.25, strength=0.02, time=t)
    /// ```
    #[pyo3(signature = (compute, **kwargs))]
    pub fn apply(
        &self,
        compute: &Compute,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        if let Some(kwargs) = kwargs {
            compute.set(Some(kwargs))?;
        }
        particles_apply(self.entity, compute.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Emit `n` particles into the next ring-buffer slots. Per-attribute data
    /// is passed as kwargs keyed by attribute name; each value is a flat list
    /// of `n * format.float_count()` floats.
    ///
    /// ```python
    /// p.emit(50, position=[x0,y0,z0, x1,y1,z1, ...], color=[r0,g0,b0,a0, ...])
    /// ```
    #[pyo3(signature = (n, **kwargs))]
    pub fn emit(&self, n: u32, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let Some(kwargs) = kwargs else {
            return particles_emit(self.entity, n, vec![])
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")));
        };
        let mut data: Vec<(Entity, Vec<u8>)> = Vec::new();
        for (key, value) in kwargs.iter() {
            let name: String = key.extract()?;
            let (attr_entity, fmt) = self.name_to_attr.get(&name).copied().ok_or_else(|| {
                PyRuntimeError::new_err(format!(
                    "no attribute named '{name}' (registered: {:?})",
                    self.name_to_attr.keys().collect::<Vec<_>>()
                ))
            })?;
            let floats: Vec<f32> = value.extract()?;
            let expected = (n as usize) * fmt.float_count();
            if floats.len() != expected {
                return Err(PyRuntimeError::new_err(format!(
                    "attribute '{name}': expected {expected} floats ({} per particle × {n}), got {}",
                    fmt.float_count(),
                    floats.len(),
                )));
            }
            let bytes: Vec<u8> = floats.iter().flat_map(|f| f.to_le_bytes()).collect();
            data.push((attr_entity, bytes));
        }
        particles_emit(self.entity, n, data)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    /// Emit `n` particles via a GPU kernel. Buffer bindings and a
    /// `emit_range: vec4<f32> = (base_slot, n, capacity, 0)` uniform are
    /// auto-bound; set any other uniforms via `compute.set(...)` first.
    pub fn emit_gpu(&self, n: u32, compute: &Compute) -> PyResult<()> {
        particles_emit_gpu(self.entity, n, compute.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }
}

impl Drop for Particles {
    fn drop(&mut self) {
        let _ = particles_destroy(self.entity);
    }
}

/// Built-in noise kernel. Uniforms: `scale`, `strength`, `time`.
pub fn kernel_noise() -> PyResult<Compute> {
    let entity = particles_kernel_noise().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(Compute::from_entity(entity))
}

/// Built-in transform kernel: scale → axis-angle rotate → translate. Uniforms:
/// `translate: vec3`, `rotation_axis: vec3`, `rotation_angle: f32`,
/// `scale: vec3`. Identity defaults are seeded so unset uniforms are no-ops.
pub fn kernel_transform() -> PyResult<Compute> {
    let entity = particles_kernel_transform().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    Ok(Compute::from_entity(entity))
}
