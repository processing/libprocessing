use std::sync::Mutex;

use bevy::prelude::Entity;

use processing_core::error::{ProcessingError, Result};

use crate::shader_value::ShaderValue;
use crate::{buffer_size, compute_dispatch, compute_set, shader_create_with_features};

const WG: u32 = 64;

pub const MAP_AFFINE: u32 = 0;
pub const MAP_ABS: u32 = 1;
pub const MAP_NEGATE: u32 = 2;
pub const MAP_CLAMP: u32 = 3;
pub const MAP_FLOOR: u32 = 4;
pub const MAP_SQUARE: u32 = 5;
pub const MAP_SQRT: u32 = 6;
pub const MAP_GREATER: u32 = 7;
pub const MAP_LESS: u32 = 8;
pub const MAP_GEQ: u32 = 9;
pub const MAP_LEQ: u32 = 10;
pub const MAP_EQ: u32 = 11;
pub const MAP_NEQ: u32 = 12;

const MAP_SRC: &str = r#"
struct Params {
    components: u32,
    op: u32,
    p0: f32,
    p1: f32,
}

@if(in_place)  @group(0) @binding(0) var<storage, read_write> a:   array<f32>;
@if(!in_place) @group(0) @binding(0) var<storage, read>       a:   array<f32>;
@if(!in_place) @group(0) @binding(1) var<storage, read_write> dst: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

fn apply_op(x: f32) -> f32 {
    switch params.op {
        case 0u: { return x * params.p0 + params.p1; }
        case 1u: { return abs(x); }
        case 2u: { return -x; }
        case 3u: { return clamp(x, params.p0, params.p1); }
        case 4u: { return floor(x); }
        case 5u: { return x * x; }
        case 6u: { return sqrt(max(x, 0.0)); }
        case 7u: { return select(0.0, 1.0, x > params.p0); }
        case 8u: { return select(0.0, 1.0, x < params.p0); }
        case 9u: { return select(0.0, 1.0, x >= params.p0); }
        case 10u: { return select(0.0, 1.0, x <= params.p0); }
        case 11u: { return select(0.0, 1.0, abs(x - params.p0) <= params.p1); }
        case 12u: { return select(0.0, 1.0, abs(x - params.p0) > params.p1); }
        default: { return x; }
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&a) / params.components;
    if i >= n { return; }
    for (var c = 0u; c < params.components; c = c + 1u) {
        let idx = i * params.components + c;
        @if(in_place)  { a[idx] = apply_op(a[idx]); }
        @if(!in_place) { dst[idx] = apply_op(a[idx]); }
    }
}
"#;

struct Variants {
    out_of_place: Entity,
    in_place: Entity,
}

fn variants(cache: &Mutex<Option<(Entity, Entity)>>, src: &str) -> Result<Variants> {
    let mut guard = cache.lock().unwrap();
    if let Some((o, i)) = *guard {
        return Ok(Variants {
            out_of_place: o,
            in_place: i,
        });
    }
    let out_of_place = shader_create_with_features(src, &[("in_place", false)])?;
    let in_place = shader_create_with_features(src, &[("in_place", true)])?;
    let out_of_place = crate::compute_create(out_of_place)?;
    let in_place = crate::compute_create(in_place)?;
    *guard = Some((out_of_place, in_place));
    Ok(Variants {
        out_of_place,
        in_place,
    })
}

static MAP: Mutex<Option<(Entity, Entity)>> = Mutex::new(None);

fn dispatch_particles(compute: Entity, floats: u64, components: u32) -> Result<()> {
    let n = (floats / components as u64) as u32;
    compute_dispatch(compute, n.div_ceil(WG), 1, 1)
}

fn check_components(verb: &str, components: u32) -> Result<()> {
    if components == 0 || components > 4 {
        return Err(ProcessingError::InvalidArgument(format!(
            "{verb}: components must be 1..=4, got {components}"
        )));
    }
    Ok(())
}

fn ensure_no_alias(verb: &str, rw: Entity, reads: &[Entity]) -> Result<()> {
    if reads.contains(&rw) {
        return Err(ProcessingError::InvalidArgument(format!(
            "{verb}: dst aliases a read-only operand; make dst the first operand \
             (in-place) or pass a distinct dst buffer"
        )));
    }
    Ok(())
}

