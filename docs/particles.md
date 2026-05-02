# Particles

A `Particles` is a GPU-resident container of named attribute buffers, drawn by
instancing a geometry once per element. The libprocessing analogue of a Houdini
point cloud.

## Pieces

- **`compute::Buffer`** (`crates/processing_render/src/compute.rs`) — typed GPU
  storage with CPU-side write, GPU readback, and a Python wrapper that tracks
  element type. Backs every Particles attribute buffer.
- **`Attribute`** (`crates/processing_render/src/geometry/attribute.rs`) —
  named typed attribute identity (`AttributeFormat::{Float, Float2, Float3,
  Float4}`), shared between Geometries and Particles. Builtins: `position`,
  `normal`, `color`, `uv`, plus the particles-only `rotation` (Float4 quat),
  `scale` (Float3), `dead` (Float, 0=alive).
- **Upstream `processing/bevy`** commit `ee443e51` adds `GpuBatchedMesh3d` and
  `GpuInstanceBatchReservations` — a fixed-capacity batch where a compute pass
  writes per-instance transforms into the upstream input buffer before
  `early_gpu_preprocess` consumes them.

## Construction

Empty:

```rust
let velocity = geometry_attribute_create("velocity", AttributeFormat::Float3)?;
let p = particles_create(10_000, vec![geometry_attribute_position(), velocity])?;
```

One zero-initialized buffer per requested attribute, sized
`capacity * attr.format.byte_size()`.

Mesh-seeded:

```rust
let source = geometry_sphere(5.0, 32, 24)?;
let p = particles_create_from_geometry(
    source,
    vec![position_attr, uv_attr, color_attr],
)?;
```

Capacity = mesh vertex count. Builtins seed from the matching mesh attribute
(`position` ← `ATTRIBUTE_POSITION`, `normal` ← `ATTRIBUTE_NORMAL`, `color` ←
`ATTRIBUTE_COLOR`, `uv` ← `ATTRIBUTE_UV_0`); particles-only builtins and custom
attributes start at zero.

## Apply

```rust
let spin = compute_create(shader_create(SPIN_WGSL)?)?;
compute_set(spin, "dt", ShaderValue::Float(0.016))?;
particles_apply(p, spin)?;
```

`particles_apply` binds each attribute buffer by name; bindings the shader
doesn't declare are skipped. Workgroup size is fixed at 64.

Built-in kernels: `particles_kernel_noise()` (uniforms `scale`, `strength`,
`time`), `particles_kernel_transform()` (`translate`, `rotation_axis`,
`rotation_angle`, `scale`, with identity defaults seeded so unset uniforms are
no-ops).

## Pack pass

Bridges Particles attribute buffers into the per-instance slots reserved by
`GpuBatchedMesh3d`. Runs as render-schedule systems:

- `extract_particles_draws` (ExtractSchedule) — copies Particles + buffer
  handles into the render world keyed by `ParticlesDraw` markers.
- `prepare_pack_bind_groups` (RenderSystems::PrepareBindGroups) — looks up or
  builds the pack pipeline for the specialization key + bind group.
- `dispatch_pack` (Core3d, before `early_gpu_preprocess`) — dispatches.

The pack shader (`particles/pack.wgsl`) is specialized per
`(HAS_ROTATION, HAS_SCALE, HAS_DEAD)`. For each slot it writes:

- `mesh_input_buffer[base+i].world_from_local` — `mat3x4` from rotation × scale
  + position translation.
- `mesh_input_buffer[base+i].tag = i` — slot index, available via
  `mesh_functions::get_tag(instance_index)`.
- `MeshCullingData[base+i].dead` — from the `dead` buffer if present, else 0.

## Materials

`ParticlesMaterial = ExtendedMaterial<StandardMaterial, ParticlesExtension>`
binds a `colors: Handle<ShaderBuffer>` and reads `particle_colors[mesh.tag]`.
Lit vs unlit is the `unlit` flag on the base `StandardMaterial`;
`apply_pbr_lighting` short-circuits when set.

Immediate-mode:

```rust
graphics_record_command(g, DrawCommand::FillBuffer(color_buffer_entity))?;
graphics_record_command(g, DrawCommand::Particles { particles, geometry: shape })?;
```

`fill(buffer)` sets the ambient albedo source; the next
`DrawCommand::Particles` allocates a `ParticlesMaterial` carrying that buffer.

Explicit:

```rust
let mat = material_create_pbr()?;
material_set_albedo_buffer(mat, color_buffer_entity)?;
material_set(mat, "roughness", ShaderValue::Float(0.4))?;
```

`material_set_albedo_buffer` / `material_set_albedo_color` swap the backing
asset between plain PBR and `ParticlesMaterial` while preserving every other
`StandardMaterial` field.

Custom shaders (per-particle UV, per-particle scalars, anything beyond color)
require a `CustomMaterial` that reads `mesh.tag` and indexes its own buffer.

## Emit

CPU-driven:

```rust
particles_emit(
    p,
    n,
    vec![
        (position_attr, position_bytes),  // n * 12 bytes
        (color_attr, color_bytes),        // n * 16 bytes
        (dead_attr, vec![0u8; n * 4]),    // alive
    ],
)?;
```

Writes to `[head, head+n) mod capacity` and advances `emit_head`. Two writes
when wrapping. No GPU allocator, no compaction. Capacity is a visible contract:
`>= peak_emission_rate × longest_lifespan`.

GPU-driven:

```rust
particles_emit_gpu(p, n, spawn_kernel)?;
```

Auto-binds attribute buffers and `emit_range: vec4<f32> = (base_slot, n,
capacity, 0)`. The kernel derives its target slot from `emit_range`.

No auto-defaults — if the field has a `dead` attribute, the caller must
include it (typically `n` zero-floats) or new slots inherit the previous
occupant's death.

## Lifecycle

`dead` is a builtin Float attribute (0=alive, non-zero=dead). When registered,
the pack pass writes it into `MeshCullingData::dead`; non-zero slots are
skipped in preprocessing.

Aging is user-managed via an apply kernel that increments age and flips
`dead` when age exceeds ttl. See `particles_lifecycle.rs`. Seed `dead = 1.0`
for unemitted ring slots so they don't render before being filled.

## Examples

- `particles_basic` — sphere-mesh-seeded particle cloud, PBR per-particle color.
- `particles_animated` — 10×10×10 grid rotating around Y via custom apply.
- `particles_oriented` — per-particle quaternion + scale.
- `particles_colored` / `particles_colored_pbr` — explicit material setup.
- `particles_emit` — continuous CPU ring-buffer emission.
- `particles_emit_gpu` — fountain spawned by a compute kernel.
- `particles_lifecycle` — emit + age + shrink-on-death.
- `particles_from_mesh` — sphere mesh as position source.
- `particles_noise` — built-in noise kernel jittering positions.
- `particles_stress` — 1M cubes on a grid, R/G/B lights, transform spin.
