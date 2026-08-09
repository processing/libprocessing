use bevy::prelude::Entity;
use processing::prelude::*;
use processing_render::geometry;
use pyo3::types::PyDict;
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError, PyValueError},
    prelude::*,
};
use std::collections::HashMap;

use crate::compute::{Buffer, Compute};
use crate::graphics::Geometry;

#[pyclass(eq, eq_int, from_py_object)]
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

#[pyclass(unsendable, frozen, hash, eq, from_py_object)]
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
    pub fn position() -> Self {
        Self {
            entity: geometry_attribute_position(),
        }
    }
    #[staticmethod]
    pub fn normal() -> Self {
        Self {
            entity: geometry_attribute_normal(),
        }
    }
    #[staticmethod]
    pub fn color() -> Self {
        Self {
            entity: geometry_attribute_color(),
        }
    }
    #[staticmethod]
    pub fn uv() -> Self {
        Self {
            entity: geometry_attribute_uv(),
        }
    }
    #[staticmethod]
    pub fn rotation() -> Self {
        Self {
            entity: geometry_attribute_rotation(),
        }
    }
    #[staticmethod]
    pub fn scale() -> Self {
        Self {
            entity: geometry_attribute_scale(),
        }
    }
    #[staticmethod]
    pub fn life() -> Self {
        Self {
            entity: geometry_attribute_life(),
        }
    }
    #[staticmethod]
    pub fn velocity() -> Self {
        Self {
            entity: geometry_attribute_velocity(),
        }
    }
    #[staticmethod]
    pub fn age() -> Self {
        Self {
            entity: geometry_attribute_age(),
        }
    }

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
    name_to_attr: HashMap<String, (Entity, AttributeFormat)>,
}

