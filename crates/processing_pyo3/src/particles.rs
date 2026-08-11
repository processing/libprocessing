use bevy::prelude::Entity;
use processing::prelude::*;
use processing_render::geometry;
use pyo3::types::PyDict;
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError, PyValueError},
    prelude::*,
};
use std::collections::HashMap;

use processing_render::particles::algebra::{
    combine as algebra_combine, extract as algebra_extract, generate as algebra_generate,
    lookup as algebra_lookup, map as algebra_map, mix as algebra_mix, pack as algebra_pack,
    reduce_components as algebra_reduce,
};
use processing_render::particles::compact::compact as compact_indices;

use crate::compute::{Buffer, Compute};
use crate::graphics::{Geometry, Image};

use processing::prelude::constants as c;
use processing_render::{
    COMBINE_ADD, COMBINE_DIV, COMBINE_MAX, COMBINE_MIN, COMBINE_MUL, COMBINE_POW, COMBINE_SUB,
};
use processing_render::{GEN_GAUSSIAN, GEN_SIGNED, GEN_UNIFORM};
use processing_render::{
    MAP_ABS, MAP_AFFINE, MAP_CLAMP, MAP_EQ, MAP_FLOOR, MAP_GEQ, MAP_GREATER, MAP_LEQ, MAP_LESS,
    MAP_NEGATE, MAP_NEQ, MAP_SQRT, MAP_SQUARE,
};
use processing_render::{
    REDUCE_LENGTH, REDUCE_MAX, REDUCE_MEAN, REDUCE_MIN, REDUCE_SUM, REDUCE_SUMSQ,
};

fn parse_map_op(s: &str) -> PyResult<u32> {
    match () {
        _ if s.eq_ignore_ascii_case(c::AFFINE) => Ok(MAP_AFFINE),
        _ if s.eq_ignore_ascii_case(c::ABS) => Ok(MAP_ABS),
        _ if s.eq_ignore_ascii_case(c::NEGATE) => Ok(MAP_NEGATE),
        _ if s.eq_ignore_ascii_case(c::CLAMP) => Ok(MAP_CLAMP),
        _ if s.eq_ignore_ascii_case(c::FLOOR) => Ok(MAP_FLOOR),
        _ if s.eq_ignore_ascii_case(c::SQUARE) => Ok(MAP_SQUARE),
        _ if s.eq_ignore_ascii_case(c::SQRT) => Ok(MAP_SQRT),
        _ if s.eq_ignore_ascii_case(c::GREATER) => Ok(MAP_GREATER),
        _ if s.eq_ignore_ascii_case(c::LESS) => Ok(MAP_LESS),
        _ if s.eq_ignore_ascii_case(c::GEQ) => Ok(MAP_GEQ),
        _ if s.eq_ignore_ascii_case(c::LEQ) => Ok(MAP_LEQ),
        _ if s.eq_ignore_ascii_case(c::EQ) => Ok(MAP_EQ),
        _ if s.eq_ignore_ascii_case(c::NEQ) => Ok(MAP_NEQ),
        _ => Err(PyValueError::new_err(format!("map: unknown op {s:?}"))),
    }
}

fn parse_combine_op(s: &str) -> PyResult<u32> {
    match () {
        _ if s.eq_ignore_ascii_case(c::ADD) => Ok(COMBINE_ADD),
        _ if s.eq_ignore_ascii_case(c::SUB) => Ok(COMBINE_SUB),
        _ if s.eq_ignore_ascii_case(c::MUL) => Ok(COMBINE_MUL),
        _ if s.eq_ignore_ascii_case(c::DIV) => Ok(COMBINE_DIV),
        _ if s.eq_ignore_ascii_case(c::MIN) => Ok(COMBINE_MIN),
        _ if s.eq_ignore_ascii_case(c::MAX) => Ok(COMBINE_MAX),
        _ if s.eq_ignore_ascii_case(c::POW) => Ok(COMBINE_POW),
        _ => Err(PyValueError::new_err(format!("combine: unknown op {s:?}"))),
    }
}

