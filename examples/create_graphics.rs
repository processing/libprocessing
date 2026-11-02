use processing::{Graphics, Result, Settings, Sketch};

struct CreateGraphics {
    pg: Graphics,
}

impl Sketch for CreateGraphics {
    fn settings() -> Settings {
        Settings::size(640, 360)
    }

    fn setup(_g: &mut Graphics) -> Result<Self> {
        let pg = Graphics::create(400, 200)?;
        Ok(CreateGraphics { pg })
    }

    fn draw(&mut self, g: &mut Graphics) {
        let mx = g.mouse_x();
        let my = g.mouse_y();

        g.fill((0, 12));
        g.rect(0, 0, g.width(), g.height());
        g.fill(255);
        g.no_stroke();
        g.ellipse(mx, my, 60, 60);

        let mut pg = self.pg.begin_draw();
        pg.background(51);
        pg.no_fill();
        pg.stroke(255);
        pg.ellipse(mx - 120.0, my - 60.0, 60, 60);
        drop(pg);

        g.image(&self.pg, 120, 60);
    }
}

fn main() -> Result<()> {
    processing::run::<CreateGraphics>()
}
