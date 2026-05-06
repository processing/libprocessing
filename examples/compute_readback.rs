use processing::prelude::*;

fn main() {
    match run() {
        Ok(_) => {
            eprintln!("Compute readback test passed!");
            exit(0).unwrap();
        }
        Err(e) => {
            eprintln!("Compute readback error: {:?}", e);
            exit(1).unwrap();
        }
    }
}

fn run() -> error::Result<()> {
    init(Config::default())?;

    let surface = surface_create_offscreen(1, 1, 1.0, TextureFormat::Rgba8Unorm)?;
    let _graphics = graphics_create(surface, 1, 1, TextureFormat::Rgba8Unorm)?;

    let buf = buffer_create(16)?;

    let shader_src = r#"
@group(0) @binding(0)
var<storage, read_write> output: array<u32>;

@compute @workgroup_size(1)
fn main() {
    output[0] = 1u;
    output[1] = 2u;
    output[2] = 3u;
    output[3] = 4u;
}
"#;
    let shader = shader_create(shader_src)?;
    let compute = compute_create(shader)?;
    compute_set(compute, "output", shader_value::ShaderValue::Buffer(buf))?;

    compute_dispatch(compute, 1, 1, 1)?;

    let data = buffer_read(buf)?;
    let values: Vec<u32> = data
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();

    assert_eq!(values, vec![1, 2, 3, 4], "Compute readback mismatch!");
    eprintln!("PASS");

    let double_src = r#"
@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@compute @workgroup_size(4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    data[id.x] = data[id.x] * 2.0;
}
"#;
    let buf2_data: Vec<u8> = [1.0f32, 2.0, 3.0, 4.0]
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    let buf2 = buffer_create_with_data(buf2_data)?;
    let shader2 = shader_create(double_src)?;
    let compute2 = compute_create(shader2)?;
    compute_set(compute2, "data", shader_value::ShaderValue::Buffer(buf2))?;
    compute_dispatch(compute2, 1, 1, 1)?;

    let data2 = buffer_read(buf2)?;
    let floats: Vec<f32> = data2
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    assert_eq!(
        floats,
        vec![2.0, 4.0, 6.0, 8.0],
        "In-place double mismatch!"
    );
    eprintln!("PASS");

    compute_destroy(compute)?;
    compute_destroy(compute2)?;
    shader_destroy(shader)?;
    shader_destroy(shader2)?;
    buffer_destroy(buf)?;
    buffer_destroy(buf2)?;

    Ok(())
}
