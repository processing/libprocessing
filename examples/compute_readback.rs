use processing::{Buffer, Compute, Graphics, Result, Settings, Shader, Sketch};

const FILL: &str = r#"
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

const DOUBLE: &str = r#"
@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@compute @workgroup_size(4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    data[id.x] = data[id.x] * 2.0;
}
"#;

struct ComputeReadback;

impl Sketch for ComputeReadback {
    fn settings() -> Settings {
        Settings::size(1, 1).headless()
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        let output = Buffer::new(16)?;
        let mut fill = Compute::new(&Shader::new(FILL)?)?;
        fill.set("output", &output);
        fill.dispatch(1, 1, 1);
        let values: Vec<u32> = output.read()?;
        assert_eq!(values, vec![1, 2, 3, 4]);

        let data = Buffer::from_slice(&[1.0f32, 2.0, 3.0, 4.0])?;
        let mut double = Compute::new(&Shader::new(DOUBLE)?)?;
        double.set("data", &data);
        double.dispatch(1, 1, 1);
        let doubled: Vec<f32> = data.read()?;
        assert_eq!(doubled, vec![2.0, 4.0, 6.0, 8.0]);

        println!("{values:?} {doubled:?}");
        g.exit();
        Ok(ComputeReadback)
    }
}

fn main() -> Result<()> {
    processing::run::<ComputeReadback>()
}
