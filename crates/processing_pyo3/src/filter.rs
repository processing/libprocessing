use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError, PyValueError},
    prelude::*,
    types::{PyDict, PyTuple},
};
use shader_value::ShaderValue;

use crate::material::py_to_shader_value;
use crate::shader::Shader;

pub const INVERT_U8: u8 = 0;
pub const GRAY_U8: u8 = 1;
pub const THRESHOLD_U8: u8 = 2;
pub const POSTERIZE_U8: u8 = 3;
pub const BLUR_U8: u8 = 4;
pub const OPAQUE_U8: u8 = 5;
pub const ERODE_U8: u8 = 6;
pub const DILATE_U8: u8 = 7;

fn resolve_user_filter(
    shader_entity: Entity,
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Entity> {
    if !args.is_empty() {
        return Err(PyValueError::new_err(
            "filter(shader): parameters must be passed as keyword arguments",
        ));
    }
    let entity =
        filter_create(shader_entity).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    if let Some(kwargs) = kwargs {
        for (key, value) in kwargs.iter() {
            let name: String = key.extract()?;
            if name == "passes" {
                let passes: u32 = value.extract()?;
                filter_set_passes(entity, passes)
                    .map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
                continue;
            }
            let sv = py_to_shader_value(&value)?;
            filter_set(entity, &name, sv).map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
        }
    }
    Ok(entity)
}

fn create_builtin(kind: u8) -> PyResult<Entity> {
    let result = match kind {
        INVERT_U8 => filter_invert(),
        GRAY_U8 => filter_gray(),
        THRESHOLD_U8 => filter_threshold(),
        POSTERIZE_U8 => filter_posterize(),
        BLUR_U8 => filter_blur(),
        OPAQUE_U8 => filter_opaque(),
        ERODE_U8 => filter_erode(),
        DILATE_U8 => filter_dilate(),
        n => {
            return Err(PyValueError::new_err(format!(
                "filter(): unknown or unimplemented filter constant {n}"
            )));
        }
    };
    result.map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn resolve_filter(
    kind: &Bound<'_, PyAny>,
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Entity> {
    if let Ok(shader) = kind.extract::<PyRef<Shader>>() {
        return resolve_user_filter(shader.entity, args, kwargs);
    }

    let Ok(kind_u8) = kind.extract::<u8>() else {
        return Err(PyTypeError::new_err(
            "filter(): first argument must be a filter constant (INVERT, GRAY, ...) or a Shader",
        ));
    };
    let entity = create_builtin(kind_u8)?;
    let set = |name: &str, value: ShaderValue| {
        filter_set(entity, name, value).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    };

    match kind_u8 {
        INVERT_U8 => reject_params(args, kwargs, "INVERT")?,
        GRAY_U8 => reject_params(args, kwargs, "GRAY")?,
        OPAQUE_U8 => reject_params(args, kwargs, "OPAQUE")?,
        ERODE_U8 => reject_params(args, kwargs, "ERODE")?,
        DILATE_U8 => reject_params(args, kwargs, "DILATE")?,
        THRESHOLD_U8 => {
            let cutoff = parse_scalar(args, kwargs, "THRESHOLD", "cutoff", Some(0.5))?;
            set("cutoff", ShaderValue::Float(cutoff))?;
        }
        POSTERIZE_U8 => {
            let levels_f = parse_scalar(args, kwargs, "POSTERIZE", "levels", None)?;
            if !(2.0..=255.0).contains(&levels_f) || levels_f.fract() != 0.0 {
                return Err(PyValueError::new_err(
                    "filter(POSTERIZE, levels): levels must be an integer in 2..=255",
                ));
            }
            set("levels", ShaderValue::UInt(levels_f as u32))?;
        }
        BLUR_U8 => {
            let radius = parse_scalar(args, kwargs, "BLUR", "radius", Some(1.0))?;
            set("radius", ShaderValue::Float(radius))?;
        }
        _ => {}
    }

    Ok(entity)
}

fn reject_params(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
    name: &str,
) -> PyResult<()> {
    if args.len() > 0 {
        return Err(PyValueError::new_err(format!(
            "filter({name}): takes no parameters, got {} positional",
            args.len()
        )));
    }
    if let Some(kw) = kwargs
        && !kw.is_empty()
    {
        return Err(PyValueError::new_err(format!(
            "filter({name}): takes no parameters"
        )));
    }
    Ok(())
}

fn parse_scalar(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
    filter_name: &str,
    kw_name: &str,
    default: Option<f32>,
) -> PyResult<f32> {
    if args.len() > 1 {
        return Err(PyValueError::new_err(format!(
            "filter({filter_name}): expected at most 1 positional arg, got {}",
            args.len()
        )));
    }
    if let Some(kw) = kwargs {
        for key in kw.keys().iter() {
            let k_str = key.str().map(|s| s.to_string()).unwrap_or_default();
            if k_str != kw_name {
                return Err(PyValueError::new_err(format!(
                    "filter({filter_name}): unknown keyword arg `{k_str}` (expected `{kw_name}`)"
                )));
            }
        }
    }

    let from_kw = kwargs
        .and_then(|kw| kw.get_item(kw_name).ok().flatten())
        .map(|v| v.extract::<f32>())
        .transpose()?;

    let from_pos = if args.len() > 0 {
        Some(args.get_item(0)?.extract::<f32>()?)
    } else {
        None
    };

    match (from_pos, from_kw) {
        (Some(_), Some(_)) => Err(PyValueError::new_err(format!(
            "filter({filter_name}): got both positional and keyword `{kw_name}`"
        ))),
        (Some(v), None) | (None, Some(v)) => Ok(v),
        (None, None) => default.ok_or_else(|| {
            PyValueError::new_err(format!(
                "filter({filter_name}): missing required parameter `{kw_name}`"
            ))
        }),
    }
}
