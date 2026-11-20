use processing::prelude::*;

struct GltfLoad {
    duck: Geometry,
    skin: Material,
    light: Light,
}

impl Sketch for GltfLoad {
    fn settings() -> Settings {
        Settings::size(800, 600)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();

        let gltf = Gltf::load(g, "gltf/Duck.glb")?;
        let duck = gltf.geometry("LOD3spShape")?;
        let skin = gltf.material("blinn3-fx")?;
        gltf.camera(0)?;
        let light = gltf.light(0)?;

        Ok(GltfLoad { duck, skin, light })
    }

    fn draw(&mut self, g: &mut Graphics) {
        let t = (g.frame_count() - 1) as f32 * 0.02;

        let radius = 1.5;
        self.light
            .position((t.cos() * radius, -1.5, t.sin() * radius))
            .look_at((0.0, -0.8, 0.0));

        let r = (t * 8.0).sin() * 0.5 + 0.5;
        let green = (t * 8.0 + 2.0).sin() * 0.5 + 0.5;
        let b = (t * 8.0 + 4.0).sin() * 0.5 + 0.5;
        self.skin.set("base_color", (r, green, b, 1.0));

        g.light(&self.light);
        g.background((0.1, 0.1, 0.12));
        g.material(&self.skin);
        g.model(&self.duck);
    }
}

fn main() -> Result<()> {
    processing::run::<GltfLoad>()
}
