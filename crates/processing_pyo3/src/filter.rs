use processing::prelude::*;
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{PyDict, PyTuple},
};

// Must match the `#[pymodule_export] const` values in lib.rs.
pub const INVERT_U8: u8 = 0;
pub const GRAY_U8: u8 = 1;
pub const THRESHOLD_U8: u8 = 2;
pub const POSTERIZE_U8: u8 = 3;
pub const OPAQUE_U8: u8 = 5;

pub fn parse_filter_op(
    kind: &Bound<'_, PyAny>,
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<FilterOp> {
    let Ok(kind_u8) = kind.extract::<u8>() else {
        return Err(PyTypeError::new_err(
            "filter(): first argument must be a filter constant (INVERT, GRAY, ...)",
        ));
    };

    let kind = match kind_u8 {
        INVERT_U8 => {
            reject_params(args, kwargs, "INVERT")?;
            FilterKind::Invert
        }
        GRAY_U8 => {
            reject_params(args, kwargs, "GRAY")?;
            FilterKind::Gray
        }
        OPAQUE_U8 => {
            reject_params(args, kwargs, "OPAQUE")?;
            FilterKind::Opaque
        }
        THRESHOLD_U8 => {
            let cutoff = parse_scalar(args, kwargs, "THRESHOLD", "cutoff", Some(0.5))?;
            FilterKind::Threshold { cutoff }
        }
        POSTERIZE_U8 => {
            let levels_f = parse_scalar(args, kwargs, "POSTERIZE", "levels", None)?;
            if !(2.0..=255.0).contains(&levels_f) || levels_f.fract() != 0.0 {
                return Err(PyValueError::new_err(
                    "filter(POSTERIZE, levels): levels must be an integer in 2..=255",
                ));
            }
            FilterKind::Posterize {
                levels: levels_f as u32,
            }
        }
        n => {
            return Err(PyValueError::new_err(format!(
                "filter(): unknown or unimplemented filter constant {n}"
            )));
        }
    };

    Ok(FilterOp::new(kind))
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