pub fn map(dst: Entity, a: Entity, components: u32, op: u32, p0: f32, p1: f32) -> Result<()> {
    check_components("map", components)?;
    let v = variants(&MAP, MAP_SRC)?;
    let floats = buffer_size(a)? / 4;

    if dst == a {
        let c = v.in_place;
        compute_set(c, "a", ShaderValue::Buffer(a))?;
        compute_set(c, "components", ShaderValue::UInt(components))?;
        compute_set(c, "op", ShaderValue::UInt(op))?;
        compute_set(c, "p0", ShaderValue::Float(p0))?;
        compute_set(c, "p1", ShaderValue::Float(p1))?;
        dispatch_particles(c, floats, components)
    } else {
        let c = v.out_of_place;
        compute_set(c, "a", ShaderValue::Buffer(a))?;
        compute_set(c, "dst", ShaderValue::Buffer(dst))?;
        compute_set(c, "components", ShaderValue::UInt(components))?;
        compute_set(c, "op", ShaderValue::UInt(op))?;
        compute_set(c, "p0", ShaderValue::Float(p0))?;
        compute_set(c, "p1", ShaderValue::Float(p1))?;
        dispatch_particles(c, floats, components)
    }
}

const COMBINE_SRC: &str = r#"
struct Params {
    components: u32,
    op: u32,
    b_scale: f32,
    b_offset: f32,
}

@if(in_place)  @group(0) @binding(0) var<storage, read_write> a:   array<f32>;
@if(!in_place) @group(0) @binding(0) var<storage, read>       a:   array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@if(!in_place) @group(0) @binding(2) var<storage, read_write> dst: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

fn combine_op(x: f32, y: f32) -> f32 {
    switch params.op {
        case 0u: { return x + y; }
        case 1u: { return x - y; }
        case 2u: { return x * y; }
        case 3u: { if y == 0.0 { return x; } return x / y; }
        case 4u: { return min(x, y); }
        case 5u: { return max(x, y); }
        case 6u: { return pow(max(x, 0.0), y); }
        default: { return x; }
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&a) / params.components;
    if i >= n { return; }
    for (var c = 0u; c < params.components; c = c + 1u) {
        let idx = i * params.components + c;
        let r = combine_op(a[idx], b[idx] * params.b_scale + params.b_offset);
        @if(in_place)  { a[idx] = r; }
        @if(!in_place) { dst[idx] = r; }
    }
}
"#;

static COMBINE: Mutex<Option<(Entity, Entity)>> = Mutex::new(None);

pub fn combine(
    dst: Entity,
    a: Entity,
    b: Entity,
    components: u32,
    op: u32,
    b_scale: f32,
    b_offset: f32,
) -> Result<()> {
    check_components("combine", components)?;
    let v = variants(&COMBINE, COMBINE_SRC)?;
    let floats = buffer_size(a)? / 4;

    let c = if dst == a {
        ensure_no_alias("combine", a, &[b])?;
        v.in_place
    } else {
        ensure_no_alias("combine", dst, &[a, b])?;
        compute_set(v.out_of_place, "a", ShaderValue::Buffer(a))?;
        compute_set(v.out_of_place, "dst", ShaderValue::Buffer(dst))?;
        v.out_of_place
    };
    if dst == a {
        compute_set(c, "a", ShaderValue::Buffer(a))?;
    }
    compute_set(c, "b", ShaderValue::Buffer(b))?;
    compute_set(c, "components", ShaderValue::UInt(components))?;
    compute_set(c, "op", ShaderValue::UInt(op))?;
    compute_set(c, "b_scale", ShaderValue::Float(b_scale))?;
    compute_set(c, "b_offset", ShaderValue::Float(b_offset))?;
    dispatch_particles(c, floats, components)
}

const MIX_SRC: &str = r#"
struct Params {
    components: u32,
    t_clamp: u32,
    t_scale: f32,
    t_offset: f32,
}

@if(in_place)  @group(0) @binding(0) var<storage, read_write> a:   array<f32>;
@if(!in_place) @group(0) @binding(0) var<storage, read>       a:   array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read> t: array<f32>;
@if(!in_place) @group(0) @binding(3) var<storage, read_write> dst: array<f32>;
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&a) / params.components;
    if i >= n { return; }
    // t is one value per particle, broadcast across the components.
    var tv = t[i] * params.t_scale + params.t_offset;
    if params.t_clamp != 0u { tv = clamp(tv, 0.0, 1.0); }
    for (var c = 0u; c < params.components; c = c + 1u) {
        let idx = i * params.components + c;
        let r = mix(a[idx], b[idx], tv);
        @if(in_place)  { a[idx] = r; }
        @if(!in_place) { dst[idx] = r; }
    }
}
"#;

