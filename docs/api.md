# API

## API Objects

All objects are referenced via an entity identifier returned to the caller as an opaque
`u64` value via a `create` function. When an object is no longer needed, it must be
explicitly destroyed via a `destroy` function.

### Surface

A surface represents a drawing area. It is primarily used as a target for rendering and typically
is associated with a window or an off-screen buffer. The equivalent of a surface in Bevy is 
a `RenderTarget`, which can be a window or a texture.

Note, a "surface" is also a technical term in graphics APIs like Vulkan and WebGPU, where it refers to
the platform-specific representation of a drawing area that the swapchain presents images to. However,
in this API, we use "surface" in a more general sense to refer to any drawing area, not just those tied 
to swapchains.

Processing users are not typically expected to interact directly with surfaces. Rather, a graphics object
is created and associated with a surface internally, which will implicitly be a new window unless the user
specifically requests a headless drawing context, in which case an off-screen surface is created.

### Graphics

Graphics objects encapsulate the rendering context and state and provide the core methods for drawing shapes,
images, and text. They manage the current drawing state, including colors, stroke weights, transformations,
and other properties that affect rendering. In Bevy, this is equivalent to a `Camera` entity.

In the Java Processing API, graphics objects are a subclass of `PImage`. In this API, graphics objects are distinct from 
images rather than bearing an explicit is-a relationship. Importantly, the "image" for a Bevy `Camera` is the internal
rendering texture that the camera draws to (`ViewTarget`), which is not typically directly exposed to users.

For consistency, all image functions should accept a graphics object, although its internal image representation is 
not guaranteed to be the same as a user-created image.

### Image

Images are 2D or 3D arrays of pixels that can be drawn onto surfaces. They can be created from files, generated procedurally,
or created as empty canvases for off-screen rendering. In Bevy, images are represented as `Image` assets. Images exist
simultaneously as GPU resources and CPU-side data structures and have a lifecycle that requires the use to load pixels
from the GPU to the CPU before accessing pixel data directly and to flush pixel data from the CPU to the GPU after
modifying pixel data directly.

### Font

[//]: # (TODO: Document Font API object)

### Geometry

Geometry (also known as `PShape` in the Java Processing API) represents complex shapes defined by vertices, edges, and 
faces. Geometry objects can encapsulate 2D shapes, 3D models, or custom vertex data. They can be created 
programmatically or loaded from external files. In Bevy, geometry is typically represented using `Mesh` assets and
require an associated `Material` to be rendered.

Like an image, geometry exists as both a GPU resource and a CPU-side data structure. Users must ensure that any changes
to the CPU-side geometry are synchronized with the GPU resource before rendering.

### Layout

A layout describes which vertex attributes are present in geometry and how they are arranged in a vertex buffer. 
In Bevy, this corresponds to a `MeshVertexBufferLayout`.

### Attribute

An attribute describe a single element of the layout of a vertex buffer. Every geometry must have a position attribute,
and may optionally have other standard attributes such as normal, tangent, color, and texture coordinates (UVs). Custom 
attributes can be defined for passing additional data to a material shader. In Bevy, these map to `MeshVertexAttribute`.

### Material

A material defines the appearance of geometry when rendered. In Bevy, materials are represented using `Material` assets 
that define how geometry interacts with light and other visual effects, typically a PBR `StandardMaterial`. Processing
has a simpler material model based on Blinn-Phong shading and in the Java Processing API, materials are typically defined
as vertex attributes within a `PShape`.

We define materials as their own API objects as this is key to enabling instanced rendering in retained mode graphics.
Bevy will batch draws of geometry that share the same material, allowing for efficient rendering of many instances of 
the same geometry with the same appearance.

This API also helps define the high level interface to work with shaders, which are used to implement custom materials.
While materials are typically defined in terms of their interaction with light in a 3D scene, the simplest "sketch"
implementation of a shader is simply a fragment shader that is applied to a full-screen quad. In this way, materials
can also be used to implement 2D image processing effects, etc.

### Shader

[//]: # (TODO: Document Shader API object, do we even need this with a sufficiently robust Material API?)