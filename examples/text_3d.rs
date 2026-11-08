use processing::{Geometry, Graphics, Light, Material, Result, Settings, Sketch, TextStyle};

struct Text3d {
    word: Geometry,
    glow: Material,
    sun: Light,
    t: f32,
}

impl Sketch for Text3d {
    fn settings() -> Settings {
        Settings::size(1200, 700)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0, 0, 800));
        g.camera_look_at((0, 0, 0));

        let sun = g.create_directional_light((0.3, 0.3, 0.4), 2000);
        sun.position((200, -300, 500)).look_at((0, 0, 0));

        let mut glow = Material::pbr()?;
        glow.set("roughness", 0.3);
        glow.set("metallic", 0.5);
        glow.set("emissive", (2.0, 0.5, 3.0, 1.0));

        g.text_size(120);
        g.text_style(TextStyle::Bold);
        let w = g.text_width("Processing");
        let word = g.text_to_model("Processing", -w / 2.0, 0, 40)?;

        Ok(Text3d {
            word,
            glow,
            sun,
            t: 0.0,
        })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.light(&self.sun);
        g.background((0.02, 0.02, 0.03));
        g.fill((0.8, 0.3, 1.0));
        g.material(&self.glow);

        {
            let mut g = g.push();
            g.scale((15, 15));
            g.rotate(-self.t * 0.3);
            g.model(&self.word);
        }

        self.t += 0.016;
    }
}

fn main() -> Result<()> {
    processing::run::<Text3d>()
}