static MIX: Mutex<Option<(Entity, Entity)>> = Mutex::new(None);

pub fn mix(
    dst: Entity,
    a: Entity,
    b: Entity,
    t: Entity,
    components: u32,
    t_scale: f32,
    t_offset: f32,
    t_clamp: bool,
) -> Result<()> {
    check_components("mix", components)?;
    let v = variants(&MIX, MIX_SRC)?;
    let floats = buffer_size(a)? / 4;

    let c = if dst == a {
        ensure_no_alias("mix", a, &[b, t])?;
        v.in_place
    } else {
        ensure_no_alias("mix", dst, &[a, b, t])?;
        compute_set(v.out_of_place, "a", ShaderValue::Buffer(a))?;
        compute_set(v.out_of_place, "dst", ShaderValue::Buffer(dst))?;
        v.out_of_place
    };
    if dst == a {
        compute_set(c, "a", ShaderValue::Buffer(a))?;
    }
    compute_set(c, "b", ShaderValue::Buffer(b))?;
    compute_set(c, "t", ShaderValue::Buffer(t))?;
    compute_set(c, "components", ShaderValue::UInt(components))?;
    compute_set(c, "t_scale", ShaderValue::Float(t_scale))?;
    compute_set(c, "t_offset", ShaderValue::Float(t_offset))?;
    compute_set(c, "t_clamp", ShaderValue::UInt(t_clamp as u32))?;
    dispatch_particles(c, floats, components)
}

const LOOKUP_SRC: &str = r#"
struct Params {
    in_components: u32,
    u_scale: f32,
    u_offset: f32,
    v_scale: f32,
    v_offset: f32,
    color_scale: f32,
    _p0: u32,
    _p1: u32,
}

@group(0) @binding(0) var<storage, read>       op_in: array<f32>;
@group(0) @binding(1) var<storage, read_write> dst:   array<f32>;
@group(0) @binding(2) var<uniform>             params: Params;
@group(0) @binding(3) var tex:  texture_2d<f32>;
@group(0) @binding(4) var samp: sampler;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&dst) / 4u;
    if i >= n { return; }

    var uv: vec2<f32>;
    if params.in_components == 1u {
        uv = vec2<f32>(op_in[i] * params.u_scale + params.u_offset, 0.5);
    } else {
        uv = vec2<f32>(
            op_in[i * 2u] * params.u_scale + params.u_offset,
            op_in[i * 2u + 1u] * params.v_scale + params.v_offset,
        );
    }
    uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));

    let c = textureSampleLevel(tex, samp, uv, 0.0);
    dst[i * 4u + 0u] = c.r * params.color_scale;
    dst[i * 4u + 1u] = c.g * params.color_scale;
    dst[i * 4u + 2u] = c.b * params.color_scale;
    dst[i * 4u + 3u] = c.a;
}
"#;

static LOOKUP: Mutex<Option<Entity>> = Mutex::new(None);

fn single_pipeline(cache: &Mutex<Option<Entity>>, src: &str) -> Result<Entity> {
    let mut guard = cache.lock().unwrap();
    if let Some(e) = *guard {
        return Ok(e);
    }
    let compute = crate::compute_create(crate::shader_create(src)?)?;
    *guard = Some(compute);
    Ok(compute)
}

#[allow(clippy::too_many_arguments)]
pub fn lookup(
    dst: Entity,
    op_in: Entity,
    tex: Entity,
    in_components: u32,
    u_scale: f32,
    u_offset: f32,
    v_scale: f32,
    v_offset: f32,
    color_scale: f32,
) -> Result<()> {
    if in_components != 1 && in_components != 2 {
        return Err(ProcessingError::InvalidArgument(format!(
            "lookup: in_components must be 1 or 2, got {in_components}"
        )));
    }
    ensure_no_alias("lookup", dst, &[op_in])?;
    let c = single_pipeline(&LOOKUP, LOOKUP_SRC)?;
    compute_set(c, "op_in", ShaderValue::Buffer(op_in))?;
    compute_set(c, "dst", ShaderValue::Buffer(dst))?;
    compute_set(c, "in_components", ShaderValue::UInt(in_components))?;
    compute_set(c, "u_scale", ShaderValue::Float(u_scale))?;
    compute_set(c, "u_offset", ShaderValue::Float(u_offset))?;
    compute_set(c, "v_scale", ShaderValue::Float(v_scale))?;
    compute_set(c, "v_offset", ShaderValue::Float(v_offset))?;
    compute_set(c, "color_scale", ShaderValue::Float(color_scale))?;
    compute_set(c, "tex", ShaderValue::Texture(tex))?;
    compute_set(c, "samp", ShaderValue::Texture(tex))?;
    let n = (buffer_size(dst)? / 16) as u32;
    compute_dispatch(c, n.div_ceil(WG), 1, 1)
}

