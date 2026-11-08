use processing::{Geometry, Graphics, Result, Settings, Sketch, Topology};

const GRID: u32 = 20;
const SPACING: f32 = 10.0;

struct AnimatedMesh {
    mesh: Geometry,
    time: f32,
}

impl Sketch for AnimatedMesh {
    fn settings() -> Settings {
        Settings::size(600, 600)
    }

    fn setup(g: &mut Graphics) -> Result<Self> {
        g.mode_3d();
        g.camera_position((150, -150, 150));
        g.camera_look_at((0, 0, 0));

        let offset = GRID as f32 * SPACING / 2.0;
        let mut mesh = Geometry::new(Topology::TriangleList)?;
        for z in 0..GRID {
            for x in 0..GRID {
                let px = x as f32 * SPACING - offset;
                let pz = z as f32 * SPACING - offset;
                mesh.color((x as f32 / GRID as f32, 0.5, z as f32 / GRID as f32));
                mesh.normal((0, -1, 0));
                mesh.vertex((px, 0.0, pz));
            }
        }

        for z in 0..GRID - 1 {
            for x in 0..GRID - 1 {
                let top_left = z * GRID + x;
                let top_right = top_left + 1;
                let bottom_left = (z + 1) * GRID + x;
                let bottom_right = bottom_left + 1;
                mesh.index(top_left);
                mesh.index(bottom_left);
                mesh.index(top_right);
                mesh.index(top_right);
                mesh.index(bottom_left);
                mesh.index(bottom_right);
            }
        }

        Ok(AnimatedMesh { mesh, time: 0.0 })
    }

    fn draw(&mut self, g: &mut Graphics) {
        let offset = GRID as f32 * SPACING / 2.0;
        for z in 0..GRID {
            for x in 0..GRID {
                let px = x as f32 * SPACING - offset;
                let pz = z as f32 * SPACING - offset;
                let wave = (px * 0.1 + self.time).sin() * (pz * 0.1 + self.time).cos() * 20.0;
                self.mesh.set_vertex(z * GRID + x, (px, -wave, pz));
            }
        }

        g.background((0.05, 0.05, 0.1));
        g.model(&self.mesh);

        self.time += 0.05;
    }
}

fn main() -> Result<()> {
    processing::run::<AnimatedMesh>()
}
