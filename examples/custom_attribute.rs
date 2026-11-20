use processing::prelude::*;

struct CustomAttribute {
    mesh: Geometry,
}

impl Sketch for CustomAttribute {
    fn settings() -> Settings {
        Settings::size(600, 600)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((0, 0, 200));
        g.camera_look_at((0, 0, 0));

        let custom = Attribute::new("Custom", AttributeFormat::Float)?;
        let layout = Layout::new()?.position().normal().color().attribute(custom);

        let mut mesh = Geometry::with_layout(&layout, Topology::LineStrip)?;
        mesh.color((1.0, 0.0, 0.0));
        mesh.normal((0, 0, 1));
        mesh.attribute(custom, 0.0);
        mesh.vertex((-50, 50));

        mesh.color((0.0, 1.0, 0.0));
        mesh.attribute(custom, 0.5);
        mesh.vertex((50, 50));

        mesh.color((0.0, 0.0, 1.0));
        mesh.attribute(custom, 1.0);
        mesh.vertex((0, -50));

        mesh.index(0);
        mesh.index(1);
        mesh.index(2);
        mesh.index(0);

        Ok(CustomAttribute { mesh })
    }

    fn draw(&mut self, g: &mut Graphics) {
        g.background((0.1, 0.1, 0.12));
        g.model(&self.mesh);
    }
}

fn main() -> Result<()> {
    processing::run::<CustomAttribute>()
}