pub const REDUCE_LENGTH: u32 = 0;
pub const REDUCE_SUM: u32 = 1;
pub const REDUCE_MIN: u32 = 2;
pub const REDUCE_MAX: u32 = 3;
pub const REDUCE_SUMSQ: u32 = 4;
pub const REDUCE_MEAN: u32 = 5;

const REDUCE_SRC: &str = r#"
struct Params { components: u32, op: u32 }

@group(0) @binding(0) var<storage, read>       src: array<f32>;
@group(0) @binding(1) var<storage, read_write> dst: array<f32>;
@group(0) @binding(2) var<uniform>             params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&dst);
    if i >= n { return; }
    let base = i * params.components;
    var s = 0.0;
    var ss = 0.0;
    var mn = src[base];
    var mx = src[base];
    for (var c = 0u; c < params.components; c = c + 1u) {
        let v = src[base + c];
        s = s + v;
        ss = ss + v * v;
        mn = min(mn, v);
        mx = max(mx, v);
    }
    switch params.op {
        case 0u: { dst[i] = sqrt(ss); }
        case 1u: { dst[i] = s; }
        case 2u: { dst[i] = mn; }
        case 3u: { dst[i] = mx; }
        case 4u: { dst[i] = ss; }
        case 5u: { dst[i] = s / f32(params.components); }
        default: { dst[i] = s; }
    }
}
"#;

static REDUCE: Mutex<Option<Entity>> = Mutex::new(None);

pub fn reduce_components(dst: Entity, src: Entity, components: u32, op: u32) -> Result<()> {
    check_components("reduce_components", components)?;
    ensure_no_alias("reduce_components", dst, &[src])?;
    let c = single_pipeline(&REDUCE, REDUCE_SRC)?;
    compute_set(c, "src", ShaderValue::Buffer(src))?;
    compute_set(c, "dst", ShaderValue::Buffer(dst))?;
    compute_set(c, "components", ShaderValue::UInt(components))?;
    compute_set(c, "op", ShaderValue::UInt(op))?;
    let n = (buffer_size(dst)? / 4) as u32;
    compute_dispatch(c, n.div_ceil(WG), 1, 1)
}

const EXTRACT_SRC: &str = r#"
struct Params { components: u32, index: u32 }

@group(0) @binding(0) var<storage, read>       src: array<f32>;
@group(0) @binding(1) var<storage, read_write> dst: array<f32>;
@group(0) @binding(2) var<uniform>             params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&dst);
    if i >= n { return; }
    dst[i] = src[i * params.components + params.index];
}
"#;

static EXTRACT: Mutex<Option<Entity>> = Mutex::new(None);

pub fn extract(dst: Entity, src: Entity, components: u32, index: u32) -> Result<()> {
    check_components("extract", components)?;
    if index >= components {
        return Err(ProcessingError::InvalidArgument(format!(
            "extract: index {index} out of range for {components} components"
        )));
    }
    ensure_no_alias("extract", dst, &[src])?;
    let c = single_pipeline(&EXTRACT, EXTRACT_SRC)?;
    compute_set(c, "src", ShaderValue::Buffer(src))?;
    compute_set(c, "dst", ShaderValue::Buffer(dst))?;
    compute_set(c, "components", ShaderValue::UInt(components))?;
    compute_set(c, "index", ShaderValue::UInt(index))?;
    let n = (buffer_size(dst)? / 4) as u32;
    compute_dispatch(c, n.div_ceil(WG), 1, 1)
}

const PACK_SRC: &str = r#"
struct Params { components: u32 }

@group(0) @binding(0) var<storage, read> s0: array<f32>;
@group(0) @binding(1) var<storage, read> s1: array<f32>;
@if(ge3) @group(0) @binding(2) var<storage, read> s2: array<f32>;
@if(ge4) @group(0) @binding(3) var<storage, read> s3: array<f32>;
@group(0) @binding(4) var<storage, read_write> dst: array<f32>;
@group(0) @binding(5) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&s0);
    if i >= n { return; }
    let c = params.components;
    dst[i * c + 0u] = s0[i];
    dst[i * c + 1u] = s1[i];
    @if(ge3) { dst[i * c + 2u] = s2[i]; }
    @if(ge4) { dst[i * c + 3u] = s3[i]; }
}
"#;

