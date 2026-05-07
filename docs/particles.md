# Particles

`Particles` are a collection of attribute buffers that can be used in order to sequence compute shaders. They are 
isomorphic to `Mesh` in the sense that they contain attributes and sets of data. In this way, you can think of a 
`Mesh` as the CPU representation of a `Particles` object, and the `Particles` object as the GPU representation of a 
`Mesh`. This allows convenient initialization of particle simulations from existing meshes, or using a mesh as a
constraint for a particle simulation, like a volume or a surface.

Another way to consider particles would be as the compute equivalent of `Graphics`. Where the `Grpahics` object
allows you to issue high level rasterization commands, the `Particles` object allows you to issue high level compute 
commands. In this way, you can think of a `Particles` object as a compute shader that is executed on the GPU, and the 
attributes as the inputs and outputs of the compute shader. In practice, a compute shader may also require additional
data, such as textures or bound vertex buffers, but the `Particles` object provides a high level abstraction for 
sequencing compute shaders and managing their inputs and outputs.

## Rasterization

Of course, it's not very interesting just to have a collection of attribute buffers on the GPU. The real power of 
`Particles` comes from the ability to rasterize them. This is done using the `particles` function, which takes a 
`Particles` object and a `Geometry` object as input, and issues a draw call that rasterizes the particles using the
current drawing state. In other words, it instances the provided geometry over the particles, using the attributes as 
to determine each instance's position, orientation, and other properties. This provides a powerful bridge between
`Particles` and the rest of the drawing API.