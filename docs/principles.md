# What is libprocessing?

In short, a new experimental effort to abstract the Processing API into a new native, cross-platform library written in
Rust and powered by the [Bevy](bevy.org) game engine. Processing is historically a few different renderers, but most
recently (as in 15 years ago) an OpenGL renderer using [JOGL](https://jogamp.org/jogl/www/). Because
of [deprecations](https://developer.apple.com/documentation/macos-release-notes/macos-mojave-10_14-release-notes#Open-GL-and-Open-CL)
and lack of support for modern graphics development techniques, it's become desirable for us to prototype a new renderer
based around [WebGPU](https://webgpu.org/) as a new cross-platform backend for future Processing efforts, both in Java
and other languages.

## Why Rust/Bevy?

Bevy is an open-source game engine written in the Rust programming language. Bevy and Rust more generally have a number
of characteristics that are desirable for a project like Processing:

- Rust is a systems programming language which means that it can easily be deployed and run on a variety of different
  platforms and be easily embedded in most programming languages which target the C ABI. For example,
  see [Bevy running on an ESP32 microcontroler].
- Unlike C/C++, Rust provides modern build and development tooling which is more suited to beginner and intermediate
  contributors. The Rust ecosystem places a high emphasis on learning material and is generally more inclusive towards
  those who are less familiar with systems programming.
- Bevy is based around the [entity component system](https://en.wikipedia.org/wiki/Entity_component_system) design
  pattern, which focuses on modularity and composability. Rather than being a monolithic game engine like Unity, Unreal,
  or Godot, it emphasizes being able to pick and choose which components any given application uses. This makes it
  particularly well suited to bridging Processing's immediate-mode API with a full-fledged engine.
- Bevy is committed to the WebGPU and open-source graphics ecosystem and an active participant in the WebGPU standards
  process.
- As a community, Bevy is strategically interested in use cases that go beyond games, including art, CAD applications,
  scientific computing, etc.

# Technology stack

There are several layers required to make this work. Starting from the outermost layer of the onion:

1. `bindgen`, a rust project for generating C headers from Rust code. This is what Java and other languages bind to.
2. Our Rust FFI library, which wraps libprocessing. FFI rust code mostly means declaring the public interface for
   `bindgen` using `extern "C"` functions and mapping types compatible with the C ABI to call our libprocessing API.
3. libprocessing is our Rust library that exposes the primary Processing API.
4. Bevy is a fully featured application framework and game engine that helps structure our Rust code and provides a
   variety of graphics related features.
5. `wgpu` is the Rust implementation of the WebGPU standard and is used by Bevy as the rendering hardware interface (
   RHI) for Bevy.
6. Vulkan/Metal are the low-level graphics APIs that actually interact with the user's hardware. For the most part, we
   just need to know these exist and are what ultimately runs our graphics code.

# Development principles

## Library design

We want to expose the Processing API in a kind of procedural style (i.e. imperative, function driven) that mirrors the
immediate-mode idiom of Processing sketches.

### Hew to the existing API, but not at the cost of baking in historical mistakes

We want to closely model the existing Processing API, but strategically fix issues that may be a consequence of legacy
design. We also want to expose features that Bevy provides that are trivial to do so. For example, Processing does not
support different texture formats, but this is super easy in Bevy, so we should make that possible in the API even if
Java Processing can't fully exercise it.

We also want to take learnings from p5 where it makes sense! There are some cases where they may have settled on a
better naming convention or api style for something and we should feel free to take their learnings, documenting
differences where we do.

### Accessible for intermediate users

While this is a lower-level library that has fewer guardrails than the high-level Processing API, we still want it to be
something that feels accessible to users who are interested in getting more involved with development of the library. In
that respect, we should always prioritize internal documentation, examples, and bread crumbs to facilitate learning.
Graphics programming is hard, and in some cases we may need to express patterns that are complicated, but should always
make sure that we describe the "why" even where the "how" may elude less experienced contributors.

### Naming

All exposed C functions should begin with the `processing_` namespace and should be further qualified by the type of
data they operate on, e.g. `processing_graphics`, `processing_geometry`, etc. Because this is a lower-level library,
long names are fine and it's better to be descriptive rather than terse. As such, we prefer multiple functions that
accomplish similar tasks rather than overloading a single function with complicated parameters, for example we have both
`processing_graphics_background_color` and `processing_grpahics_background_image` although these are presented in the
user-facing API via an overload.

### Handles, not pointers

We never return pointers to data that lives in libprocessing. Because we use the ECS to manage Rust data, where longer
lifetimes are necessary we return an `Entity` id, which can be returned to the user as a `u64` containing both the index
and generation of the ECS entity. Any data not representing an API-level object should be returned on the stack and
where an allocation is necessary (e.g. buffers for pixel data), it's the responsibility of the consumer to allocate and
provide the pointer to the requisite allocation.

API-level objects should wrap ids and they should never be exposed to the user as a first-class concept. All data which
is returned via an `Entity` id should have a corresponding destructor function which removes the entity from the ECS and
frees any associated resources. It's the responsibility of the API wrapper object to call this function when being
destroyed.

### Bridging Bevy and immediate-mode APIs

The central architectural challenge of libprocessing is reconciling two fundamentally different models of how graphics
programming works.

Immediate mode treats drawing as a sequence of imperative commands: when you call `rect(10, 10, 50, 50)` a rectangle
appears *now*. State is global and mutable where the user thinks in terms of a linear script that paints pixels onto a 
canvas. This is the traditional "sketch" model of Processing.

Retained mode in the case of Bevy's ECS treats the scene as a database of entities with components. Systems query and
transform this data, often in parallel. Rendering is a separate phase that happens later, potentially pipelined across
frames. The renderer batches draw calls for efficiency and has a number of optimizations that could be considered a form 
of eventual consistency (think of a game where objects take flicker in and out on screen as assets load). The user 
thinks in terms of a scene graph that is updated over time, where multiple asynchronous systems are modifying data.

Neither model is wrong! But they very much optimize for different things. Immediate mode is intuitive and exploratory 
which is why it's so well suited to learning, prototyping, and creative coding. Retained mode is efficient and scalable, 
perfect for games with thousands or hundreds of thousands of objects or for more complex artworks that require 
sophisticated rendering techniques.

Our job is to present the former while implementing it atop the latter.

This requires us to invert several of Bevy's defaults:

- Recording instead of executing: When user code calls a draw function, we don't spawn entities immediately.
  Instead, we record the intent as a `DrawCommand` in a per-graphics `CommandBuffer`. This preserves call order and
  allows us to process commands in a controlled batch.
- Synchronous frame control: Bevy wants to manage its own main loop with pipelined rendering. We instead hold the
  `App` in a thread-local and call `app.update()` only when the user explicitly flushes, i.e. makes a change that
  requires rendering to occur in occur because of some data dependency. 
- Selective rendering: By default, Bevy will render all active cameras every update. We disable cameras unless the
  user has requested a flush, using marker components to signal which surfaces should actually render.
- Transient geometry: In immediate mode, shapes exist only for the frame they're drawn. We spawn mesh entities when
  flushing commands and despawn them before the next frame. The ECS becomes a staging area rather than a persistent
  scene graph.

We work around this in the following manner:

- By default, all `Camera` entities should be disabled via the `active` field, which ensures calls to `app.update()`
  will not do any rendering work unless a camera has been specifically enabled for rendering.
- When a `Camera` is desired to be rendered, it's `active` field should be set and the `Flush` marker component should
  be added to its parent surface. All systems which produce renderable data should check for the `Flush` component to
  ensure they only work on the specific surface that is being rendered.
- A `Camera` should set `CameraWriteMode::Skip` to ensure that its intermediate texture is not written to the final
  `RenderTarget` until the user calls `endDraw`.

In this way, as long as `Camera` state is managed correctly, it's totally fine to call `app.update()` or run individual
systems as they will not trigger unnecessary renders and presents to the surface.

### Working with systems and borrows

Working with raw `&mut World` in Bevy can lead to frustrating situations with respect to borrows, as many methods
require mutable world access which then prevents doing other operations. Because of our immediate mode style, we much
more frequently are imperatively modifying the world rather than adding "normal" Bevy systems that run in a schedule.

There are several strategies that can help work around this:

1. You can call systems on `World` that accept and return data using `In<T>` parameters:

```rust
// In a object module define plain systems with `In` params:
pub fn create(
    In((width, height, surface_entity)): In<(u32, u32, Entity)>,
    mut commands: Commands,
    render_device: Res<RenderDevice>,
) -> Result<Entity> {
    // implementation that uses Commands, queries, resources, etc.
    Ok(entity)
}

// In lib, call the system via run_system_cached_with:
pub fn graphics_create(surface_entity: Entity, width: u32, height: u32) -> error::Result<Entity> {
    app_mut(|app| {
        app.world_mut()
            .run_system_cached_with(graphics::create, (width, height, surface_entity))
            .unwrap()
    })
}
```

The `In<T>` parameter receives input data passed via `run_system_cached_with()`. For multiple parameters, use tuples:
`In<(T, U, V)>`. The `In` parameter must always be the first system parameter.

2. Collect results from queries into intermediate collections. This can resolve the borrow for a query at the cost of a
   bit of inefficiency.
3. Use `world.resource_scope` or otherwise temporarily remove certain resources from the world (making sure to add them
   back later).

### Thread safety

As of now, libprocessing is intended to be used in a single-threaded manner on the main thread of the application (which
is a specific requirement for macOS window rendering and could be loosened). This is primarily for implementation
simplicity; Bevy itself is designed to be highly parallel and does not have this restriction. In the future, we may
decide to fully embrace multi-threading if the implementation complexity is worth the performance benefits.

What this means concretely is:

- Our `App` instance lives on a single thread.
- Calling `app.update()` will run the entire main and render schedule in a blocking manner, including presenting the
  frame if configured to do so.
- Asset loads via Bevy's `AssetServer` are blocking.
- Non-send resources are guaranteed to live on the same thread as `App`.

## Error handling

Due to working at the FFI boundary, libprocessing requires interaction with unsafe code. We strive to be memory and
panic-safe in all instances, which means that a consumer of libprocessing should not be able to trigger undefined
behavior or crash their process simply by calling into libprocessing. However, libProcessing is not intended as a high
level API and incorrect use may result in unexpected results, memory leaks, or other undesirable behavior.

In general, our assumption is that errors encountered by consumers indicate exceptional situations in which the correct
behavior is to halt program execution. In other words, errors are not intended for user feedback and where necessary the
caller is expected to validate input prior to calling into libprocessing. More specifically, we want to expose
validation as data, so have functions like `validate_shader` which returns friendly, user oriented text, rather than
something like `compile_shader` which provides a validation error. The latter should also still validate but without the
assumption of surfacing feedback to the user.

### Rust errors

We have a `ProcessingError` enum that uses the [`thiserror`](https://docs.rs/thiserror/latest/thiserror/) library to
help manage error variants (see the documentation for more info). In general, we encourage adding new error variants
that are unique to a given situation. As error states are considered exceptional, it's okay if these messages are better
oriented towards a bug report that a user may file than informative to the user. Our expectation is that users should
never see them in ordinary practice.

We also expose a `Result<T, ProcessingError>` type alias that should be used in most cases as the return type for
top-level functions.

In our rendering code, `unwrap` and `expect` should only be used in situations that indicate the renderer itself has a
bug, but is totally acceptable where invariants should be held. See [this blog post](https://burntsushi.net/unwrap/) by
Burntsushi for some general guidelines here.

### Cross-boundary error conditions

Currently, we store the errors state in a thread-local `CString`, where any non-null value indicates the presence of an
error. Consumers are required to call `processing_check_error`
after every operation and throw an exception if any value is present.

Inside the FFI library, all expose functions must:

- Clear any existing error state using `clear_error` at the top of their function.
- Check any results and set the error state using `set_error`.
- Catch any panics using `catch_unwind`.

The convenience function `check` is provided for working with `ProcessingError` and should be used in most cases to help
with error handling.