static PACK: Mutex<Option<[Entity; 3]>> = Mutex::new(None);

fn pack_pipeline(components: u32) -> Result<Entity> {
    let mut guard = PACK.lock().unwrap();
    if guard.is_none() {
        let p2 = crate::compute_create(shader_create_with_features(
            PACK_SRC,
            &[("ge3", false), ("ge4", false)],
        )?)?;
        let p3 = crate::compute_create(shader_create_with_features(
            PACK_SRC,
            &[("ge3", true), ("ge4", false)],
        )?)?;
        let p4 = crate::compute_create(shader_create_with_features(
            PACK_SRC,
            &[("ge3", true), ("ge4", true)],
        )?)?;
        *guard = Some([p2, p3, p4]);
    }
    Ok(guard.unwrap()[(components - 2) as usize])
}

pub fn pack(dst: Entity, sources: &[Entity]) -> Result<()> {
    let components = sources.len() as u32;
    if !(2..=4).contains(&components) {
        return Err(ProcessingError::InvalidArgument(format!(
            "pack: expects 2..=4 source buffers, got {}",
            sources.len()
        )));
    }
    ensure_no_alias("pack", dst, sources)?;
    let c = pack_pipeline(components)?;
    compute_set(c, "s0", ShaderValue::Buffer(sources[0]))?;
    compute_set(c, "s1", ShaderValue::Buffer(sources[1]))?;
    if components >= 3 {
        compute_set(c, "s2", ShaderValue::Buffer(sources[2]))?;
    }
    if components >= 4 {
        compute_set(c, "s3", ShaderValue::Buffer(sources[3]))?;
    }
    compute_set(c, "dst", ShaderValue::Buffer(dst))?;
    compute_set(c, "components", ShaderValue::UInt(components))?;
    let n = (buffer_size(sources[0])? / 4) as u32;
    compute_dispatch(c, n.div_ceil(WG), 1, 1)
}

pub const GEN_UNIFORM: u32 = 0;
pub const GEN_SIGNED: u32 = 1;
pub const GEN_GAUSSIAN: u32 = 2;

const GENERATE_SRC: &str = r#"
struct Params {
    components: u32,
    mode: u32,
    seed: u32,
    scale: f32,
    offset: f32,
}

@group(0) @binding(0) var<storage, read_write> dst: array<f32>;
@group(0) @binding(1) var<uniform>             params: Params;

fn hashu(x0: u32) -> u32 {
    var x = x0;
    x = x ^ (x >> 16u);
    x = x * 0x7feb352du;
    x = x ^ (x >> 15u);
    x = x * 0x846ca68bu;
    x = x ^ (x >> 16u);
    return x;
}

fn rnd(x: u32) -> f32 {
    return f32(hashu(x)) / 4294967295.0;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&dst) / params.components;
    if i >= n { return; }
    for (var c = 0u; c < params.components; c = c + 1u) {
        let id = i * params.components + c;
        let base = id ^ (params.seed * 0x9e3779b9u);
        var v: f32;
        switch params.mode {
            case 1u: { v = rnd(base) * 2.0 - 1.0; }
            case 2u: {
                let u1 = max(rnd(base), 1e-7);
                let u2 = rnd(base ^ 0x85ebca6bu);
                v = sqrt(-2.0 * log(u1)) * cos(6.28318530718 * u2);
            }
            default: { v = rnd(base); }
        }
        dst[id] = v * params.scale + params.offset;
    }
}
"#;

static GENERATE: Mutex<Option<Entity>> = Mutex::new(None);

pub fn generate(
    dst: Entity,
    components: u32,
    mode: u32,
    seed: u32,
    scale: f32,
    offset: f32,
) -> Result<()> {
    check_components("generate", components)?;
    let c = single_pipeline(&GENERATE, GENERATE_SRC)?;
    compute_set(c, "dst", ShaderValue::Buffer(dst))?;
    compute_set(c, "components", ShaderValue::UInt(components))?;
    compute_set(c, "mode", ShaderValue::UInt(mode))?;
    compute_set(c, "seed", ShaderValue::UInt(seed))?;
    compute_set(c, "scale", ShaderValue::Float(scale))?;
    compute_set(c, "offset", ShaderValue::Float(offset))?;
    let floats = buffer_size(dst)? / 4;
    dispatch_particles(c, floats, components)
}