fn parse_reduce_op(s: &str) -> PyResult<u32> {
    match () {
        _ if s.eq_ignore_ascii_case(c::LENGTH) => Ok(REDUCE_LENGTH),
        _ if s.eq_ignore_ascii_case(c::SUM) => Ok(REDUCE_SUM),
        _ if s.eq_ignore_ascii_case(c::MIN) => Ok(REDUCE_MIN),
        _ if s.eq_ignore_ascii_case(c::MAX) => Ok(REDUCE_MAX),
        _ if s.eq_ignore_ascii_case(c::SUMSQ) => Ok(REDUCE_SUMSQ),
        _ if s.eq_ignore_ascii_case(c::MEAN) => Ok(REDUCE_MEAN),
        _ => Err(PyValueError::new_err(format!("reduce: unknown op {s:?}"))),
    }
}

fn parse_generate_mode(s: &str) -> PyResult<u32> {
    match () {
        _ if s.eq_ignore_ascii_case(c::UNIFORM) => Ok(GEN_UNIFORM),
        _ if s.eq_ignore_ascii_case(c::SIGNED) => Ok(GEN_SIGNED),
        _ if s.eq_ignore_ascii_case(c::GAUSSIAN) => Ok(GEN_GAUSSIAN),
        _ => Err(PyValueError::new_err(format!(
            "generate: unknown mode {s:?}"
        ))),
    }
}

const NEIGHBOR_SUM: u32 = 0;
const NEIGHBOR_MEAN: u32 = 1;
const NEIGHBOR_COUNT: u32 = 2;

fn parse_neighbor_op(s: &str) -> PyResult<u32> {
    match () {
        _ if s.eq_ignore_ascii_case(c::SUM) => Ok(NEIGHBOR_SUM),
        _ if s.eq_ignore_ascii_case(c::MEAN) => Ok(NEIGHBOR_MEAN),
        _ if s.eq_ignore_ascii_case(c::COUNT) || s.eq_ignore_ascii_case(c::DENSITY) => {
            Ok(NEIGHBOR_COUNT)
        }
        _ => Err(PyValueError::new_err(format!("neighbor: unknown op {s:?}"))),
    }
}

fn parse_falloff(s: &str) -> PyResult<u32> {
    match () {
        _ if s.eq_ignore_ascii_case(c::CONSTANT) => Ok(FALLOFF_CONST),
        _ if s.eq_ignore_ascii_case(c::LINEAR) => Ok(FALLOFF_LINEAR),
        _ if s.eq_ignore_ascii_case(c::SMOOTHSTEP) => Ok(FALLOFF_SMOOTHSTEP),
        _ if s.eq_ignore_ascii_case(c::QUADRATIC) => Ok(FALLOFF_QUADRATIC),
        _ if s.eq_ignore_ascii_case(c::CUBIC) => Ok(FALLOFF_CUBIC),
        _ if s.eq_ignore_ascii_case(c::INVERSE) => Ok(FALLOFF_INVERSE),
        _ => Err(PyValueError::new_err(format!(
            "neighbor: unknown falloff {s:?}"
        ))),
    }
}

#[pyclass(unsendable)]
pub struct Grid {
    pub(crate) inner: processing_render::particles::grid::Grid,
}

