fn falloff(d: f32, radius: f32, mode: u32) -> f32 {
    let n = 1.0 - d / radius;
    switch mode {
        case 1u: { return n; }
        case 2u: { return n * n * (3.0 - 2.0 * n); }
        case 3u: { return n * n; }
        case 4u: { return n * n * n; }
        case 5u: { return radius / (d + radius); }
        default: { return 1.0; }
    }
}

fn cell_coords(p: vec3<f32>, grid_min: vec3<f32>, cell_size: f32, dims: vec3<u32>) -> vec3<i32> {
    let rel = (p - grid_min) / cell_size;
    return vec3<i32>(
        clamp(i32(floor(rel.x)), 0, i32(dims.x) - 1),
        clamp(i32(floor(rel.y)), 0, i32(dims.y) - 1),
        clamp(i32(floor(rel.z)), 0, i32(dims.z) - 1),
    );
}

fn cell_index(c: vec3<u32>, dims: vec3<u32>) -> u32 {
    return c.x + c.y * dims.x + c.z * dims.x * dims.y;
}

fn cell_of(p: vec3<f32>, grid_min: vec3<f32>, cell_size: f32, dims: vec3<u32>) -> u32 {
    return cell_index(vec3<u32>(cell_coords(p, grid_min, cell_size, dims)), dims);
}
