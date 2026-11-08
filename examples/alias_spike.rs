use processing::{Buffer, Compute, Graphics, Result, Settings, Shader, Sketch};

const TWO_BINDING_SRC: &str = r#"
@group(0) @binding(0) var<storage, read>       src: array<f32>;
@group(0) @binding(1) var<storage, read_write> dst: array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= arrayLength(&dst) { return; }
    dst[i] = src[i] * 2.0;
}
"#;

const ONE_BINDING_SRC: &str = r#"
@group(0) @binding(0) var<storage, read_write> data: array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= arrayLength(&data) { return; }
    data[i] = data[i] * 2.0;
}
"#;

struct AliasSpike;

impl Sketch for AliasSpike {
    fn settings() -> Settings {
        Settings::size(1, 1).headless()
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        let base = [1.0f32, 2.0, 3.0, 4.0];
        let mut two = Compute::new(&Shader::new(TWO_BINDING_SRC)?)?;

        let a_src = Buffer::from_slice(&base)?;
        let a_dst = Buffer::from_slice(&[0.0f32; 4])?;
        two.set("src", &a_src);
        two.set("dst", &a_dst);
        two.dispatch(1, 1, 1);
        println!("A distinct read+read_write  -> {:?}", a_dst.read::<f32>()?);

        let mut one = Compute::new(&Shader::new(ONE_BINDING_SRC)?)?;
        let c = Buffer::from_slice(&base)?;
        one.set("data", &c);
        one.dispatch(1, 1, 1);
        println!(
            "C single read_write in-place-> {:?} (want [2,4,6,8])",
            c.read::<f32>()?
        );

        let b = Buffer::from_slice(&base)?;
        two.set("src", &b);
        two.set("dst", &b);
        println!("B aliased read+read_write   -> dispatching (expect fatal Validation Error)...");
        two.dispatch(1, 1, 1);
        println!(
            "B aliased read+read_write   -> {:?} (want [2,4,6,8])",
            b.read::<f32>()?
        );

        g.exit();
        Ok(AliasSpike)
    }
}

fn main() -> Result<()> {
    processing::run::<AliasSpike>()
}