#[pymethods]
impl Grid {
    pub fn build(&self, position: &Buffer) -> PyResult<()> {
        grid_build(&self.inner, position.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    pub fn bind(&self, compute: &Compute) -> PyResult<()> {
        grid_bind(&self.inner, compute.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[getter]
    pub fn cell_size(&self) -> f32 {
        self.inner.params.cell_size
    }
}

static FLOCK_COMPUTE: std::sync::Mutex<Option<Entity>> = std::sync::Mutex::new(None);

fn flock_compute() -> PyResult<Entity> {
    let mut guard = FLOCK_COMPUTE.lock().unwrap();
    if let Some(e) = *guard {
        return Ok(e);
    }
    let e = particles_kernel_flock().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    *guard = Some(e);
    Ok(e)
}

static PHYSICS_COMPUTES: std::sync::Mutex<Option<HashMap<String, Entity>>> =
    std::sync::Mutex::new(None);

fn physics_compute(name: &str) -> PyResult<Option<Entity>> {
    let lower = name.to_ascii_lowercase();
    if let Some(cache) = PHYSICS_COMPUTES.lock().unwrap().as_ref() {
        if let Some(&e) = cache.get(&lower) {
            return Ok(Some(e));
        }
    }
    let created = if lower == c::NOISE {
        particles_kernel_noise()
    } else if lower == c::TRANSFORM {
        particles_kernel_transform()
    } else if lower == c::ATTRACT {
        particles_kernel_attract()
    } else if lower == c::DRAG {
        particles_kernel_drag()
    } else if lower == c::VORTEX {
        particles_kernel_vortex()
    } else if lower == c::FORCE {
        particles_kernel_force()
    } else if lower == c::INTEGRATE {
        particles_kernel_integrate()
    } else if lower == c::AGE {
        particles_kernel_age()
    } else if lower == c::IMPULSE {
        particles_kernel_impulse()
    } else if lower == c::ORIENT {
        particles_kernel_orient()
    } else if lower == c::FIELD {
        particles_kernel_field()
    } else if lower == c::BOUNDS_SPHERE {
        particles_kernel_bounds_sphere()
    } else if lower == c::BOUNDS_BOX {
        particles_kernel_bounds_box()
    } else {
        return Ok(None);
    };
    let entity = created.map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    PHYSICS_COMPUTES
        .lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(lower, entity);
    Ok(Some(entity))
}

fn kw<'a>(kwargs: Option<&Bound<'a, PyDict>>, key: &str) -> Option<Bound<'a, PyAny>> {
    kwargs.and_then(|d| d.get_item(key).ok().flatten())
}

fn kw_f32(kwargs: Option<&Bound<'_, PyDict>>, key: &str, default: f32) -> PyResult<f32> {
    match kw(kwargs, key) {
        Some(v) => v.extract(),
        None => Ok(default),
    }
}

fn kw_u32(kwargs: Option<&Bound<'_, PyDict>>, key: &str, default: u32) -> PyResult<u32> {
    match kw(kwargs, key) {
        Some(v) => v.extract(),
        None => Ok(default),
    }
}

fn kw_bool(kwargs: Option<&Bound<'_, PyDict>>, key: &str, default: bool) -> PyResult<bool> {
    match kw(kwargs, key) {
        Some(v) => v.extract(),
        None => Ok(default),
    }
}

fn map_params(kwargs: Option<&Bound<'_, PyDict>>, op: u32) -> PyResult<(f32, f32)> {
    Ok(match op {
        MAP_AFFINE => (
            kw_f32(kwargs, "scale", 1.0)?,
            kw_f32(kwargs, "offset", 0.0)?,
        ),
        MAP_CLAMP => (kw_f32(kwargs, "lo", 0.0)?, kw_f32(kwargs, "hi", 1.0)?),
        MAP_GREATER | MAP_LESS | MAP_GEQ | MAP_LEQ | MAP_EQ | MAP_NEQ => (
            kw_f32(kwargs, "threshold", 0.0)?,
            kw_f32(kwargs, "epsilon", 1.0e-6)?,
        ),
        _ => (0.0, 0.0),
    })
}

fn map_param_keys(op: u32) -> &'static [&'static str] {
    match op {
        MAP_AFFINE => &["scale", "offset"],
        MAP_CLAMP => &["lo", "hi"],
        MAP_GREATER | MAP_LESS | MAP_GEQ | MAP_LEQ | MAP_EQ | MAP_NEQ => &["threshold", "epsilon"],
        _ => &[],
    }
}

fn reject_unknown_kwargs(kwargs: Option<&Bound<'_, PyDict>>, valid: &[&str]) -> PyResult<()> {
    let Some(kwargs) = kwargs else {
        return Ok(());
    };
    for key in kwargs.keys() {
        let name: String = key.extract()?;
        if !valid.iter().any(|v| *v == name) {
            return Err(PyValueError::new_err(format!(
                "apply(): unknown parameter {name:?} (valid: {})",
                valid.join(", ")
            )));
        }
    }
    Ok(())
}

fn kw_op(
    kwargs: Option<&Bound<'_, PyDict>>,
    default: u32,
    parse: fn(&str) -> PyResult<u32>,
) -> PyResult<u32> {
    match kw(kwargs, "op") {
        Some(v) => parse(&v.extract::<String>()?),
        None => Ok(default),
    }
}

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

