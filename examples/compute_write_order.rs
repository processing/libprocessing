use processing::prelude::*;
use processing_render::geometry::AttributeFormat;
use processing_render::{GridParams, grid_build, grid_create, grid_get};

fn main() {
    match run() {
        Ok(_) => exit(0).unwrap(),
        Err(e) => {
            eprintln!("{e:?}");
            exit(1).unwrap();
        }
    }
}

fn u32s(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn f32s(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn bytes_f32(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn run() -> error::Result<()> {
    init(Config::default())?;
    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    // partial writes after a kernel
    let double = compute_create(shader_create(
        r#"
@group(0) @binding(0) var<storage, read_write> data: array<f32>;
@compute @workgroup_size(4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    data[id.x] = data[id.x] * 2.0;
}
"#,
    )?)?;
    let buf = buffer_create_with_data(bytes_f32(&[1.0, 2.0, 3.0, 4.0]))?;
    compute_set(double, "data", shader_value::ShaderValue::Buffer(buf))?;
    compute_dispatch(double, 1, 1, 1)?;
    buffer_write_element(buf, 4, bytes_f32(&[99.0]))?; // asset unsynced: GPU only
    assert_eq!(f32s(&buffer_read(buf)?), [2.0, 99.0, 6.0, 8.0]);
    buffer_write_element(buf, 0, bytes_f32(&[7.0]))?; // asset synced: GPU and asset
    assert_eq!(f32s(&buffer_read(buf)?), [7.0, 99.0, 6.0, 8.0]);
    compute_dispatch(double, 1, 1, 1)?;
    assert_eq!(f32s(&buffer_read(buf)?), [14.0, 198.0, 12.0, 16.0]);
    // a later update must not re-upload over the kernel's output
    let _ = buffer_create(4)?;
    assert_eq!(f32s(&buffer_read(buf)?), [14.0, 198.0, 12.0, 16.0]);
    println!("partial writes: ok");

    // grid rebuilds must see each CPU write
    let params = GridParams {
        min: [0.0, 0.0, 0.0],
        cell_size: 1.0,
        dims: [4, 1, 1],
    };
    let n = 64u32;
    let grid = grid_create(params, n)?;
    let position = buffer_create((n * 3 * 4) as u64)?;
    for round in 0..4u32 {
        let xs: Vec<f32> = (0..n)
            .map(|i| ((i * (round + 1)) % 4) as f32 + 0.5)
            .collect();
        let packed: Vec<f32> = xs.iter().flat_map(|&x| [x, 0.5, 0.5]).collect();
        buffer_write(position, bytes_f32(&packed))?;
        grid_build(grid, position)?;
        let offsets = u32s(&buffer_read(grid_get(grid)?.offsets)?);
        let mut expected = [0u32; 4];
        for x in &xs {
            expected[*x as usize] += 1;
        }
        let got: Vec<u32> = (0..4).map(|c| offsets[c + 1] - offsets[c]).collect();
        assert_eq!(got, expected, "round {round}: offsets {offsets:?}");
    }
    println!("grid rebuilds after CPU writes: ok");

    let values: Vec<u32> = (0..3000).map(|i| i % 7).collect();
    let scan = buffer_create_with_data(values.iter().flat_map(|v| v.to_le_bytes()).collect())?;
    prefix_sum_u32(scan)?;
    let got = u32s(&buffer_read(scan)?);
    let mut acc = 0u32;
    let exclusive: Vec<u32> = values
        .iter()
        .map(|v| {
            let s = acc;
            acc += v;
            s
        })
        .collect();
    let mut acc = 0u32;
    let inclusive: Vec<u32> = values
        .iter()
        .map(|v| {
            acc += v;
            acc
        })
        .collect();
    assert!(
        got == exclusive || got == inclusive,
        "scan mismatch: {:?}",
        &got[..16]
    );
    println!("prefix sum: ok");

    // written before its GPU buffer exists
    let attr = geometry_attribute_create("heat", AttributeFormat::Float)?;
    let particles = particles_create(4, vec![attr])?;
    let heat = particles_buffer(particles, attr)?.expect("heat buffer");
    buffer_write(heat, bytes_f32(&[1.0, 2.0, 3.0, 4.0]))?;
    assert_eq!(f32s(&buffer_read(heat)?), [1.0, 2.0, 3.0, 4.0]);
    compute_set(double, "data", shader_value::ShaderValue::Buffer(heat))?;
    compute_dispatch(double, 1, 1, 1)?;
    assert_eq!(f32s(&buffer_read(heat)?), [2.0, 4.0, 6.0, 8.0]);
    println!("write before prepare: ok");

    Ok(())
}