impl Particles {
    fn build_name_index(
        attrs: &[Attribute],
    ) -> PyResult<HashMap<String, (Entity, AttributeFormat)>> {
        let mut map = HashMap::with_capacity(attrs.len());
        for attr in attrs {
            let (name, fmt) = geometry_attribute_info(attr.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
            map.insert(name, (attr.entity, AttributeFormat::from_inner(fmt)));
        }
        Ok(map)
    }

    /// Resolve a built-in attribute name to its `Attribute`.
    fn builtin_attribute(name: &str) -> Option<Attribute> {
        Some(match name {
            "position" => Attribute::position(),
            "velocity" => Attribute::velocity(),
            "normal" => Attribute::normal(),
            "color" => Attribute::color(),
            "uv" => Attribute::uv(),
            "rotation" => Attribute::rotation(),
            "scale" => Attribute::scale(),
            "life" => Attribute::life(),
            "age" => Attribute::age(),
            _ => return None,
        })
    }

    /// Resolve a `buffer()` argument (an attribute name string or an `Attribute`)
    /// to its attribute entity. Built-in names map to their factory; other names
    /// must have been declared as custom attributes at construction.
    fn resolve_attribute(&self, attribute: &Bound<'_, PyAny>) -> PyResult<Entity> {
        if let Ok(attr) = attribute.extract::<Attribute>() {
            return Ok(attr.entity);
        }
        if let Ok(name) = attribute.extract::<String>() {
            if let Some(attr) = Self::builtin_attribute(&name) {
                return Ok(attr.entity);
            }
            if let Some((entity, _)) = self.name_to_attr.get(&name) {
                return Ok(*entity);
            }
            return Err(PyValueError::new_err(format!(
                "\"{name}\" is not a built-in attribute; pass its Attribute to buffer()"
            )));
        }
        Err(PyTypeError::new_err(
            "buffer() expects an attribute name or an Attribute",
        ))
    }

    /// Build a particle system (backs `create_particles`). Attributes default to
    /// `position`; the rest (built-in or declared custom) materialize on demand.
    pub(crate) fn create(
        capacity: Option<u32>,
        attributes: Option<Vec<PyRef<Attribute>>>,
        geometry: Option<&Geometry>,
    ) -> PyResult<Self> {
        let attrs: Vec<Attribute> = match attributes {
            Some(list) => list.iter().map(|a| (**a).clone()).collect(),
            None => vec![Attribute::position()],
        };
        let attr_entities: Vec<Entity> = attrs.iter().map(|a| a.entity).collect();

        let entity = match (capacity, geometry) {
            (Some(cap), None) => particles_create(cap, attr_entities)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
            (None, Some(g)) => particles_create_from_geometry(g.entity, attr_entities)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?,
            (None, None) => {
                return Err(PyRuntimeError::new_err(
                    "create_particles() requires either capacity or geometry",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(PyRuntimeError::new_err(
                    "create_particles() accepts capacity or geometry, not both",
                ));
            }
        };

        Ok(Self {
            entity,
            name_to_attr: Particles::build_name_index(&attrs)?,
        })
    }
}

#[pymethods]
impl Particles {
    #[getter]
    pub fn capacity(&self) -> PyResult<u32> {
        particles_capacity(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (attribute, default=None))]
    pub fn add_attribute(
        &mut self,
        attribute: PyRef<Attribute>,
        default: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let default_value = default
            .map(crate::material::py_to_shader_value)
            .transpose()?;
        particles_attribute_add(self.entity, attribute.entity, default_value)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let (name, fmt) = geometry_attribute_info(attribute.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        self.name_to_attr
            .insert(name, (attribute.entity, AttributeFormat::from_inner(fmt)));
        Ok(())
    }

    /// The GPU buffer for an attribute, materialized on demand. `attribute` is a
    /// built-in name (`"position"`, `"velocity"`, `"color"`, `"scale"`, `"life"`,
    /// `"age"`, `"normal"`, `"uv"`, `"rotation"`), a declared custom attribute's
    /// name, or an `Attribute`.
    pub fn buffer(&self, attribute: &Bound<'_, PyAny>) -> PyResult<Buffer> {
        let attr_entity = self.resolve_attribute(attribute)?;
        let buf = particles_ensure_attribute(self.entity, attr_entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let (_, fmt) = geometry_attribute_info(attr_entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let element_type = match AttributeFormat::from_inner(fmt) {
            AttributeFormat::Float => shader_value::ShaderValue::Float(0.0),
            AttributeFormat::Float2 => shader_value::ShaderValue::Float2([0.0; 2]),
            AttributeFormat::Float3 => shader_value::ShaderValue::Float3([0.0; 3]),
            AttributeFormat::Float4 => shader_value::ShaderValue::Float4([0.0; 4]),
        };
        Ok(Buffer::from_entity(buf, Some(element_type)))
    }

    #[pyo3(signature = (compute, **kwargs))]
    pub fn apply(&self, compute: &Compute, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        if let Some(kwargs) = kwargs {
            compute.set(Some(kwargs))?;
        }
        particles_apply(self.entity, compute.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

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
        particles_emit(self.entity, n, data).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn emit_gpu(&self, n: u32, compute: &Compute) -> PyResult<()> {
        particles_emit_gpu(self.entity, n, compute.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[staticmethod]
    pub fn noise() -> PyResult<Compute> {
        let entity =
            particles_kernel_noise().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn transform() -> PyResult<Compute> {
        let entity =
            particles_kernel_transform().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn attract() -> PyResult<Compute> {
        let entity =
            particles_kernel_attract().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn drag() -> PyResult<Compute> {
        let entity =
            particles_kernel_drag().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn vortex() -> PyResult<Compute> {
        let entity =
            particles_kernel_vortex().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn force() -> PyResult<Compute> {
        let entity =
            particles_kernel_force().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn integrate() -> PyResult<Compute> {
        let entity =
            particles_kernel_integrate().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn age() -> PyResult<Compute> {
        let entity = particles_kernel_age().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn bounds_sphere() -> PyResult<Compute> {
        let entity = particles_kernel_bounds_sphere()
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn bounds_box() -> PyResult<Compute> {
        let entity =
            particles_kernel_bounds_box().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn bounds_geometry(geometry: &Geometry) -> PyResult<Compute> {
        let entity = particles_kernel_bounds_geometry(geometry.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn impulse() -> PyResult<Compute> {
        let entity =
            particles_kernel_impulse().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn flock() -> PyResult<Compute> {
        let entity =
            particles_kernel_flock().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn orient() -> PyResult<Compute> {
        let entity =
            particles_kernel_orient().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn field() -> PyResult<Compute> {
        let entity =
            particles_kernel_field().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn attr_linear() -> PyResult<Compute> {
        let entity =
            particles_kernel_attr_linear().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn attr_combine() -> PyResult<Compute> {
        let entity =
            particles_kernel_attr_combine().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn attr_mix() -> PyResult<Compute> {
        let entity =
            particles_kernel_attr_mix().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn attr_lookup1d() -> PyResult<Compute> {
        let entity = particles_kernel_attr_lookup1d()
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn attr_lookup2d() -> PyResult<Compute> {
        let entity = particles_kernel_attr_lookup2d()
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn scatter_surface(geometry: &Geometry) -> PyResult<Compute> {
        let entity = particles_scatter_create(geometry.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }

    #[staticmethod]
    pub fn scatter_volume(geometry: &Geometry) -> PyResult<Compute> {
        let entity = particles_scatter_volume_create(geometry.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Compute::from_entity(entity))
    }
}

impl Drop for Particles {
    fn drop(&mut self) {
        let _ = particles_destroy(self.entity);
    }
}
