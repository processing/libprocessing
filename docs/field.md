# Field — GPU-resident particle and instancing

A `Field` is a GPU-resident container of named attribute buffers, drawn by instancing a
geometry once per element. It is the libprocessing analogue of a Houdini point cloud: a
collection of points carrying arbitrary named attributes, where storage is contextual and
attributes are first-class.

The high-level model is built on two existing libprocessing systems and one upstream
contribution:

- The `compute::Buffer` infrastructure (`crates/processing_render/src/compute.rs`)
  provides typed GPU storage buffers, CPU-side write, GPU readback, compute dispatch,
  and a Python wrapper that tracks element type for validation.
- The `Attribute` system (`crates/processing_render/src/geometry/attribute.rs`) provides
  named, typed attribute identities (`AttributeFormat::{Float, Float2, Float3, Float4}`)
  and a `BuiltinAttributes` resource holding stable entity IDs for `position`, `normal`,
  `color`, `uv`. The same identities flow through Geometries (per-vertex) and Fields
  (per-instance).
- Upstream `processing/bevy` commit `ee443e51` adds `GpuBatchedMesh3d` and the
  `GpuInstanceBatchReservations` machinery — a fixed-capacity batch where compute can
  write per-instance transforms into the upstream input buffer before
  `early_gpu_preprocess` consumes them.

## Concepts

### Field

The top-level container. Holds a set of named PBuffers (one per registered attribute),
the upstream reservation handle, and lifecycle metadata (capacity, emission head). Does
not carry geometry — it is the GPU compute context, not the shape that gets drawn.

### PBuffer

A single typed GPU storage buffer holding the values for one attribute across all
elements of a Field. Backed by `compute::Buffer`. Indexed by particle slot.

### Attribute

The naming and type identity for a buffer of values. Already exists. A `Field` registers
PBuffers against `Attribute` entities; lookups are typed entity comparisons, never string
matches. Format is declared at attribute creation and is the source of truth for element
size and shader-side type.

### Draw verb: `field`

`field(f, shape)` is the rasterization verb, analogous to `shape()`. Consumes ambient
material/fill/stroke state at call time and instances `shape` once per slot in `f`.

## Lifecycle

### Construction

```
let f = createField(|| {
    sphere(1.0);          // immediate-mode shape API
    // ...
}, capacity: 10_000);
```

The closure runs once and seeds initial attribute values via the existing immediate-mode
shape API (`beginShape`/`vertex`/`endShape`, `sphere`, `box`, etc.) The mapping is 1:1
from emitted vertices to particle slots. `capacity` is the upstream reservation size; if
omitted, it defaults to the closure's emitted vertex count.

`createField` called inside `draw()` emits a warning (hard error in strict mode), since
re-uploading every frame defeats the point of GPU residence.

### Apply

```
f.apply(NOISE, target: builtins.position, scale: 0.5)
 .apply(CURL,  target: builtins.position, strength: 1.0)
 .apply(custom_kernel(my_shader));
```

`apply()` dispatches a compute pass against the field. It is **chainable** — returns the
field. Built-in kernels (`NOISE`, `CURL`, `TURBULENCE`, etc.) are named constants;
custom WGSL is a separate constructor that takes a `Shader` and declares which
attributes it reads/writes.

`apply()` calls placed in `setup()` run once. `apply()` calls in `draw()` run every
frame — the retained-vs-dynamic distinction is purely about placement, not API.

`apply()` only ever touches PBuffers and uniforms. It has no knowledge of upstream
mesh-input buffers or render-side state. This keeps user-authored kernels free of
upstream coupling.

### Draw

```
fill(255, 100, 50);
field(f, sphere_shape);
```

The draw verb reads ambient material state at call time, dispatches the pack pass for
this field if not already packed this frame, and issues the instanced raster.

### Read / write

```
let positions = f.read(builtins.position);   // CPU readback as typed values
f.write(builtins.velocity, [...]);           // CPU upload
```

Inherited from the `compute::Buffer` Python surface (typed `__getitem__`/`__setitem__`,
`read()`, `write()`).

## Compute model

### apply() is PBuffer-only

A compute dispatch from `apply()` binds the field's PBuffers (those the kernel
declares it needs) plus any uniforms. It does not bind the upstream
`mesh_input_buffer`, `MeshCullingDataBuffer`, or any other upstream-managed resource.
This means kernel authors — including users writing CUSTOM WGSL — never need to know
upstream internals.

### Pack pass

The pack pass is the only code that bridges to the upstream batch infrastructure. It
runs once per `(frame, field)`, lazily, **only when** `field(f, shape)` is called this
frame. If a Field is used purely offline (apply + read), pack never runs and the
upstream input slots stay untouched.

Pack reads the current `position` / `rotation` / `scale` / lifecycle PBuffers, builds
the `world_from_local` `mat3x4`, and writes:

- `mesh_input_buffer[base + i].world_from_local`
- `mesh_input_buffer[base + i].tag = i` (the side-channel index for material shaders)
- `MeshCullingData[base + i].dead` from the field's lifecycle PBuffer if present

A CPU-side dirty flag on the Field component prevents redundant packing when the field
is drawn multiple times in one frame (e.g. shadow + main, multiple cameras).

### In-place vs ping-pong

The default for `apply()` is **in-place mutation**. The kernel reads `state[i]`, writes
back to `state[i]`. This is correct for every kernel that doesn't read other particles'
slots — which covers the overwhelming majority of creative-coding particle work
(noise/curl/drag/integration/attractor/repulsor). It is what the upstream
`gpu_particles` example does.

Kernels that read neighbor slots (smoothing, SPH-style fluid, sort steps) must opt into
ping-pong. Built-in kernels declare their access pattern; CUSTOM kernels accept a
`mode: PingPong` argument:

```
f.apply(custom_kernel(my_shader), mode: PingPong);
```

When ping-pong is requested, libprocessing transparently allocates the shadow buffer per
affected attribute and swaps after the dispatch. The user sees a single logical PBuffer
per Attribute regardless.

Between distinct `apply()` calls, no swap is needed — render-graph barriers between
sequential dispatches make in-place chaining correct.

## Material integration

### Ambient state

`field(f, shape)` participates in the same ambient material/fill/stroke state machine as
`shape()`. No new public material API.

### Default material — `ProcessingMaterial`

`ProcessingMaterial` is extended to consume tag-indexed PBuffers for the common
per-particle cases — at minimum **per-particle color**, so a `color` PBuffer and a
default `fill()` together produce per-particle tinting with the default material. This
is implemented as a tag-indexed storage-buffer read inside the material's fragment path:
`color_buffer[in.tag]` if the field declared a `color` PBuffer; fall through to ambient
fill otherwise.

### Custom material

Anything richer than what `ProcessingMaterial` consumes requires a `CustomMaterial`. The
user's WGSL declares storage bindings for the PBuffers it cares about and reads them
indexed by `in.tag`. The framework wires the bindings; the user writes the shader.

The asymmetry must be honest in the docs: per-particle color works with the default
material; per-particle UV / scale / arbitrary scalar attributes require a custom
material.

PBuffers do not bind to `@location(N)` per-instance vertex inputs. The upstream batch
infrastructure does not support per-instance attributes beyond the transform; the tag
side-channel is the route.

## Capacity, emission, lifecycle

Capacity is fixed at field creation. Two distinct emission patterns are supported, only
one of which needs new API.

### Continuous self-recycling — no new API

A field set up with a fixed population, where particles respawn on death within a
single user-authored kernel. The upstream `gpu_particles` example uses `pos.w` as a
lifecycle counter and a respawn branch in the simulate shader. This works on a regular
Field with a CUSTOM apply — the user's WGSL handles birth and death internally. No
emit primitive is needed; document the pattern, ship nothing.

### Discrete emission — ring buffer

When user code says "spawn N particles right now," use a ring buffer:

```
on_mouse_pressed:
    f.emit(50, |w| {
        w[builtins.position] = mouse_world_pos();
        w[builtins.velocity] = random_unit_vec3() * 5.0;
    });
```

Field carries a CPU-side `emit_head: u32`. `emit(n, init)` writes attribute values to
slots `[head, head + n) mod capacity` via `compute::Buffer::write_buffer_cpu`, then
advances head. No GPU-side allocator, no atomics, no compaction.

When the ring wraps, oldest particles are overwritten — graceful degradation if
emission outruns lifespan. Capacity is therefore a visible contract:
`capacity >= peak_emission_rate * longest_lifespan`.

Aliveness for raster: a particle is considered alive if its lifecycle PBuffer says so.
The pack pass writes `MeshCullingData[slot].dead` accordingly. The user is responsible
for setting the lifecycle PBuffer in their apply (typically `dead = age >= lifespan ?
1.0 : 0.0`).

## Immediate-mode compatibility

The settled "automatic instancing of repeated draw calls with the same material"
remains the immediate-mode escape hatch. A user looping `translate; sphere()` gets
auto-instancing for free, no Field needed. Field is for cases where compute matters or
populations are large and dynamic.

`createField` inside `draw()` warns (hard error in strict mode). There is no separate
"ephemeral field" API — the warning is the educational nudge. Most rebuild-every-frame
intentions should be a static Field with a per-frame `apply(CUSTOM, ...)`.

## Upstream bridge

A Field is backed by a single entity carrying:

- `GpuBatchedMesh3d { mesh, max_capacity }` — upstream
- `MeshMaterial3d<M>` — upstream, ambient material handle
- `Field` — libprocessing component holding the PBuffer map, ring-buffer head, dirty
  flag, and other lifecycle metadata
- An `Aabb` for culling

`GpuBatchedMesh3d` and `Mesh3d` are mutually exclusive on one entity by upstream design;
the immediate-mode `Mesh3d` path is not available on a Field entity, and vice versa.

The pack pass schedules its work to land in the render world before
`early_gpu_preprocess`. It does not register as a `Render` system; it is called inline
from the Field draw-command processor.

## Non-goals (v1)

- **GPU-driven emission.** No GPU-side atomic counter / dead-slot allocator. Emission
  is CPU-driven only.
- **Sparse alive set / compaction.** Every reserved slot is part of the rendered batch;
  cull happens via the per-slot `dead` flag.
- **Per-instance attributes beyond the tag side-channel.** Upstream does not support
  per-instance vertex inputs other than the transform; the tag plus storage-buffer
  lookup is the only mechanism.
- **Multi-emitter pools.** A Field is one ring buffer. Use multiple Fields if logical
  separation is needed.
- **Cross-field operations.** No `apply()` that reads from one field and writes to
  another. Single-field kernels only.

## Open questions

- **Rotation format ambiguity.** `Float3` rotation = euler, `Float4` = quat is decided
  at attribute registration. Worth re-examining if users frequently want one and get
  the other; alternatively, ship a typed wrapper helper.
- **Multiple cameras / shadow path.** Pack-once-per-frame assumes the upstream input is
  the same across all views. If a future camera-specific pass needs different
  per-instance state, the dirty model needs to grow per-view.
- **Custom material binding declaration.** How a `CustomMaterial` declares which Field
  PBuffers it needs as storage bindings is unsettled. Likely an explicit
  `material.bind("color", attribute)` call at material creation time.
