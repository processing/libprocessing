import struct

from mewnala import Graphics, Shader, Compute, Buffer

g = Graphics.new_offscreen(1, 1, "", None)
g.begin_draw()

shader = Shader("""
@group(0) @binding(0)
var<storage, read_write> output: array<u32>;

@compute @workgroup_size(1)
fn main() {
    output[0] = 1u;
    output[1] = 2u;
    output[2] = 3u;
    output[3] = 4u;
}
""")

buf = Buffer(size=16)
compute = Compute(shader)
compute.set(output=buf)
compute.dispatch(1, 1, 1)

data = buf.read()
assert isinstance(data, bytes), f"expected bytes, got {type(data)}"
assert list(struct.unpack("<4I", data)) == [1, 2, 3, 4]
print("PASS")


buf2 = Buffer(data=[10.0, 20.0, 30.0, 40.0])
assert len(buf2) == 4
assert buf2[0] == 10.0
assert buf2[-1] == 40.0
assert buf2[1:3] == [20.0, 30.0]

buf2[2] = 99.0
assert buf2[2] == 99.0

buf2[0:2] = [111.0, 222.0]
assert buf2[0] == 111.0
assert buf2[1] == 222.0
print("PASS")


double_shader = Shader("""
@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@compute @workgroup_size(4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    data[id.x] = data[id.x] * 2.0;
}
""")

buf3 = Buffer(data=[1.0, 2.0, 3.0, 4.0])
compute3 = Compute(double_shader)
compute3.set(data=buf3)
compute3.dispatch(1, 1, 1)
assert buf3.read() == [2.0, 4.0, 6.0, 8.0]
print("PASS")


compute3.dispatch(1, 1, 1)
assert buf3.read() == [4.0, 8.0, 12.0, 16.0]
print("PASS")


wg_shader = Shader("""
@group(0) @binding(0)
var<storage, read_write> output: array<u32>;

@compute @workgroup_size(4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    output[id.x] = id.x + 1u;
}
""")

buf5 = Buffer(size=32)
compute5 = Compute(wg_shader)
compute5.set(output=buf5)
compute5.dispatch(2, 1, 1)
assert list(struct.unpack("<8I", buf5.read())) == [1, 2, 3, 4, 5, 6, 7, 8]
print("PASS")


copy_shader = Shader("""
@group(0) @binding(0) var<storage, read>       src: array<f32>;
@group(0) @binding(1) var<storage, read_write> dst: array<f32>;

@compute @workgroup_size(4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    dst[id.x] = src[id.x] * 10.0;
}
""")

src_buf = Buffer(data=[1.0, 2.0, 3.0, 4.0])
dst_buf = Buffer(size=16)
compute6 = Compute(copy_shader)
compute6.set(src=src_buf, dst=dst_buf)
compute6.dispatch(1, 1, 1)
assert list(struct.unpack("<4f", dst_buf.read())) == [10.0, 20.0, 30.0, 40.0]
print("PASS")

g.end_draw()