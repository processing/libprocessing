# Particles — GPU-resident particle and instancing

A `Particles` is a GPU-resident container of named attribute buffers, drawn by instancing a
geometry once per element. It is the libprocessing analogue of a Houdini point cloud: a
collection of points carrying arbitrary named attributes, where storage is contextual and
attributes are first-class.

The implementation rests on two existing libprocessing systems and one upstream contribution:

- **`compute::Buffer`** (`crates/processing_render/src/compute.rs`) — typed GPU storage
  buffers with CPU-side write, GPU readback, compute dispatch, and a Python wrapper that
  tracks element type for validation. This is what backs every Particles attribute buffer.
- **`Attribute`** (`crates/processing_render/src/geometry/attribute.rs`) — named typed
  attribute identities (`AttributeFormat::{Float, Float2, Float3, Float4}`) shared between
  Geometries (per-vertex) and Particles (per-instance). `BuiltinAttributes` exposes
  `position`, `normal`, `color`, `uv`, `rotation` (Float4 quat), `scale` (Float3), `dead`
  (Float, 0=alive). The last three are particles-only.
- **Upstream `processing/bevy`** commit `ee443e51` adds `GpuBatchedMesh3d` and the
  `GpuInstanceBatchReservations` machinery — a fixed-capacity batch where a compute pass
  can write per-instance transforms into the upstream input buffer before
  `early_gpu_preprocess` consumes them.

## Concepts

### Particles

The top-level container. Holds a set of named attribute buffers (one per registered
attribute), an optional persistent rasterization entity, a ring-buffer emit cursor, and
per-Particles render state. Does not carry geometry — that's supplied at draw time.

### Attribute buffer

A single typed GPU storage buffer holding the values for one attribute across all
elements. Backed by `compute::Buffer`. Indexed by particle slot.

### Attribute

The naming + type identity. A Particles maps `Attribute` entities to `compute::Buffer`
entities. Lookups are typed entity comparisons, never strings. The Format declared at
attribute creation is the source of truth for element byte size and shader-side semantics
(Float4 rotation = quat, Float3 = position/scale, etc.).

### Draw verb: `particles`

`particles(f, shape)` (`DrawCommand::Particles { particles, geometry }`) is the rasterization verb.
Reads ambient material at call time and instances `shape` once per slot in `f`.

## Construction

### Empty Particles

```rust
let position = geometry_attribute_position();
let velocity = geometry_attribute_create("velocity", AttributeFormat::Float3)?;
let f = particles_create(10_000, vec![position, velocity])?;
```

Allocates one zero-initialized buffer per requested attribute, sized by `capacity *
attr.format.byte_size()`.

### Mesh-seeded Particles

```rust
let source = geometry_sphere(5.0, 32, 24)?;
let f = particles_create_from_geometry(
    source,
    vec![position_attr, uv_attr, color_attr],
)?;
```

Capacity = mesh vertex count. Each registered attribute is pre-seeded from the matching
mesh attribute when names + formats line up:

| Particles attribute | Mesh attribute (Bevy) |
|----|----|
| `position` (Float3) | `Mesh::ATTRIBUTE_POSITION` |
| `normal` (Float3) | `Mesh::ATTRIBUTE_NORMAL` |
| `color` (Float4) | `Mesh::ATTRIBUTE_COLOR` |
| `uv` (Float2) | `Mesh::ATTRIBUTE_UV_0` |