    fn resolve_operand(&self, val: &Bound<'_, PyAny>) -> PyResult<(Entity, u32)> {
        if let Ok(b) = val.extract::<PyRef<Buffer>>() {
            let comp = b.components().ok_or_else(|| {
                PyRuntimeError::new_err(
                    "operand buffer has no element type; pass a particle attribute \
                     or a buffer created with typed data",
                )
            })?;
            return Ok((b.entity, comp));
        }
        let attr_entity = self.resolve_attribute(val)?;
        let buf = particles_ensure_attribute(self.entity, attr_entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let (_, fmt) = geometry_attribute_info(attr_entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let comp = match AttributeFormat::from_inner(fmt) {
            AttributeFormat::Float => 1,
            AttributeFormat::Float2 => 2,
            AttributeFormat::Float3 => 3,
            AttributeFormat::Float4 => 4,
        };
        Ok((buf, comp))
    }

    fn operand(&self, kwargs: Option<&Bound<'_, PyDict>>, key: &str) -> PyResult<(Entity, u32)> {
        let val = kw(kwargs, key)
            .ok_or_else(|| PyRuntimeError::new_err(format!("apply(): missing operand '{key}'")))?;
        self.resolve_operand(&val)
    }

    fn dest(&self, kwargs: Option<&Bound<'_, PyDict>>, in_place: Entity) -> PyResult<Entity> {
        match kw(kwargs, "out") {
            Some(v) => Ok(self.resolve_operand(&v)?.0),
            None => Ok(in_place),
        }
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

    fn apply_named(&self, name: &str, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        fn rt(e: impl std::fmt::Display) -> PyErr {
            PyRuntimeError::new_err(format!("{e}"))
        }

        if let Some(entity) = physics_compute(name)? {
            if let Some(kwargs) = kwargs {
                crate::compute::set_compute_kwargs(entity, kwargs)?;
            }
            return particles_apply(self.entity, entity).map_err(rt);
        }

        if name.eq_ignore_ascii_case(c::MAP) {
            let (a, comp) = self.operand(kwargs, "a")?;
            let out = self.dest(kwargs, a)?;
            let op = kw_op(kwargs, MAP_AFFINE, parse_map_op)?;
            let mut valid = vec!["a", "out", "op"];
            valid.extend_from_slice(map_param_keys(op));
            reject_unknown_kwargs(kwargs, &valid)?;
            let (p0, p1) = map_params(kwargs, op)?;
            algebra_map(out, a, comp, op, p0, p1).map_err(rt)
        } else if name.eq_ignore_ascii_case(c::COMBINE) {
            reject_unknown_kwargs(kwargs, &["a", "b", "out", "op", "b_scale", "b_offset"])?;
            let (a, comp) = self.operand(kwargs, "a")?;
            let (b, b_comp) = self.operand(kwargs, "b")?;
            if b_comp != comp {
                return Err(PyValueError::new_err(format!(
                    "apply(combine): `a` has {comp} components but `b` has {b_comp} (must match)"
                )));
            }
            let out = self.dest(kwargs, a)?;
            let op = kw_op(kwargs, COMBINE_ADD, parse_combine_op)?;
            let b_scale = kw_f32(kwargs, "b_scale", 1.0)?;
            let b_offset = kw_f32(kwargs, "b_offset", 0.0)?;
            algebra_combine(out, a, b, comp, op, b_scale, b_offset).map_err(rt)
        } else if name.eq_ignore_ascii_case(c::MIX) {
            reject_unknown_kwargs(
                kwargs,
                &["a", "b", "t", "out", "t_scale", "t_offset", "t_clamp"],
            )?;
            let (a, comp) = self.operand(kwargs, "a")?;
            let (b, b_comp) = self.operand(kwargs, "b")?;
            let (t, t_comp) = self.operand(kwargs, "t")?;
            if b_comp != comp {
                return Err(PyValueError::new_err(format!(
                    "apply(mix): `a` has {comp} components but `b` has {b_comp} (must match)"
                )));
            }
            if t_comp != 1 {
                return Err(PyValueError::new_err(format!(
                    "apply(mix): `t` must be a per-particle scalar (1 component), got {t_comp}"
                )));
            }
            let out = self.dest(kwargs, a)?;
            let t_scale = kw_f32(kwargs, "t_scale", 1.0)?;
            let t_offset = kw_f32(kwargs, "t_offset", 0.0)?;
            let t_clamp = kw_bool(kwargs, "t_clamp", true)?;
            algebra_mix(out, a, b, t, comp, t_scale, t_offset, t_clamp).map_err(rt)
        } else if name.eq_ignore_ascii_case(c::LOOKUP) {
            reject_unknown_kwargs(
                kwargs,
                &[
                    "a",
                    "out",
                    "tex",
                    "u_scale",
                    "u_offset",
                    "v_scale",
                    "v_offset",
                    "color_scale",
                ],
            )?;
            let (a, in_comp) = self.operand(kwargs, "a")?;
            let out = self.operand(kwargs, "out")?.0;
            let tex = kw(kwargs, "tex")
                .ok_or_else(|| PyRuntimeError::new_err("apply(lookup): missing 'tex' Image"))?
                .extract::<PyRef<Image>>()
                .map_err(|_| PyRuntimeError::new_err("apply(lookup): 'tex' must be an Image"))?
                .entity;
            let u_scale = kw_f32(kwargs, "u_scale", 1.0)?;
            let u_offset = kw_f32(kwargs, "u_offset", 0.0)?;
            let v_scale = kw_f32(kwargs, "v_scale", 1.0)?;
            let v_offset = kw_f32(kwargs, "v_offset", 0.0)?;
            let color_scale = kw_f32(kwargs, "color_scale", 1.0)?;
            algebra_lookup(
                out,
                a,
                tex,
                in_comp,
                u_scale,
                u_offset,
                v_scale,
                v_offset,
                color_scale,
            )
            .map_err(rt)
        } else if name.eq_ignore_ascii_case(c::REDUCE) {
            reject_unknown_kwargs(kwargs, &["a", "out", "op"])?;
            let (a, comp) = self.operand(kwargs, "a")?;
            let out = self.operand(kwargs, "out")?.0;
            let op = kw_op(kwargs, REDUCE_LENGTH, parse_reduce_op)?;
            algebra_reduce(out, a, comp, op).map_err(rt)
        } else if name.eq_ignore_ascii_case(c::EXTRACT) {
            reject_unknown_kwargs(kwargs, &["a", "out", "index"])?;
            let (a, comp) = self.operand(kwargs, "a")?;
            let out = self.operand(kwargs, "out")?.0;
            let index = kw_u32(kwargs, "index", 0)?;
            algebra_extract(out, a, comp, index).map_err(rt)
        } else if name.eq_ignore_ascii_case(c::PACK) {
            reject_unknown_kwargs(kwargs, &["out", "sources"])?;
            let out = self.operand(kwargs, "out")?.0;
            let sources = kw(kwargs, "sources")
                .ok_or_else(|| PyRuntimeError::new_err("apply(pack): missing 'sources' list"))?;
            let items: Vec<Bound<'_, PyAny>> = sources.extract()?;
            let mut entities = Vec::with_capacity(items.len());
            for item in &items {
                entities.push(self.resolve_operand(item)?.0);
            }
            algebra_pack(out, &entities).map_err(rt)
        } else if name.eq_ignore_ascii_case(c::GENERATE) {
            reject_unknown_kwargs(kwargs, &["out", "mode", "seed", "scale", "offset"])?;
            let (out, comp) = self.operand(kwargs, "out")?;
            let mode = match kw(kwargs, "mode") {
                Some(v) => parse_generate_mode(&v.extract::<String>()?)?,
                None => GEN_UNIFORM,
            };
            let seed = kw_u32(kwargs, "seed", 0)?;
            let scale = kw_f32(kwargs, "scale", 1.0)?;
            let offset = kw_f32(kwargs, "offset", 0.0)?;
            algebra_generate(out, comp, mode, seed, scale, offset).map_err(rt)
        } else if name.eq_ignore_ascii_case(c::NEIGHBOR) {
            reject_unknown_kwargs(kwargs, &["a", "out", "grid", "op", "radius", "falloff"])?;
            let grid = kw(kwargs, "grid")
                .ok_or_else(|| PyRuntimeError::new_err("apply(neighbor): missing 'grid'"))?
                .extract::<PyRef<Grid>>()
                .map_err(|_| PyRuntimeError::new_err("apply(neighbor): 'grid' must be a Grid"))?;
            let op = kw_op(kwargs, NEIGHBOR_MEAN, parse_neighbor_op)?;
            let falloff = match kw(kwargs, "falloff") {
                Some(v) => parse_falloff(&v.extract::<String>()?)?,
                None => FALLOFF_SMOOTHSTEP,
            };
            let cell = grid.inner.params.cell_size;
            let radius = kw_f32(kwargs, "radius", cell)?.min(cell);

            let (out, out_comp) = self.operand(kwargs, "out")?;
            let (a, components) = if op == NEIGHBOR_COUNT {
                if out_comp != 1 {
                    return Err(PyValueError::new_err(
                        "apply(neighbor, op=count/density): `out` must be a scalar (1 component)",
                    ));
                }
                let a = match kw(kwargs, "a") {
                    Some(_) => self.operand(kwargs, "a")?.0,
                    None => {
                        let pos = Self::builtin_attribute("position")
                            .expect("position is a built-in")
                            .entity;
                        particles_ensure_attribute(self.entity, pos).map_err(rt)?
                    }
                };
                (a, 1u32)
            } else {
                let (a, in_comp) = self.operand(kwargs, "a")?;
                if in_comp != out_comp {
                    return Err(PyValueError::new_err(format!(
                        "apply(neighbor): source has {in_comp} components but out has {out_comp}"
                    )));
                }
                (a, in_comp)
            };
            particles_gather(
                self.entity,
                &grid.inner,
                a,
                out,
                op,
                radius,
                falloff,
                components,
            )
            .map_err(rt)
        } else {
            Err(PyValueError::new_err(format!(
                "apply(): unknown operation {name:?}"
            )))
        }
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

    pub fn index_buffer(&self, index_count: u32) -> PyResult<Buffer> {
        let entity = particles_set_connectivity(self.entity, index_count)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Buffer::from_entity(
            entity,
            Some(shader_value::ShaderValue::UInt(0)),
        ))
    }

    pub fn draw_args(&self) -> PyResult<Buffer> {
        let entity = particles_connectivity_indirect(self.entity)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Buffer::from_entity(
            entity,
            Some(shader_value::ShaderValue::UInt(0)),
        ))
    }

    pub fn reset_indices(&self) -> PyResult<()> {
        particles_reset_indices(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    }

    #[pyo3(signature = (kind, **kwargs))]
    pub fn apply(
        &self,
        kind: &Bound<'_, PyAny>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        if let Ok(compute) = kind.extract::<PyRef<Compute>>() {
            if let Some(kwargs) = kwargs {
                compute.set(Some(kwargs))?;
            }
            return particles_apply(self.entity, compute.entity)
                .map_err(|e| PyRuntimeError::new_err(format!("{e}")));
        }
        let name: String = kind.extract().map_err(|_| {
            PyTypeError::new_err(
                "apply(): first argument must be an operation constant or a Compute",
            )
        })?;
        self.apply_named(&name, kwargs)
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

    pub fn compact(&self, flags: &Bound<'_, PyAny>, out: &Buffer) -> PyResult<u32> {
        let (flag_buf, _) = self.resolve_operand(flags)?;
        compact_indices(flag_buf, out.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
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

    pub fn create_grid(&self, min: [f32; 3], cell_size: f32, dims: [u32; 3]) -> PyResult<Grid> {
        let capacity =
            particles_capacity(self.entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        let params = GridParams {
            min,
            cell_size,
            dims,
        };
        let inner =
            grid_create(params, capacity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        Ok(Grid { inner })
    }

    #[pyo3(signature = (grid, **kwargs))]
    pub fn flock(&self, grid: &Grid, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let flock = flock_compute()?;
        if let Some(kwargs) = kwargs {
            crate::compute::set_compute_kwargs(flock, kwargs)?;
        }
        let cell = grid.inner.params.cell_size;
        let neighbor_distance = kw_f32(kwargs, "neighbor_distance", cell)?.min(cell);
        compute_set(
            flock,
            "neighbor_distance",
            shader_value::ShaderValue::Float(neighbor_distance),
        )
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        particles_flock(self.entity, flock, &grid.inner)
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
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
