//! Dynamic topology: primitives added by kernels via `processing::prims`.
//!
//! A primitives target owns an internal particle field (the expanded
//! vertices — unique by construction, so no index buffer) plus a widened
//! indirect-args buffer whose head is the GPU-read `DrawIndirect` arguments
//! and whose tail carries stats the sketch can poll. `apply` binds the
//! target's buffers under the `processing::prims` reserved names, resets the
//! counts on the first apply of each frame, and dispatches the kernel over
//! the source field. Drawing reuses the raster connectivity seam with
//! `index_buffer: None` (non-indexed `draw_indirect`).

use bevy::prelude::*;
use bevy::render::render_resource::BufferUsages;
use processing_core::{app_mut, error};

use crate::geometry::Topology;
use crate::particles::{Connectivity, Particles};
use crate::shader_value::ShaderValue;

/// Layout of the args buffer, in u32 words. The first four words are the
/// non-indexed `DrawIndirect` arguments; the tail is stats + configuration
/// for `processing::prims` (see `shaders/processing/prims.wesl`).
const ARGS_WORDS: usize = 8;
const ARGS_ATTEMPTED_OFFSET: u64 = 16;

/// The reserved binding names declared by `processing::prims`. The WESL
/// compiler mangles imported module bindings, so these are matched by
/// suffix against the kernel's reflected parameter names.
const RESERVED_POSITION: &str = "prims_out_position";
const RESERVED_COLOR: &str = "prims_out_color";
const RESERVED_ARGS: &str = "prims_args";

#[derive(Component)]
pub struct PrimitivesTarget {
    /// The internal particle field holding the expanded vertices.
    pub field: Entity,
    /// The source field the kernels dispatch over.
    pub source: Entity,
    pub topology: Topology,
    pub capacity_prims: u32,
    pub verts_per_prim: u32,
    /// The widened indirect-args buffer (also the field's connectivity).
    pub args: Entity,
}

fn verts_per_prim(topology: Topology) -> error::Result<u32> {
    match topology {
        Topology::PointList => Ok(1),
        Topology::LineList => Ok(2),
        Topology::TriangleList => Ok(3),
        _ => Err(error::ProcessingError::InvalidArgument(format!(
            "primitives targets support POINTS, LINES, and TRIANGLES; \
             strips have shared vertices — use an index buffer instead \
             (got {topology:?})",
        ))),
    }
}

fn u32s_to_bytes(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// Creates a primitives target over `source`. Backs `p.primitives(...)`.
pub fn particles_primitives_create(
    source: Entity,
    topology: Topology,
    capacity_prims: u32,
) -> error::Result<Entity> {
    let vpp = verts_per_prim(topology)?;
    let capacity_verts = capacity_prims
        .checked_mul(vpp)
        .ok_or_else(|| error::ProcessingError::InvalidArgument("capacity overflow".to_string()))?;

    let position = crate::geometry_attribute_position();
    let color = crate::geometry_attribute_color();
    let field = crate::particles_create(capacity_verts, vec![position, color])?;

    let args = crate::buffer_create_with_usage(
        ARGS_WORDS as u64 * 4,
        BufferUsages::INDIRECT | BufferUsages::STORAGE,
    )?;
    crate::buffer_write(
        args,
        u32s_to_bytes(&[0, 1, 0, 0, 0, capacity_verts, vpp, 0]),
    )?;

    app_mut(|app| {
        let mut field_data = app
            .world_mut()
            .get_mut::<Particles>(field)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        field_data.connectivity = Some(Connectivity {
            index_buffer: None,
            indirect_buffer: args,
        });
        Ok(app
            .world_mut()
            .spawn(PrimitivesTarget {
                field,
                source,
                topology,
                capacity_prims,
                verts_per_prim: vpp,
                args,
            })
            .id())
    })
}

/// Resolves a reserved `processing::prims` binding to its mangled reflected
/// name in the kernel, then binds the buffer through the normal path.
fn set_reserved_buffer(
    compute_entity: Entity,
    reserved: &str,
    buffer_entity: Entity,
) -> error::Result<()> {
    let actual = app_mut(|app| {
        let compute = app
            .world()
            .get::<crate::compute::Compute>(compute_entity)
            .ok_or(error::ProcessingError::ComputeNotFound)?;
        let reflection = compute.shader.reflection();
        let name = reflection
            .parameters()
            .filter_map(|p| p.name().map(String::from))
            .find(|n| n == reserved || n.ends_with(reserved) || n.contains(reserved));
        Ok(name)
    })?;
    let Some(actual) = actual else {
        return Err(error::ProcessingError::InvalidArgument(format!(
            "kernel does not import `processing::prims` (no `{reserved}` binding); \
             add `import {{ processing::prims }};` to add primitives from a kernel",
        )));
    };
    crate::compute_set(compute_entity, actual, ShaderValue::Buffer(buffer_entity))
}

/// Applies `kernel` over the target's source field with the target's buffers
/// bound under the `processing::prims` reserved names. The first apply after
/// the target was drawn resets the primitive count; further applies before
/// the next draw append.
pub fn particles_primitives_apply(
    target_entity: Entity,
    compute_entity: Entity,
) -> error::Result<()> {
    let (field, source, args, drawn) = app_mut(|app| {
        let target = app
            .world()
            .get::<PrimitivesTarget>(target_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        let (field, source, args) = (target.field, target.source, target.args);
        let mut field_data = app
            .world_mut()
            .get_mut::<Particles>(field)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        let drawn = std::mem::take(&mut field_data.drawn_since_apply);
        Ok((field, source, args, drawn))
    })?;

    if drawn {
        // Reset draw count + attempted; instance_count stays 1, and the
        // capacity/verts-per-prim tail is untouched.
        crate::buffer_write_element(args, 0, u32s_to_bytes(&[0, 1, 0, 0, 0]))?;
    }

    let (position_buf, color_buf) = app_mut(|app| {
        let world = app.world();
        let attrs = world.resource::<crate::geometry::attribute::BuiltinAttributes>();
        let (position_attr, color_attr) = (attrs.position, attrs.color);
        let field_data = world
            .get::<Particles>(field)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        let position = field_data
            .buffer(position_attr)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        let color = field_data
            .buffer(color_attr)
            .ok_or(error::ProcessingError::ParticlesNotFound)?;
        Ok((position, color))
    })?;

    set_reserved_buffer(compute_entity, RESERVED_POSITION, position_buf)?;
    set_reserved_buffer(compute_entity, RESERVED_COLOR, color_buf)?;
    set_reserved_buffer(compute_entity, RESERVED_ARGS, args)?;

    crate::particles::particles_apply(source, compute_entity)
}

/// The internal particle field holding a target's expanded vertices (what
/// `particles(target)` draws).
pub fn particles_primitives_field(target_entity: Entity) -> error::Result<Entity> {
    app_mut(|app| {
        Ok(app
            .world()
            .get::<PrimitivesTarget>(target_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?
            .field)
    })
}

/// Primitives the kernels attempted to add this frame (including any dropped
/// for capacity). Reads a 4-byte stat from the GPU; call it when you want
/// the number, not every frame.
pub fn particles_primitives_attempted(target_entity: Entity) -> error::Result<u32> {
    let args = app_mut(|app| {
        Ok(app
            .world()
            .get::<PrimitivesTarget>(target_entity)
            .ok_or(error::ProcessingError::ParticlesNotFound)?
            .args)
    })?;
    let bytes = crate::buffer_read_element(args, ARGS_ATTEMPTED_OFFSET, 4)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}