Particles-only builtins (`rotation`, `scale`, `dead`) and custom attributes are zero-init
(meshes don't carry them).

## Apply (attribute-buffer-only compute)

```rust
let shader = shader_create(SPIN_WGSL)?;
let spin = compute_create(shader)?;
compute_set(spin, "dt", ShaderValue::Float(0.016))?;
particles_apply(field, spin)?;
```

`particles_apply` iterates the field's attribute buffers and calls `compute_set(compute,
attr.name, ShaderValue::Buffer(buf_entity))` for each. Unknown shader properties are
silently skipped, so the kernel only declares the attributes it needs. Workgroup size is
fixed at 64 — kernels must declare `@workgroup_size(64)`.

The kernel's bind group only ever contains the field's attribute buffers + uniforms. The
kernel never touches upstream input/culling buffers — that's the pack pass's job.

In `setup()` apply runs once; in `draw()` it runs every frame. The retained-vs-dynamic
distinction is purely about placement.

## Pack pass

The pack pass is the only code that bridges to the upstream batch infrastructure. It runs
as standard render-schedule systems:

- **`extract_particles_draws`** (`ExtractSchedule`) — reads `ParticlesDraw` markers from main
  world, copies (Particles, position/rotation/scale/dead buffer handles) into render world.
- **`prepare_pack_bind_groups`** (`RenderSystems::PrepareBindGroups`) — looks up or
  creates the pack pipeline for the field's specialization key, builds a bind group with
  the field's buffers + the upstream input/culling buffers + a uniform with `(base_index,
  count)`.
- **`dispatch_pack`** (`Core3d`, `before(early_gpu_preprocess)`) — dispatches the compute
  pass.

The pack shader (`particles/pack.wgsl`) is specialized via shader_defs:

- `HAS_ROTATION` — bind a `rotation` buffer (Float4 quat). Otherwise identity.
- `HAS_SCALE` — bind a `scale` buffer (Float3). Otherwise unit.
- `HAS_DEAD` — bind a `dead` buffer (Float). Otherwise alive.

For each particle slot the pack writes:

- `mesh_input_buffer[base+i].world_from_local` — `mat3x4` from rotation × scale +
  position translation.
- `mesh_input_buffer[base+i].tag = i` — slot index, available to material shaders via
  `mesh_functions::get_tag(instance_index)`.
- `MeshCullingData[base+i].dead` — from the dead buffer if present, else 0.

Pipelines are cached per `PackPipelineKey { has_rotation, has_scale, has_dead }`.

## Materials

A single material type — `ParticlesMaterial` (`ExtendedMaterial<StandardMaterial,
ParticlesExtension>`) — handles both lit and unlit per-particle color. The
extension binds `colors: Handle<ShaderBuffer>` and the shader reads
`particle_colors[mesh.tag]`. Lit vs unlit is the `unlit` flag on the base
`StandardMaterial`; `apply_pbr_lighting` short-circuits to base × particle color
when set.

### `fill(buffer)` — immediate-mode

```rust
graphics_record_command(g, DrawCommand::FillBuffer(color_buffer_entity))?;
graphics_record_command(g, DrawCommand::Particles { particles, geometry: shape })?;
```

Sets the ambient fill source to the buffer; the next `DrawCommand::Particles`
allocates a `ParticlesMaterial` carrying that buffer. No explicit material
construction needed.

### Explicit material with `albedo` source

```rust
let mat = material_create_pbr()?;
material_set_albedo_buffer(mat, color_buffer_entity)?;
material_set(mat, "roughness", ShaderValue::Float(0.4))?;
```

`albedo` accepts either a constant color (`material_set_albedo_color`) or a
buffer (`material_set_albedo_buffer`); switching between them swaps the backing
asset type while preserving the `StandardMaterial` state (roughness / metallic /
emissive / unlit / etc.).

### Anything richer

Per-particle UV, custom scalars, etc. require a `CustomMaterial` where the user writes
WGSL that reads `mesh.tag` and indexes into their own storage buffer.

## Emit (ring buffer)

```rust
particles_emit(
    field,
    n,
    vec![
        (position_attr, position_bytes),  // n * 12 bytes
        (color_attr, color_bytes),        // n * 16 bytes
        (dead_attr, vec![0u8; n * 4]),    // alive
    ],
)?;
```

Writes to slots `[head, head+n) mod capacity` via `compute::Buffer::write_buffer_cpu`,
then advances the field's `emit_head`. Two writes when wrapping. No GPU-side allocator,
no atomics, no compaction.

When the ring wraps, oldest particles are overwritten — capacity is a visible contract:
`>= peak_emission_rate × longest_lifespan`.

The user supplies bytes explicitly per attribute. There is no auto-default — if the field
has a `dead` attribute, the user must include it (typically as `n` zero-floats) or new
slots inherit the previous occupant's death.

## Lifecycle

`dead` is a builtin Float attribute (0=alive, non-zero=dead). When the field has it
registered, the pack pass reads it and writes `MeshCullingData::dead` — non-zero means
the slot is skipped in preprocessing and never rendered.

Aging is user-managed: write an apply() shader that increments an age attribute and sets
`dead = 1.0` when age exceeds a threshold. The canonical pattern (`particles_lifecycle.rs`):

```wgsl
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= arrayLength(&age) { return; }
    if dead[i] != 0.0 { return; }

    age[i] = age[i] + dt;
    let life = clamp(1.0 - age[i] / ttl, 0.0, 1.0);
    let s = life * life;
    scale[i*3+0] = s; scale[i*3+1] = s; scale[i*3+2] = s;  // shrink

    if age[i] > ttl { dead[i] = 1.0; }
}
```

For unemitted ring-buffer slots, seed `dead = 1.0` at field-create time so they don't
render before being emitted into.

## Compute model

Default mutation mode is **in-place**. Most particle kernels (NOISE, CURL, drag,
integration) only read their own slot; in-place is correct and 2× cheaper than
ping-pong. Ping-pong (for kernels that read neighbor slots) is not yet shipped.

Between sequential `apply()` calls, no buffer swap is needed — render-graph barriers
handle ordering.

## Immediate-mode compatibility

The "automatic instancing of repeated draw calls with the same material" path remains the
non-Particles instancing escape hatch. A user looping `translate; sphere()` gets
auto-instancing via `Mesh3d` for free, no Particles needed. Particles is for cases where compute
matters or populations are large + dynamic.

`GpuBatchedMesh3d` (used by Particles's transient draw entity) and `Mesh3d` are mutually
exclusive on one entity by upstream design.

## v1 non-goals

- **Chainable `apply()`** — currently flat function call. Quality of life.
- **Stateful builder methods on Particles** (`particles.color() / field.vertex()`) — the
  mesh-seeding path covers most cases.
- **Closure-based `create_particles(|| { sphere(); ... })` recording mode** — would need
  shape-API recording infrastructure (sphere/box dispatching into a Geometry instead of
  drawing).
- **GPU-driven emission**, sparse alive set / compaction, multi-emitter pools, cross-field
  operations.
- **Per-instance attributes via `@location`** — upstream supports only the transform; the
  tag side-channel into a storage buffer is the only path for non-transform per-instance
  data.
- **Auto-default attribute reset on `particles_emit`**.
- **User-configurable PBR properties** on `ParticlesMaterial` (roughness, metallic) via
  `material_set`.
- **Built-in compute kernels** (NOISE, CURL, etc.) — packaged WGSL.
- **Ping-pong apply**.

## Architectural notes

- **Pack pass schedule.** The original design intent was to tie pack to the `particles(f,
  shape)` draw verb call (lazy, one-shot). The implementation runs pack as standard
  render-schedule systems triggered by the `ParticlesDraw` marker on transient draw entities.
  Same effect (pack only fires when there's something to draw), simpler integration.
- **Per-particle color material.** The original design intent was to extend
  `ProcessingMaterial`. The implementation is two standalone material types
  (`ParticlesMaterial`, `ParticlesMaterial`). Standalone was cleaner; ambient `fill()`
  doesn't auto-tint particles, but the user explicitly opts in via the dedicated factory.
- **Persistent draw entity.** The Particles's `draw_entity` must persist across frames — the
  upstream batching queue processes mesh instance batches one frame after the reservation
  is created, so despawning per-frame would lose the entity before queueing.

## Examples

- `particles_basic` — 1000 spheres on a 10×10×10 grid, static positions, default material.
- `particles_animated` — same grid, rotating around Y via per-frame compute apply.
- `particles_oriented` — 125 cubes with per-particle quaternion rotation + per-particle scale.
- `particles_colored` — RGB-gradient cube via `ParticlesMaterial` (unlit).
- `particles_colored_pbr` — same, lit with `ParticlesMaterial`.
- `particles_emit` — continuous ring-buffer emission in a spiral.
- `particles_lifecycle` — fountain that emits particles with aging + shrink-on-death.
- `particles_from_mesh` — particles positioned at the vertices of a source sphere mesh.

## Fixed bugs (during development)

- **`bevy_naga_reflect` struct uniform encoding.** `type_size` previously aligned every
  struct member to 16 bytes (so 4 f32s claimed 64 bytes). `write_to_buffer` used
  `encase::UniformBuffer::write` which resets to offset 0 each call — only the last
  member's bytes survived. Both fixed in the local checkout at
  `~/src/github.com/tychedelia/bevy_naga_reflect`. libprocessing's `Cargo.toml` points at
  the local checkout via `path =` until the fix is pushed back.
- **`mode_3d` near-plane.** Was `camera_z / 10` (~60 units), which clipped particles when
  the camera was moved closer via `transform_set_position`. Changed to fixed `near = 1.0`.
